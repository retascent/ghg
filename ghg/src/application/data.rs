use std::cell::Cell;
use std::future::join;
use std::path::Path;
use std::rc::Rc;

use ghg_data_core::metadata::Metadata;
use image::Rgba;
use serde_json::from_slice;
use wasm_bindgen::JsValue;
use web_sys::WebGl2RenderingContext;

use crate::application::shaders::ShaderContext;
use crate::render_core::animation_params::AnimationParams;
use crate::render_core::frame_sequencer::FrameGate;
use crate::render_core::image::load_into_texture_with_filters;
use crate::render_core::uniform;
use crate::request_data::fetch_bytes;
use crate::utils::prelude::*;

// struct DataMapping {
// 	pub metadata: Metadata,
// 	pub texture_uniform: SmartUniform<i32>,
// 	pub min_uniforms: SmartUniform<nglm::Vec4>,
// 	pub max_uniforms: SmartUniform<nglm::Vec4>,
// }

const MONTH_NAMES: [&str; 12] = [
	"January",
	"February",
	"March",
	"April",
	"May",
	"June",
	"July",
	"August",
	"September",
	"October",
	"November",
	"December",
];

async fn load_temp_data(
	shader_context: ShaderContext,
	file_stem: &str,
	texture_index: i32,
) -> Result<Metadata, JsValue> {
	let temp_root = Path::new("images/earth_temp");

	let summer_temp_image = temp_root.join((file_stem.to_owned() + ".png").as_str());
	let summer_temp_metadata = summer_temp_image.with_extension("metadata");

	let texture = fetch_bytes(summer_temp_image.to_str().unwrap()).await?;

	let metadata_bytes = fetch_bytes(summer_temp_metadata.to_str().unwrap()).await?;
	let metadata: Metadata = from_slice(&metadata_bytes).map_err(|e| e.to_string())?;

	shader_context.use_shader();
	load_into_texture_with_filters::<Rgba<u8>>(
		shader_context.context.clone(),
		&texture,
		WebGl2RenderingContext::TEXTURE0 + texture_index as u32,
		WebGl2RenderingContext::LINEAR,
		WebGl2RenderingContext::NEAREST,
	)?;

	Ok(metadata)
}

pub async fn handle_data(
	gate: FrameGate<AnimationParams>,
	shader_context: ShaderContext,
	current_month: Rc<Cell<usize>>,
) {
	let first_map_index: i32 = 2;

	let load_all_results = join!(
		load_temp_data(shader_context.clone(), "2021.01.04", first_map_index + 0),
		load_temp_data(shader_context.clone(), "2021.05.08", first_map_index + 1),
		load_temp_data(shader_context.clone(), "2021.09.12", first_map_index + 2),
	)
	.await;

	if !load_all_results.0.is_ok() || !load_all_results.1.is_ok() || !load_all_results.2.is_ok() {
		ghg_error!("Failed to load some temperature data: {:?}", load_all_results)
	}

	let mins_and_maxes: [(nglm::Vec4, nglm::Vec4); 3] = [
		load_all_results.0.ok().unwrap().clone().try_into().expect("Failed to convert metadata"),
		load_all_results.1.ok().unwrap().clone().try_into().expect("Failed to convert metadata"),
		load_all_results.2.ok().unwrap().clone().try_into().expect("Failed to convert metadata"),
	];

	const NUM_CHANNELS: i32 = 4;

	let (mins, maxes): (Vec<nglm::Vec4>, Vec<nglm::Vec4>) = mins_and_maxes.into_iter().unzip();

	let min_mat = nglm::Mat4x3::from_columns(&mins);
	let max_mat = nglm::Mat4x3::from_columns(&maxes);

	let _min_uniforms = uniform::init_smart_mat4x3("u_dataMinValues", &shader_context, min_mat);
	let _max_uniforms = uniform::init_smart_mat4x3("u_dataMaxValues", &shader_context, max_mat);

	let mut texture_uniform = uniform::new_smart_i32("s_dataMap", &shader_context);
	let mut data_month_uniform = uniform::new_smart_i32("u_dataMonth", &shader_context);

	loop {
		let _params = (&gate).await;

		let current_month = current_month.get() as i32;
		let current_map_index = current_month / NUM_CHANNELS;

		ghg_log!(
			"Month: {}, map index: {}",
			MONTH_NAMES[current_month as usize],
			first_map_index + current_map_index
		);

		texture_uniform.smart_write(first_map_index + current_map_index);
		data_month_uniform.smart_write(current_month);
	}
}
