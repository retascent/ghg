use std::path::Path;
use image::Rgb;
use serde_json::from_slice;
use wasm_bindgen::JsValue;
use web_sys::WebGl2RenderingContext;
use crate::application::shaders::ShaderContext;
use ghg_data_core::metadata::Metadata;
use crate::request_data::fetch_bytes;
use crate::render_core::image::load_into_texture;
use crate::render_core::uniform;
use crate::render_core::uniform::SmartUniform;

pub struct DataMapping {
    pub metadata: Metadata,
    pub texture_uniform: SmartUniform<i32>,
    pub min_uniforms: SmartUniform<nglm::Vec3>,
    pub max_uniforms: SmartUniform<nglm::Vec3>,
}

pub async fn load_temp_data(shader_context: ShaderContext) -> Result<DataMapping, JsValue> {
    let temp_root = Path::new("images/earth_temp");

    let august_temp_image = temp_root.join("2021-1980.08.png");
    let august_temp_metadata = august_temp_image.with_extension("metadata");

    let texture = fetch_bytes(august_temp_image.to_str().unwrap()).await?;

    let metadata_bytes = fetch_bytes(august_temp_metadata.to_str().unwrap()).await?;
    let metadata: Metadata = from_slice(&metadata_bytes).map_err(|e| e.to_string())?;

    let texture_index = 2;

    shader_context.use_shader(); // TODO: Probably not efficient
    load_into_texture::<Rgb<u8>>(shader_context.context.clone(), &texture,
                                 WebGl2RenderingContext::TEXTURE0 + texture_index as u32)?;

    let (min_vals, max_vals): (nglm::Vec3, nglm::Vec3) = metadata.clone().try_into()?; // TODO: Wasteful clone

    Ok(DataMapping {
        metadata: metadata.clone(),
        texture_uniform: uniform::init_smart_i32("s_dataMap", &shader_context, texture_index),
        min_uniforms: uniform::init_smart_vec3("u_dataMinValues", &shader_context, min_vals),
        max_uniforms: uniform::init_smart_vec3("u_dataMaxValues", &shader_context, max_vals),
    })
}
