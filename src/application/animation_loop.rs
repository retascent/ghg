use std::cell::{RefCell, RefMut};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::time::Duration;
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};
use crate::application::control::Controller;
use crate::application::lighting::LightParameters;
use crate::application::planet::{load_planet_color, load_planet_terrain};
use crate::application::shaders::get_shaders;
use crate::application::sphere::generate_sphere;
use crate::application::vertex::BasicMesh;
use crate::render_core::animation::{AnimationFn, wrap_animation_body};
use crate::render_core::camera::Camera;
use crate::render_core::mesh::{add_mesh, clear_frame, draw_meshes, DrawBuffers, DrawMode, MeshMode};
use crate::render_core::uniform;
use crate::render_core::uniform::ShaderContext;
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
            *terrain_loaded.borrow_mut().deref_mut() = true;
            if *color_loaded.borrow().deref() {
                *done.borrow_mut().deref_mut() = true;
                remove_overlay();
            }
        });
    }

    {
        clone_all!(context, done, terrain_loaded, color_loaded);
        spawn_local(async move {
            load_planet_color(context).await.expect("Failed to download color texture!");
            *color_loaded.borrow_mut().deref_mut() = true;
            if *terrain_loaded.borrow().deref() {
                *done.borrow_mut().deref_mut() = true;
                remove_overlay();
            }
        });
    }
}

pub fn get_animation_loop(canvas: HtmlCanvasElement, context: WebGl2RenderingContext)
        -> Result<AnimationFn, JsValue> {

    // Debug camera for testing frustum culling
    let mut frustum_test_camera = Camera::new(&nglm::vec3(1.1, 0.0, 0.0), &nglm::vec3(0.0, 0.0, 0.0));
    const DEBUG_FRUSTUM: bool = false;

    let camera = Rc::new(RefCell::new(Camera::new(&nglm::vec3(0.0, 0.0, 3.0),
                                                  &nglm::vec3(0.0, 0.0, 0.0))));

    let program = get_shaders(&context)?;
    context.use_program(Some(&program));

    let sphere_meshes = generate_sphere(20, 20);
    let buffers: Vec<DrawBuffers> = sphere_meshes.iter()
        .map(|m| {
            add_mesh(&context, &program, m, MeshMode::Static).unwrap()
        }).collect();

    let meshes_and_buffers: Vec<(BasicMesh, DrawBuffers)> = sphere_meshes.into_iter().zip(buffers.into_iter()).collect();

    let shader_context = ShaderContext::new(&context, &program);
    let terrain_scale = uniform::init_f32("u_terrainScale", &shader_context, 0.03);
    let mut controller = Controller::new(canvas, &camera, terrain_scale);

    let mut lighting = LightParameters::new(&shader_context);

    uniform::init_i32("s_textureMap", &shader_context, 0);
    uniform::init_i32("s_colorMap", &shader_context, 1);

    let mut model = uniform::new_smart_mat4("u_model", &shader_context);
    let mut view = uniform::new_smart_mat4("u_view", &shader_context);
    let mut projection = uniform::new_smart_mat4("u_projection", &shader_context);

    let textures_loaded = Rc::new(RefCell::new(false));
    load_textures_async(context.clone(), textures_loaded.clone());

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

        controller.frame();

        clear_frame(viewport.context());

        let camera_position = camera.deref().borrow().position();
        lighting.camera_position.smart_write(camera_position.clone());
        lighting.light_position.smart_write(camera_position - nglm::vec3(0.0, 0.5, 0.3));

        let width = viewport.width() as i32;
        let height = viewport.height() as i32;

        let mvp = camera.deref().borrow().get_perspective_matrices(width, height);

        model.smart_write(mvp.model.clone());
        view.smart_write(mvp.view.clone());
        projection.smart_write(mvp.projection.clone());

        if DEBUG_FRUSTUM {
            draw_meshes(viewport.context(), &frustum_test_camera, &meshes_and_buffers, DrawMode::Surface);
        } else {
            draw_meshes(viewport.context(), camera.deref().borrow().deref(), &meshes_and_buffers, DrawMode::Surface);
        }
    }))
}
