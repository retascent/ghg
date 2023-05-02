use image::{Luma, Rgb};
use wasm_bindgen::JsValue;
use web_sys::WebGl2RenderingContext;

use crate::render_core::image::load_into_texture;
use crate::request_data::fetch_bytes;
#[allow(unused_imports)]
use crate::utils::prelude::*;

pub async fn load_planet_terrain(context: WebGl2RenderingContext) -> Result<(), JsValue> {
	let texture = fetch_bytes("images/earth_height/2/full.png").await?;
	load_into_texture::<Luma<u8>>(context, &texture, WebGl2RenderingContext::TEXTURE0)
}

pub async fn load_planet_color(context: WebGl2RenderingContext) -> Result<(), JsValue> {
	let texture = fetch_bytes("images/earth_color/2/full.png").await?;
	load_into_texture::<Rgb<u8>>(context, &texture, WebGl2RenderingContext::TEXTURE1)?;
	Ok(())
}
