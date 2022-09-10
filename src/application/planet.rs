use image::{GenericImageView, ImageFormat, Luma, Rgb};
use wasm_bindgen::JsValue;
use web_sys::{WebGl2RenderingContext};
use crate::data_core::request_data::fetch_bytes;
use crate::render_core::image::LoadableImageType;

#[allow(unused_imports)]
use crate::utils::prelude::*;

fn load_into_texture<T: LoadableImageType>(
    context: WebGl2RenderingContext,
    png_bytes: &[u8],
    texture_number: u32
) -> Result<(), JsValue> {
    let decoder = png::Decoder::new(png_bytes);
    let mut reader = decoder.read_info().map_err(|s| s.to_string())?;
    let mut buf = vec![0; reader.output_buffer_size()];

    let info = reader.next_frame(&mut buf).map_err(|s| s.to_string())?;
    let bytes = &buf[..info.buffer_size()];

    let dimensions = (info.width, info.height);

    // TODO: Probably slower, but worth profiling:
    // let dyn_img = image::load_from_memory_with_format(png_bytes, ImageFormat::Png)
    //     .map_err(|e| e.to_string())?;
    // let name = T::name();
    // let concrete_image = T::cast_to(&dyn_img)
    //     .expect(format!("Image was not stored with type {name}").as_str());
        // .ok_or(format!("Image was not stored with type {name}"));
    // let dimensions = concrete_image.dimensions();

    let texture = context.create_texture().ok_or("no texture")?;

    context.active_texture(texture_number);
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
        T::texture_internal_format() as i32,
        dimensions.0 as i32,
        dimensions.1 as i32,
        0,
        T::texture_format(),
        T::texture_type(),
        Some(bytes))?;

    Ok(())
}

pub async fn load_planet_terrain(context: WebGl2RenderingContext) -> Result<(), JsValue> {
    let texture = fetch_bytes("images/earth_height/2/full.png").await?;
    load_into_texture::<Luma<u8>>(context, &texture, WebGl2RenderingContext::TEXTURE0)
}

pub async fn load_planet_color(context: WebGl2RenderingContext) -> Result<(), JsValue> {
    let texture = fetch_bytes("images/earth_color/2/full.png").await?;
    load_into_texture::<Rgb<u8>>(context, &texture, WebGl2RenderingContext::TEXTURE1)?;
    Ok(())
}
