use image::{DynamicImage, EncodableLayout, GenericImageView, GrayImage, Luma, Rgb, RgbImage};
use wasm_bindgen::JsValue;
use web_sys::WebGl2RenderingContext;

/// This feels like it probably duplicates something that can be done in the
/// image library already.
pub trait LoadableImageType {
	type ImageType: GenericImageView;

	// Combinations: https://registry.khronos.org/webgl/specs/latest/2.0/#TEXTURE_TYPES_FORMATS_FROM_DOM_ELEMENTS_TABLE
	fn texture_internal_format() -> u32;
	fn texture_format() -> u32;
	fn texture_type() -> u32;

	fn cast_to(dynamic: &DynamicImage) -> Option<&Self::ImageType>;
	fn copy_to(dynamic: &DynamicImage) -> Self::ImageType;
	fn raw(img: &Self::ImageType) -> &[u8];

	fn name() -> String;
}

impl LoadableImageType for Luma<u8> {
	type ImageType = GrayImage;

	fn texture_internal_format() -> u32 { WebGl2RenderingContext::LUMINANCE }

	fn texture_format() -> u32 { WebGl2RenderingContext::LUMINANCE }

	fn texture_type() -> u32 { WebGl2RenderingContext::UNSIGNED_BYTE }

	fn cast_to(dynamic: &DynamicImage) -> Option<&Self::ImageType> { dynamic.as_luma8() }

	fn copy_to(dynamic: &DynamicImage) -> Self::ImageType { dynamic.to_luma8() }

	fn raw(img: &Self::ImageType) -> &[u8] { img.as_bytes() }

	fn name() -> String { "Luma8".to_owned() }
}

impl LoadableImageType for Rgb<u8> {
	type ImageType = RgbImage;

	fn texture_internal_format() -> u32 { WebGl2RenderingContext::RGB }

	fn texture_format() -> u32 { WebGl2RenderingContext::RGB }

	fn texture_type() -> u32 { WebGl2RenderingContext::UNSIGNED_BYTE }

	fn cast_to(dynamic: &DynamicImage) -> Option<&Self::ImageType> { dynamic.as_rgb8() }

	fn copy_to(dynamic: &DynamicImage) -> Self::ImageType { dynamic.to_rgb8() }

	fn raw(img: &Self::ImageType) -> &[u8] { img.as_bytes() }

	fn name() -> String { "Rgb8".to_owned() }
}

pub fn load_into_texture<T: LoadableImageType>(
	context: WebGl2RenderingContext,
	png_bytes: &[u8],
	texture_number: u32,
) -> Result<(), JsValue> {
	let decoder = png::Decoder::new(png_bytes);
	let mut reader = decoder.read_info().map_err(|s| s.to_string())?;
	let mut buf = vec![0; reader.output_buffer_size()];

	let info = reader.next_frame(&mut buf).map_err(|s| s.to_string())?;
	let bytes = &buf[..info.buffer_size()];

	let dimensions = (info.width, info.height);

	// TODO: Probably slower, but worth profiling:
	// let dyn_img = image::load_from_memory_with_format(png_bytes,
	// ImageFormat::Png)     .map_err(|e| e.to_string())?;
	// let name = T::name();
	// let concrete_image = T::cast_to(&dyn_img)
	//     .expect(format!("Image was not stored with type {name}").as_str());
	// .ok_or(format!("Image was not stored with type {name}"));
	// let dimensions = concrete_image.dimensions();

	let texture = context.create_texture().ok_or("no texture")?;

	context.active_texture(texture_number);
	context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));

	context.tex_parameteri(
		WebGl2RenderingContext::TEXTURE_2D,
		WebGl2RenderingContext::TEXTURE_MIN_FILTER,
		WebGl2RenderingContext::LINEAR as i32,
	);
	context.tex_parameteri(
		WebGl2RenderingContext::TEXTURE_2D,
		WebGl2RenderingContext::TEXTURE_MAG_FILTER,
		WebGl2RenderingContext::LINEAR as i32,
	);

	context.tex_parameteri(
		WebGl2RenderingContext::TEXTURE_2D,
		WebGl2RenderingContext::TEXTURE_WRAP_S,
		WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
	);
	context.tex_parameteri(
		WebGl2RenderingContext::TEXTURE_2D,
		WebGl2RenderingContext::TEXTURE_WRAP_T,
		WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
	);

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
		Some(bytes),
	)?;

	Ok(())
}
