use image::{DynamicImage, EncodableLayout, GenericImageView, GrayImage, Luma, Rgb, RgbImage};
use web_sys::WebGl2RenderingContext;

/// This feels like it probably duplicates something that can be done in the image library already.
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

    fn texture_internal_format() -> u32 {
        WebGl2RenderingContext::LUMINANCE
    }

    fn texture_format() -> u32 {
        WebGl2RenderingContext::LUMINANCE
    }

    fn texture_type() -> u32 {
        WebGl2RenderingContext::UNSIGNED_BYTE
    }

    fn cast_to(dynamic: &DynamicImage) -> Option<&Self::ImageType> {
        dynamic.as_luma8()
    }

    fn copy_to(dynamic: &DynamicImage) -> Self::ImageType {
        dynamic.to_luma8()
    }

    fn raw(img: &Self::ImageType) -> &[u8] {
        img.as_bytes()
    }

    fn name() -> String {
        "Luma8".to_owned()
    }
}

impl LoadableImageType for Rgb<u8> {
    type ImageType = RgbImage;

    fn texture_internal_format() -> u32 {
        WebGl2RenderingContext::RGB
    }

    fn texture_format() -> u32 {
        WebGl2RenderingContext::RGB
    }

    fn texture_type() -> u32 {
        WebGl2RenderingContext::UNSIGNED_BYTE
    }

    fn cast_to(dynamic: &DynamicImage) -> Option<&Self::ImageType> {
        dynamic.as_rgb8()
    }

    fn copy_to(dynamic: &DynamicImage) -> Self::ImageType {
        dynamic.to_rgb8()
    }

    fn raw(img: &Self::ImageType) -> &[u8] {
        img.as_bytes()
    }

    fn name() -> String {
        "Rgb8".to_owned()
    }
}
