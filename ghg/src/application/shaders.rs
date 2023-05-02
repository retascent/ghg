use web_sys::{WebGl2RenderingContext, WebGlProgram};

use crate::render_core::shader;

#[derive(Clone, Debug)]
pub struct ShaderContext {
	pub context: WebGl2RenderingContext,
	pub program: WebGlProgram,
}

impl ShaderContext {
	pub fn new(context: &WebGl2RenderingContext, program: &WebGlProgram) -> Self {
		Self { context: context.clone(), program: program.clone() }
	}

	pub fn use_shader(&self) { self.context.use_program(Some(&self.program)); }
}

pub fn get_planet_shaders(context: &WebGl2RenderingContext) -> Result<ShaderContext, String> {
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

	let program = shader::link_program(&context, &vert_shader, &frag_shader)?;
	Ok(ShaderContext::new(&context, &program))
}

pub fn get_data_shaders(context: &WebGl2RenderingContext) -> Result<ShaderContext, String> {
	let vert_shader = shader::compile_shader(
		&context,
		WebGl2RenderingContext::VERTEX_SHADER,
		include_str!("shaders/data.vert"),
	)?;

	let frag_shader = shader::compile_shader(
		&context,
		WebGl2RenderingContext::FRAGMENT_SHADER,
		include_str!("shaders/data.frag"),
	)?;

	let program = shader::link_program(&context, &vert_shader, &frag_shader)?;
	Ok(ShaderContext::new(&context, &program))
}
