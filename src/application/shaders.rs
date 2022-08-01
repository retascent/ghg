use web_sys::{WebGl2RenderingContext, WebGlProgram};
use crate::render_core::shader;

pub fn get_shaders(context: &WebGl2RenderingContext) -> Result<WebGlProgram, String> {
    let vert_shader = shader::compile_shader(
        &context,
        WebGl2RenderingContext::VERTEX_SHADER,
        include_str!("shaders/planet.vert"),
    )?;

    let frag_shader = shader::compile_shader(
        &context,
        WebGl2RenderingContext::FRAGMENT_SHADER,
        include_str!("shaders/planet.frag"),
    )?;

    shader::link_program(&context, &vert_shader, &frag_shader)
}
