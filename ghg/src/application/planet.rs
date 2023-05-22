use std::cell::{Cell, RefCell, RefMut};
use std::future::join;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::time::Duration;

use image::{Luma, Rgb};
use single_thread_executor::Spawner;
use wasm_bindgen::JsValue;
use web_sys::WebGl2RenderingContext;

use crate::application::lighting::LightParameters;
use crate::application::shaders::ShaderContext;
use crate::application::sphere::generate_sphere;
use crate::application::vertex::BasicMesh;
use crate::render_core::animation_params::AnimationParams;
use crate::render_core::camera::Camera;
use crate::render_core::frame_sequencer::FrameGate;
use crate::render_core::image::load_into_texture;
use crate::render_core::mesh::{
	add_mesh, clear_frame, draw_meshes, DrawBuffers, DrawMode, MeshMode,
};
use crate::render_core::uniform;
use crate::request_data::fetch_bytes;
#[allow(unused_imports)]
use crate::utils::prelude::*;

async fn load_planet_terrain(context: WebGl2RenderingContext) -> Result<(), JsValue> {
	let texture = fetch_bytes("images/earth_height/2/full.png").await?;
	load_into_texture::<Luma<u8>>(context, &texture, WebGl2RenderingContext::TEXTURE0)
}

async fn load_planet_color(context: WebGl2RenderingContext) -> Result<(), JsValue> {
	let texture = fetch_bytes("images/earth_color/2/full.png").await?;
	load_into_texture::<Rgb<u8>>(context, &texture, WebGl2RenderingContext::TEXTURE1)?;
	Ok(())
}

async fn load_all_textures(context: WebGl2RenderingContext, done: Rc<Cell<bool>>) {
	let (color_result, terrain_result) =
		join!(load_planet_color(context.clone()), load_planet_terrain(context.clone()),).await;

	assert!(color_result.is_ok(), "Color load failed");
	assert!(terrain_result.is_ok(), "Terrain load failed");

	remove_overlay();
	done.replace(true);
}

// TODO: Move to module
#[wasm_bindgen(module = "/www/overlay.js")]
extern "C" {
	fn remove_overlay();
}

fn generate_drawable_sphere(
	subdivisions: u32,
	points_per_subdivision: u32,
	shader_context: ShaderContext,
) -> Vec<(BasicMesh, DrawBuffers)> {
	let sphere_meshes = generate_sphere(subdivisions, points_per_subdivision);
	let buffers: Vec<DrawBuffers> = sphere_meshes
		.iter()
		.map(|m| add_mesh(&shader_context, m, MeshMode::Static).unwrap())
		.collect();

	sphere_meshes.into_iter().zip(buffers.into_iter()).collect()
}

pub async fn load_textures(
	gate: FrameGate<AnimationParams>,
	spawner: Spawner,
	context: WebGl2RenderingContext,
	camera: Rc<RefCell<Camera>>,
) {
	let textures_loaded = Rc::new(Cell::new(false));
	spawner.spawn(load_all_textures(context, textures_loaded.clone()));

	let mut initial_spin = 3.0f32;
	let mut spinner = move |delta_time: Duration, mut camera: RefMut<Camera>| -> bool {
		if initial_spin > 0.0 {
			let mut cam = camera.deref_mut();
			let spin_amount = initial_spin * 0.5f32;
			cam.deref_mut().orbit_around_target(
				&nglm::zero(),
				&nglm::vec2(-spin_amount, spin_amount.min(0.2)),
				0.5,
			);
			initial_spin -= delta_time.as_secs_f32();
			true
		} else {
			false
		}
	};

	loop {
		let params = (&gate).await;

		if textures_loaded.get() {
			if !spinner(params.delta_time, camera.deref().borrow_mut()) {
				break;
			}
		}
	}
}

pub async fn draw(
	gate: FrameGate<AnimationParams>,
	shader: ShaderContext,
	camera: Rc<RefCell<Camera>>,
) {
	let mut frustum_test_camera =
		Camera::new(&nglm::vec3(1.1, 0.0, 0.0), &nglm::vec3(0.0, 0.0, 0.0));
	const DEBUG_FRUSTUM: bool = false;

	shader.use_shader();

	let planet_meshes_and_buffers = generate_drawable_sphere(10, 10, shader.clone());

	let mut lighting = LightParameters::new(&shader);

	uniform::init_i32("s_textureMap", &shader, 0);
	uniform::init_i32("s_colorMap", &shader, 1);

	// TODO: Need to make a uniform that can bind into multiple shader programs
	let mut planet_model = uniform::new_smart_mat4("u_model", &shader);
	let mut planet_view = uniform::new_smart_mat4("u_view", &shader);
	let mut planet_projection = uniform::new_smart_mat4("u_projection", &shader);

	loop {
		let params = (&gate).await;

		if DEBUG_FRUSTUM {
			frustum_test_camera.orbit_around_target(
				&nglm::zero(),
				&nglm::vec2(params.delta_time.as_millis() as f32, 0.0),
				0.05,
			);
		}

		clear_frame(params.viewport.context());

		let camera_position = camera.deref().borrow().position();
		lighting.camera_position.smart_write(camera_position.clone());
		lighting.light_position.smart_write(camera_position.clone()); // - nglm::vec3(0.0, 0.5, 0.3)

		let width = params.viewport.width() as i32;
		let height = params.viewport.height() as i32;

		let mvp = camera.deref().borrow().get_perspective_matrices(width, height);

		if DEBUG_FRUSTUM {
			planet_model.smart_write(mvp.model.clone());
			planet_view.smart_write(mvp.view.clone());
			planet_projection.smart_write(mvp.projection.clone());

			draw_meshes(
				params.viewport.context(),
				&frustum_test_camera,
				&planet_meshes_and_buffers,
				DrawMode::Surface,
			);
		} else {
			planet_model.smart_write(mvp.model.clone());
			planet_view.smart_write(mvp.view.clone());
			planet_projection.smart_write(mvp.projection.clone());

			draw_meshes(
				params.viewport.context(),
				camera.deref().borrow().deref(),
				&planet_meshes_and_buffers,
				DrawMode::Surface,
			);
		}
	}
}
