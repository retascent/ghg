use image::{GenericImageView, ImageFormat};
use wasm_bindgen::JsValue;
use web_sys::{WebGl2RenderingContext};

#[allow(unused_imports)]
use crate::utils::prelude::*;

pub fn load_planet_terrain(context: WebGl2RenderingContext) -> Result<(), JsValue> {

    // TODO: I want dynamic loading, not compile-time

    // Heightmap from https://visibleearth.nasa.gov/images/73934/topography
    let pic = include_bytes!("../../www/images/earth_height_10800x5400.png");
    let img = image::load_from_memory_with_format(pic, ImageFormat::Png)
        .map_err(|e| e.to_string())?;

    let dimensions = img.dimensions();

    let texture = context.create_texture().ok_or("no texture")?;

    context.active_texture(WebGl2RenderingContext::TEXTURE0);
    context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));

    context.tex_parameteri(WebGl2RenderingContext::TEXTURE_2D,
                           WebGl2RenderingContext::TEXTURE_MIN_FILTER,
                           WebGl2RenderingContext::LINEAR as i32);
    context.tex_parameteri(WebGl2RenderingContext::TEXTURE_2D,
                           WebGl2RenderingContext::TEXTURE_MAG_FILTER,
                           WebGl2RenderingContext::LINEAR as i32);

    context.tex_parameteri(WebGl2RenderingContext::TEXTURE_2D,
                           WebGl2RenderingContext::TEXTURE_WRAP_S,
                           WebGl2RenderingContext::CLAMP_TO_EDGE as i32);
    context.tex_parameteri(WebGl2RenderingContext::TEXTURE_2D,
                           WebGl2RenderingContext::TEXTURE_WRAP_T,
                           WebGl2RenderingContext::CLAMP_TO_EDGE as i32);

    // Lol. This should just be a builder.
    context.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
        WebGl2RenderingContext::TEXTURE_2D,
        0,
        WebGl2RenderingContext::RGB as i32,
        dimensions.0 as i32,
        dimensions.1 as i32,
        0,
        WebGl2RenderingContext::RGB,
        WebGl2RenderingContext::UNSIGNED_BYTE,
        Some(&img.into_rgb8().into_vec()))?;

    Ok(())
}

