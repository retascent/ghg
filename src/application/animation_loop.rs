use std::cell::{RefCell, RefMut};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::time::Duration;
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};
use crate::application::control::Controller;
use crate::application::data::load_temp_data;
use crate::application::lighting::LightParameters;
use crate::application::planet::{load_planet_color, load_planet_terrain};
use crate::application::shaders::{get_data_shaders, get_planet_shaders, ShaderContext};
use crate::application::sphere::{generate_sphere, generate_spikey_sphere};
use crate::application::vertex::BasicMesh;
use crate::render_core::animation::{AnimationFn, wrap_animation_body};
use crate::render_core::camera::Camera;
use crate::render_core::mesh::{add_mesh, clear_frame, draw_meshes, DrawBuffers, DrawMode, MeshMode};
use crate::render_core::uniform;
use crate::utils::{assign_shared, read_shared};
use crate::Viewport;

use crate::utils::prelude::*;

#[wasm_bindgen(module = "/www/overlay.js")]
extern {
    fn remove_overlay();
}

fn load_textures_async(context: WebGl2RenderingContext, done: Rc<RefCell<bool>>) {
    let terrain_loaded = Rc::new(RefCell::new(false));
    let color_loaded = Rc::new(RefCell::new(false));

    // This pattern is still a work in progress. It's not clean, and would be much better and
    // easier to understand if I used a real graph representation or something.
    {
        clone_all!(context, done, terrain_loaded, color_loaded);
        spawn_local(async move {
            load_planet_terrain(context).await.expect("Failed to download terrain texture!");
            assign_shared(&terrain_loaded, true);
            if read_shared(&color_loaded) {
                assign_shared(&done, true);
                remove_overlay();
            }
        });
    }

    {
        clone_all!(context, done, terrain_loaded, color_loaded);
        spawn_local(async move {
            load_planet_color(context).await.expect("Failed to download color texture!");
            assign_shared(&color_loaded, true);
            if read_shared(&terrain_loaded) {
                assign_shared(&done, true);
                remove_overlay();
            }
        });
    }
}

fn load_temp_data_async(shader_context: ShaderContext, done: Rc<RefCell<bool>>) {
    spawn_local(async move {
        load_temp_data(shader_context).await.expect("Failed to download temperature data");
        assign_shared(&done, true);
    })
}

fn generate_drawable_sphere(subdivisions: u32, points_per_subdivision: u32,
                            shader_context: ShaderContext) -> Vec<(BasicMesh, DrawBuffers)> {
    let sphere_meshes = generate_sphere(subdivisions, points_per_subdivision);
    let buffers: Vec<DrawBuffers> = sphere_meshes.iter()
        .map(|m| {
            add_mesh(&shader_context, m, MeshMode::Static).unwrap()
        }).collect();

    sphere_meshes.into_iter()
        .zip(buffers.into_iter())
        .collect()
}

fn generate_drawable_spikey_sphere(subdivisions: u32, points_per_subdivision: u32,
                                   thickness: f64, data_height: f64,
                                   shader_context: ShaderContext) -> Vec<(BasicMesh, DrawBuffers)> {
    let sphere_meshes = generate_spikey_sphere(subdivisions, points_per_subdivision, thickness, data_height);
    let buffers: Vec<DrawBuffers> = sphere_meshes.iter()
        .map(|m| {
            add_mesh(&shader_context, m, MeshMode::Static).unwrap()
        }).collect();

    sphere_meshes.into_iter()
        .zip(buffers.into_iter())
        .collect()
}

pub fn get_animation_loop(canvas: HtmlCanvasElement, context: WebGl2RenderingContext)
        -> Result<AnimationFn, JsValue> {

    // Debug camera for testing frustum culling
    let mut frustum_test_camera = Camera::new(&nglm::vec3(1.1, 0.0, 0.0), &nglm::vec3(0.0, 0.0, 0.0));
    const DEBUG_FRUSTUM: bool = false;

    let camera = Rc::new(RefCell::new(Camera::new(&nglm::vec3(0.0, 0.0, 3.0),
                                                  &nglm::vec3(0.0, 0.0, 0.0))));

    let data_shader = get_data_shaders(&context)?;
    data_shader.use_shader();
    let data_height = 1.35;
    let data_meshes_and_buffers = generate_drawable_spikey_sphere(
        5, 8, 0.01, data_height, data_shader.clone());

    let _data_min_radius = uniform::init_f32("u_dataMinRadius", &data_shader, 1.05);
    let _data_max_radius = uniform::init_f32("u_dataMaxRadius", &data_shader, data_height as f32);
    let _data_scale = uniform::init_f32("u_dataScaleMultiplier", &data_shader, 5.0);

    let planet_shader = get_planet_shaders(&context)?;
    planet_shader.use_shader();
    let planet_meshes_and_buffers = generate_drawable_sphere(
        20, 20, planet_shader.clone());

    let terrain_scale = uniform::init_f32("u_terrainScale", &planet_shader, 0.03);
    let mut controller = Controller::new(canvas, &camera, terrain_scale);

    let mut lighting = LightParameters::new(&planet_shader);

    uniform::init_i32("s_textureMap", &planet_shader, 0);
    uniform::init_i32("s_colorMap", &planet_shader, 1);

    // TODO: Need to make a uniform that can bind into multiple shader programs
    let mut planet_model = uniform::new_smart_mat4("u_model", &planet_shader);
    let mut planet_view = uniform::new_smart_mat4("u_view", &planet_shader);
    let mut planet_projection = uniform::new_smart_mat4("u_projection", &planet_shader);

    let mut data_model = uniform::new_smart_mat4("u_model", &data_shader);
    let mut data_view = uniform::new_smart_mat4("u_view", &data_shader);
    let mut data_projection = uniform::new_smart_mat4("u_projection", &data_shader);

    let textures_loaded = Rc::new(RefCell::new(false));
    load_textures_async(context.clone(), textures_loaded.clone());

    let temp_data_loaded = Rc::new(RefCell::new(false));
    load_temp_data_async(data_shader.clone(), temp_data_loaded.clone());

    let mut initial_spin = 3.0f32;
    let mut do_spin = false;
    let mut spinner = move |delta_time: Duration, mut camera: RefMut<Camera>| {
        if initial_spin > 0.0 {
            let mut cam = camera.deref_mut();
            let spin_amount = initial_spin * 0.5f32;
            cam.deref_mut().orbit_around_target(&nglm::zero(),
                                                &nglm::vec2(-spin_amount, spin_amount.min(0.2)),
                                                0.5);
            initial_spin -= delta_time.as_secs_f32();
        }
    };

    Ok(wrap_animation_body(move |viewport: &Viewport, _delta_time: Duration| {
        if DEBUG_FRUSTUM {
            frustum_test_camera.orbit_around_target(&nglm::zero(),
                                                    &nglm::vec2(_delta_time.as_millis() as f32, 0.0),
                                                    0.05);
        }

        if do_spin {
            spinner(_delta_time, camera.deref().borrow_mut());
        }

        if *textures_loaded.borrow().deref() {
            do_spin = true;
        }

        planet_shader.use_shader();

        controller.frame();

        clear_frame(viewport.context());

        let camera_position = camera.deref().borrow().position();
        lighting.camera_position.smart_write(camera_position.clone());
        lighting.light_position.smart_write(camera_position - nglm::vec3(0.0, 0.5, 0.3));

        let width = viewport.width() as i32;
        let height = viewport.height() as i32;

        let mvp = camera.deref().borrow().get_perspective_matrices(width, height);

        if DEBUG_FRUSTUM {
            planet_model.smart_write(mvp.model.clone());
            planet_view.smart_write(mvp.view.clone());
            planet_projection.smart_write(mvp.projection.clone());

            draw_meshes(viewport.context(), &frustum_test_camera,
                        &planet_meshes_and_buffers, DrawMode::Surface);
        } else {

            planet_model.smart_write(mvp.model.clone());
            planet_view.smart_write(mvp.view.clone());
            planet_projection.smart_write(mvp.projection.clone());

            draw_meshes(viewport.context(), camera.deref().borrow().deref(),
                        &planet_meshes_and_buffers, DrawMode::Surface);

            data_shader.use_shader();

            data_model.smart_write(mvp.model.clone());
            data_view.smart_write(mvp.view.clone());
            data_projection.smart_write(mvp.projection.clone());

            draw_meshes(viewport.context(), camera.deref().borrow().deref(),
                        &data_meshes_and_buffers, DrawMode::Surface);
        }
    }))
}
