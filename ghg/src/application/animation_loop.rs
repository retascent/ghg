use std::cell::{RefCell, RefMut};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::time::Duration;

use single_thread_executor::new_executor_and_spawner;
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

use crate::application::control::controller_frame;
use crate::application::frame_params::AnimationParams;
use crate::application::planet;
// use crate::application::data::load_temp_data;
use crate::application::shaders::get_planet_shaders;
use crate::render_core::animation::{wrap_animation_body, AnimationFn};
use crate::render_core::camera::Camera;
use crate::render_core::frame_sequencer::{FrameGate, FrameMarker, FrameSequencer};
use crate::utils::prelude::*;
use crate::Viewport;

pub fn get_animation_loop(
	canvas: HtmlCanvasElement,
	context: WebGl2RenderingContext,
) -> Result<AnimationFn, JsValue> {
	let (executor, spawner) = new_executor_and_spawner();
	spawn_local(async move {
		executor.run().await;
	});

	let camera =
		Rc::new(RefCell::new(Camera::new(&nglm::vec3(0.0, 0.0, 3.0), &nglm::vec3(0.0, 0.0, 0.0))));

	let planet_shader = get_planet_shaders(&context)?;

	let frame_sequencer = Rc::new(FrameSequencer::<AnimationParams>::new());
	spawner.spawn(planet::load_textures(
		FrameGate::new(frame_sequencer.clone(), "Load Textures".to_owned()),
		spawner.clone(),
		context.clone(),
		camera.clone(),
	));

	spawner.spawn(controller_frame(
		FrameGate::new(frame_sequencer.clone(), "Controller".to_owned()),
		canvas.clone(),
		planet_shader.clone(),
		camera.clone(),
	));

	spawner.spawn(planet::draw(
		FrameGate::new(frame_sequencer.clone(), "Draw Planet".to_owned()),
		planet_shader.clone(),
		camera.clone(),
	));

	let frame_marker = FrameMarker::new(frame_sequencer.clone());

	Ok(wrap_animation_body(move |viewport: &Viewport, delta_time: Duration| {
		frame_marker.frame(AnimationParams { viewport: viewport.clone(), delta_time });
	}))
}
