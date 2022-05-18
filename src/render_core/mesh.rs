use std::convert::TryInto;
use wasm_bindgen::JsValue;
use web_sys::{WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlVertexArrayObject};

#[derive(Clone, Debug)]
pub struct VertexAttribute {
    name: String,
    size: usize,
    offset: usize,
}

impl VertexAttribute {
    pub fn new(name: String, size: usize, offset: usize) -> Self {
        Self {
            name,
            size,
            offset,
        }
    }
}

pub trait ToMesh {
    type Vertex;

    fn get_attributes(&self) -> Vec<VertexAttribute>;
    fn get_flat_vertex_buffer(&self) -> &[f32];
    fn get_flat_index_buffer(&self) -> &[u32];
    fn get_bounding_box(&self) -> nglm::Mat3x2;
}

#[derive(Copy, Clone, Debug)]
struct VertexForDisplay {
    location: u32,
    size: i32,
    offset: i32,
}

pub struct DrawBuffers {
    pub vertex_buffer: WebGlBuffer,
    pub vertex_array_object: WebGlVertexArrayObject,
    pub index_buffer: WebGlBuffer,

    pub num_vertices: u32,
    pub num_indices: u32,
}

pub fn add_static_mesh<T: ToMesh>(
            context: &WebGl2RenderingContext,
            program: &WebGlProgram,
            mesh: &T
        ) ->  Result<DrawBuffers, JsValue> {
    let vertex_attribute_tags = mesh.get_attributes();

    let vertex_attributes = vertex_attribute_tags.iter()
        .map(|a|  {
            let a_name: &str = &a.name[..];
            VertexForDisplay{
                location: context.get_attrib_location(&program, a_name) as u32,
                size: a.size as i32,
                offset: a.offset as i32,
            }});

    let vertex_buffer = context.create_buffer().ok_or("Failed to create vertex buffer")?;
    context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));

    let vertices = mesh.get_flat_vertex_buffer();
    let num_vertices: u32 = vertices.len() as u32;
    unsafe {
        let vert_array_buffer_view = js_sys::Float32Array::view(vertices);

        context.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &vert_array_buffer_view,
            WebGl2RenderingContext::STATIC_DRAW,
        );
    }

    let vertex_array_object = context
        .create_vertex_array()
        .ok_or("Could not create vertex array object")?;
    context.bind_vertex_array(Some(&vertex_array_object));

    vertex_attributes
        .for_each(|a| {
            context.enable_vertex_attrib_array(a.location);
            context.vertex_attrib_pointer_with_i32(a.location, a.size, WebGl2RenderingContext::FLOAT, false,
                                                   std::mem::size_of::<T::Vertex>() as i32, a.offset
            );
        });

    let index_buffer = context.create_buffer().ok_or("Failed to create index buffer")?;
    context.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));

    let indices = mesh.get_flat_index_buffer();
    let num_indices = indices.len() as u32;
    unsafe {
        let index_array_buffer_view = js_sys::Uint32Array::view(indices);

        context.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            &index_array_buffer_view,
            WebGl2RenderingContext::STATIC_DRAW,
        );
    }

    Ok(DrawBuffers {
        vertex_buffer,
        vertex_array_object,
        index_buffer,
        num_vertices,
        num_indices,
    })
}

#[allow(dead_code)]
pub enum DrawMode {
    Surface,
    Wireframe,
}

pub fn draw_buffers(context: &WebGl2RenderingContext, buffers: &Vec<DrawBuffers>, draw_mode: DrawMode) {
    context.clear_color(0.45, 0.67, 0.93, 1.0);
    context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT);

    buffers.iter()
        .for_each(|b| {
            context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&b.vertex_buffer));
            context.bind_vertex_array(Some(&b.vertex_array_object));

            let mode: u32 = match draw_mode {
                DrawMode::Surface => WebGl2RenderingContext::TRIANGLES,
                DrawMode::Wireframe => WebGl2RenderingContext::LINE_STRIP,
            };
            context.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&b.index_buffer));
            context.draw_elements_with_i32(mode,
                                           b.num_indices.try_into().unwrap(),
                                           WebGl2RenderingContext::UNSIGNED_INT,
                                           0);
        });
}
