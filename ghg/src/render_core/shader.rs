use std::ops::Index;

use clap::builder::Str;
use phf::{phf_map, Map};
use regex;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader};

/// Note: `shader_path` must be relative to this crate's `src` directory
pub fn preprocess_and_compile_shader(
	context: &WebGl2RenderingContext,
	shader_type: u32,
	shader_source: &str,
) -> Result<WebGlShader, String> {
	let preprocessed = preprocess_shader(shader_source);
	compile_shader(context, shader_type, preprocessed.as_str())
}

fn compile_shader(
	context: &WebGl2RenderingContext,
	shader_type: u32,
	shader_source: &str,
) -> Result<WebGlShader, String> {
	let shader = context
		.create_shader(shader_type)
		.ok_or_else(|| String::from("Unable to create shader object"))?;

	context.shader_source(&shader, shader_source);
	context.compile_shader(&shader);

	if context
		.get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
		.as_bool()
		.unwrap_or(false)
	{
		Ok(shader)
	} else {
		Err(context
			.get_shader_info_log(&shader)
			.unwrap_or_else(|| String::from("Unknown error creating shader")))
	}
}

pub fn link_program(
	context: &WebGl2RenderingContext,
	vert_shader: &WebGlShader,
	frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
	let program =
		context.create_program().ok_or_else(|| String::from("Unable to create shader object"))?;

	context.attach_shader(&program, vert_shader);
	context.attach_shader(&program, frag_shader);
	context.link_program(&program);

	if context
		.get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
		.as_bool()
		.unwrap_or(false)
	{
		Ok(program)
	} else {
		Err(context
			.get_program_info_log(&program)
			.unwrap_or_else(|| String::from("Unknown error creating program object")))
	}
}

const INCLUDE_STRING_MATCH: &str = r#"#include <([a-zA-Z0-9\.\-\_/]+)>"#;
const LINE_COMMENT_MATCH: &str = r#"//"#;

macro_rules! include_strs {
	($($relative_to_src:expr),+ $(,)?) => {
		phf_map! {
			$($relative_to_src => include_str!(concat!("../", $relative_to_src)),)+
		}
	};
}

const PREPROCESSABLE_SHADERS: Map<&str, &str> =
	include_strs!["application/shaders/pointmapping.glsl",];

fn load_shader(source_path: &str) -> &str {
	PREPROCESSABLE_SHADERS
		.get(source_path)
		.expect(format!("Shader {} was not listed for preprocessing", source_path).as_str())
}

fn preprocess_shader(shader_source: &str) -> String {
	let with_includes = fill_includes(shader_source);
	with_includes
}

fn fill_includes(shader_source: &str) -> String {
	let mut filled_source = shader_source.to_owned();
	let include_str_regex = regex::Regex::new(INCLUDE_STRING_MATCH).unwrap();
	loop {
		if let Some(include_line) = include_str_regex.find(filled_source.as_str()) {
			if !is_match_commented(include_line, filled_source.as_str()) {
				let file_name = {
					let captures = include_str_regex.captures(include_line.as_str()).expect(
						format!(
							"Parsing error for include {}-{}: {}",
							include_line.start(),
							include_line.end(),
							include_line.as_str()
						)
							.as_str(),
					);
					assert_eq!(captures.len(), 2);
					captures.index(1).to_owned()
				};
				println!("{include_line:?}, {file_name}");

				let include_file_contents = load_shader(file_name.as_str());
				let processed_include = preprocess_shader(include_file_contents);
				filled_source.replace_range(
					include_line.start()..include_line.end(),
					processed_include.as_str(),
				);
			}
		} else {
			break;
		}
	}

	filled_source
}

fn is_match_commented(regex_match: regex::Match, full_source: &str) -> bool {
	// TODO
	false
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn basic() {
		let result = preprocess_shader(include_str!("../application/shaders/data.vert"));
		println!("{}", result);
	}

	#[test]
	fn shader_map() {
		let shader_map = include_strs!("application/shaders/pointmapping.glsl");
		println!("{:?}", shader_map);
	}
}
