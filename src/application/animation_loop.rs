use std::cell::RefCell;
use std::ops::{Deref};
use std::rc::Rc;
use std::time::Duration;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext, WebGlProgram};
use crate::application::control::Controller;
use crate::application::lighting::LightParameters;
use crate::application::planet::load_planet_terrain;
use crate::application::sphere::generate_sphere;
use crate::application::vertex::BasicMesh;
use crate::render_core::animation::{AnimationFn, wrap_animation_body};
use crate::render_core::camera::Camera;
use crate::render_core::mesh::{add_mesh, clear_frame, draw_meshes, DrawBuffers, DrawMode, MeshMode};
use crate::render_core::smart_uniform::SmartUniform;
use crate::Viewport;

use crate::utils::prelude::*;

pub fn get_animation_loop(canvas: HtmlCanvasElement, context: WebGl2RenderingContext, program: WebGlProgram)
        -> Result<AnimationFn, JsValue> {
    let mut lighting = LightParameters::new(&context, &program);

    let mut texture_map = SmartUniform::<i32>::new("s_textureMap", context.clone(), program.clone());
    texture_map.smart_write(0);

    let mut terrain_scale = SmartUniform::<f32>::new("u_terrainScale", context.clone(), program.clone());
    terrain_scale.smart_write(0.1);

    let mut model = SmartUniform::<nglm::Mat4>::new("u_model", context.clone(), program.clone());
    let mut view = SmartUniform::<nglm::Mat4>::new("u_view", context.clone(), program.clone());
    let mut projection = SmartUniform::<nglm::Mat4>::new("u_projection", context.clone(), program.clone());

    let camera = Rc::new(RefCell::new(Camera::new(&nglm::vec3(0.0, 0.0, 3.0), // Sigh, backwards y...
                                                  &nglm::vec3(0.0, 0.0, 0.0))));

    let mut controller = Controller::new(canvas, &camera, terrain_scale.clone());

    let sphere_meshes = generate_sphere(20, 20);
    load_planet_terrain(context.clone())?;

    let buffers: Vec<DrawBuffers> = sphere_meshes.iter()
        .map(|m| {
            add_mesh(&context, &program, m, MeshMode::Static).unwrap()
        }).collect();

    let meshes_and_buffers: Vec<(BasicMesh, DrawBuffers)> = sphere_meshes.into_iter().zip(buffers.into_iter()).collect();


    // TODO: Don't run frames all the time, just run when there's input. That's what Google Maps does.
    // If no uniforms and no vertices have changed, then I don't have to render anything new. I can implement
    // that abstraction behind two interfaces, I think.


    // let mut frustum_test_camera = Camera::new(&nglm::vec3(1.1, 0.0, 0.0),
    //                                       &nglm::vec3(0.0, 0.0, 0.0));


    Ok(wrap_animation_body(move |viewport: &Viewport, _delta_time: Duration| {
        // {
        //     frustum_test_camera.orbit_around_target(&nglm::zero(),
        //                                             &nglm::vec2(_delta_time.as_millis() as f32, 0.0),
        //                                             0.05);
        // }

        controller.frame();

        let camera_position = camera.deref().borrow().position();
        lighting.camera_position.smart_write(camera_position.clone());
        lighting.light_position.smart_write(camera_position - nglm::vec3(0.0, 0.5, 0.3));

        let width = viewport.width() as i32;
        let height = viewport.height() as i32;

        let mvp = camera.deref().borrow().get_perspective_matrices(width, height);

        model.smart_write(mvp.model.clone());
        view.smart_write(mvp.view.clone());
        projection.smart_write(mvp.projection.clone());

        clear_frame(viewport.context());
        draw_meshes(viewport.context(), camera.deref().borrow().deref(), &meshes_and_buffers, DrawMode::Surface);
    }))
}
