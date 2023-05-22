use std::convert::TryInto;

use wasm_bindgen::JsValue;
use web_sys::{WebGl2RenderingContext, WebGlBuffer, WebGlVertexArrayObject};

use crate::application::shaders::ShaderContext;
use crate::application::vertex::BasicMesh;
use crate::render_core::camera::Camera;
#[allow(unused_imports)]
use crate::utils::prelude::*;

#[derive(Clone, Debug)]
pub struct VertexAttribute {
	name: String,
	size: usize,
	offset: usize,
}

impl VertexAttribute {
	pub fn new(name: &str, size: usize, offset: usize) -> Self {
		Self { name: name.to_owned(), size, offset }
	}
}

pub trait ToMesh {
	type Vertex;

	fn get_attributes(&self) -> Vec<VertexAttribute>;
	fn get_flat_vertex_buffer(&self) -> &[f32];
	fn get_flat_index_buffer(&self) -> &[u32];

	fn get_bounding_box(&self) -> Option<nglm::Mat3x2>;
	fn get_center(&self) -> Option<nglm::Vec3>;

	fn is_visible(&self, camera: &Camera) -> bool;
}

#[derive(Copy, Clone, Debug)]
struct VertexForDisplay {
	location: Option<u32>,
	size: i32,
	offset: i32,
}

pub struct DrawBuffers {
	pub vertex_buffer: WebGlBuffer,
	pub vertex_array_object: WebGlVertexArrayObject,
	pub index_buffer: WebGlBuffer,

	#[allow(dead_code)]
	num_vertices: u32,
	num_indices: u32,
}

#[allow(dead_code)]
pub enum MeshMode {
	Static,
	Dynamic,
}

pub fn add_mesh<T: ToMesh>(
	shader_context: &ShaderContext,
	mesh: &T,
	mode: MeshMode,
) -> Result<DrawBuffers, JsValue> {
	let vertex_attribute_tags = mesh.get_attributes();

	let vertex_attributes = vertex_attribute_tags.iter().map(|a| {
		let a_name: &str = &a.name[..];
		let location = shader_context.context.get_attrib_location(&shader_context.program, a_name);
		VertexForDisplay {
			location: if location != -1 { Some(location as u32) } else { None },
			size: a.size as i32,
			offset: a.offset as i32,
		}
	});

	let vertex_buffer =
		shader_context.context.create_buffer().ok_or("Failed to create vertex buffer")?;
	shader_context.context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));

	let draw_mode = match mode {
		MeshMode::Static => WebGl2RenderingContext::STATIC_DRAW,
		MeshMode::Dynamic => WebGl2RenderingContext::DYNAMIC_DRAW,
	};

	let vertices = mesh.get_flat_vertex_buffer();
	let num_vertices: u32 = vertices.len() as u32;
	unsafe {
		let vert_array_buffer_view = js_sys::Float32Array::view(vertices);

		shader_context.context.buffer_data_with_array_buffer_view(
			WebGl2RenderingContext::ARRAY_BUFFER,
			&vert_array_buffer_view,
			draw_mode,
		);
	}

	let vertex_array_object = shader_context
		.context
		.create_vertex_array()
		.ok_or("Could not create vertex array object")?;
	shader_context.context.bind_vertex_array(Some(&vertex_array_object));

	vertex_attributes.for_each(|a| {
		if a.location.is_some() {
			shader_context.context.enable_vertex_attrib_array(a.location.unwrap());
			shader_context.context.vertex_attrib_pointer_with_i32(
				a.location.unwrap(),
				a.size,
				WebGl2RenderingContext::FLOAT,
				false,
				std::mem::size_of::<T::Vertex>() as i32,
				a.offset,
			);
		}
	});

	let index_buffer =
		shader_context.context.create_buffer().ok_or("Failed to create index buffer")?;
	shader_context
		.context
		.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));

	let indices = mesh.get_flat_index_buffer();
	let num_indices = indices.len() as u32;
	unsafe {
		let index_array_buffer_view = js_sys::Uint32Array::view(indices);

		shader_context.context.buffer_data_with_array_buffer_view(
			WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
			&index_array_buffer_view,
			draw_mode,
		);
	}

	Ok(DrawBuffers { vertex_buffer, vertex_array_object, index_buffer, num_vertices, num_indices })
}

#[allow(dead_code)]
pub enum DrawMode {
	Surface,
	Wireframe,
	Points,
}

pub fn clear_frame(context: &WebGl2RenderingContext) {
	context.clear_color(0.45, 0.67, 0.93, 1.0);
	context
		.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT);
}

pub fn draw_meshes(
	context: &WebGl2RenderingContext,
	camera: &Camera,
	buffers: &Vec<(BasicMesh, DrawBuffers)>,
	draw_mode: DrawMode,
) {
	buffers.iter().for_each(|(m, b)| {
		if m.is_visible(camera) {
			context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&b.vertex_buffer));
			context.bind_vertex_array(Some(&b.vertex_array_object));

			let mode: u32 = match draw_mode {
				DrawMode::Surface => WebGl2RenderingContext::TRIANGLES,
				DrawMode::Wireframe => WebGl2RenderingContext::LINE_STRIP,
				DrawMode::Points => WebGl2RenderingContext::POINTS,
			};
			context
				.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&b.index_buffer));
			context.draw_elements_with_i32(
				mode,
				b.num_indices.try_into().unwrap(),
				WebGl2RenderingContext::UNSIGNED_INT,
				0,
			);
		}
	});
}

// pub fn draw_buffers(context: &WebGl2RenderingContext, buffers:
// &Vec<DrawBuffers>, draw_mode: DrawMode) {     buffers.iter()
//         .for_each(|b| {
//             context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER,
// Some(&b.vertex_buffer));
// context.bind_vertex_array(Some(&b.vertex_array_object));
//
//             let mode: u32 = match draw_mode {
//                 DrawMode::Surface => WebGl2RenderingContext::TRIANGLES,
//                 DrawMode::Wireframe => WebGl2RenderingContext::LINE_STRIP,
//             };
//             context.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
// Some(&b.index_buffer));             context.draw_elements_with_i32(mode,
//                                            b.num_indices.try_into().unwrap(),
//
// WebGl2RenderingContext::UNSIGNED_INT,
// 0);         });
// }
