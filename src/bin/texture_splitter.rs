#![feature(associated_type_defaults)]

extern crate core;

use std::fs;
use std::ops::Deref;
use std::path::Path;
use image::{ColorType, DynamicImage, EncodableLayout, GenericImageView, GrayImage, ImageBuffer, Luma, Pixel, Rgb};
use image::imageops::{FilterType, resize};
use image::io::Reader;

// TODO: Consolidate with render_core::image::LoadableImageType? At least some shared logic
trait ColorMapping: Pixel {
    fn color_type() -> ColorType;
    fn dynamic_to_specific(d: DynamicImage) -> ImageBuffer<Self, Vec<Self::Subpixel>>;
    fn force_bytes(b: &ImageBuffer<Self, Vec<Self::Subpixel>>) -> &[u8]; // TODO: Hack
}

impl ColorMapping for Rgb<u8> {
    fn color_type() -> ColorType {
        ColorType::Rgb8
    }

    fn dynamic_to_specific(d: DynamicImage) -> ImageBuffer<Self, Vec<Self::Subpixel>> {
        d.to_rgb8()
    }

    fn force_bytes(b: &ImageBuffer<Self, Vec<Self::Subpixel>>) -> &[u8] {
        b.as_bytes()
    }
}

impl ColorMapping for Luma<u8> {
    fn color_type() -> ColorType {
        ColorType::L8
    }

    fn dynamic_to_specific(d: DynamicImage) -> ImageBuffer<Self, Vec<Self::Subpixel>> {
        d.to_luma8()
    }

    fn force_bytes(b: &ImageBuffer<Self, Vec<Self::Subpixel>>) -> &[u8] {
        b.as_bytes()
    }
}

fn read_image(path: &Path) -> DynamicImage {
    let mut reader = Reader::open(path)
        .expect("Failed to open image.")
        .with_guessed_format()
        .expect("Failed to guess format");
    reader.no_limits();

    reader.decode().expect("Failed to decode image.")
}

fn create_original_terrain(image_root: &Path) -> GrayImage {
    let lod_root = image_root.join("earth_height/0");
    let topo_path = lod_root.join("topography-original.png");
    let bathy_path = lod_root.join("bathymetry-original.png");

    const U8_HALF: u8 = 128u8;

    let output_dimensions: (u32, u32);
    let mut output_buffer: Vec<u8>;

    { // BATHY
        let bathy_image = read_image(&bathy_path);
        println!("Bathymetry color type: {:?}", bathy_image.color());
        let bathy_gray = bathy_image.to_luma8();

        let (bathy_width, bathy_height) = bathy_image.dimensions();

        output_dimensions = (bathy_width, bathy_height);
        output_buffer = Vec::with_capacity(bathy_width as usize * bathy_height as usize);

        bathy_gray.as_bytes().iter().enumerate()
            .for_each(|(_index, &pixel)| {
                let rescaled = pixel / 2;
                output_buffer.push(rescaled);
            });
    }

    { // TOPO
        let topo_image = read_image(&topo_path);
        let topo_gray = topo_image.as_luma8().expect("Failed to read topography as Luma8");

        let (topo_width, topo_height) = topo_image.dimensions();

        if output_dimensions.0 != topo_width || output_dimensions.1 != topo_height {
            let output_width = output_dimensions.0;
            let output_height = output_dimensions.1;
            panic!("Mismatched input sizes: Topography is {output_width}x{output_height}, Bathymetry is {topo_width}x{topo_height}.");
        }

        topo_gray.as_bytes().iter().enumerate()
            .for_each(|(index, &pixel)| {
                if pixel != 0 {
                    let rescaled = pixel / 2 + U8_HALF;
                    output_buffer[index] = rescaled;
                }
            });
    }

    let output_image = GrayImage::from_raw(output_dimensions.0, output_dimensions.1, output_buffer)
        .expect("Failed to create merged image!");

    output_image.save(lod_root.join("full.png")).expect("Failed to save merged image!");

    output_image
}

fn create_downscaled_originals<P: 'static + Pixel<Subpixel = u8> + ColorMapping>(
            image_root: &Path,
            internal_destination: &str,
            max_level: u32,
            original: &ImageBuffer<P, Vec<P::Subpixel>>,
        ) where DynamicImage: From<ImageBuffer<P, Vec<<P as Pixel>::Subpixel>>> {
    const FILTER_TYPE: FilterType = FilterType::Gaussian;

    let (original_width, original_height) = original.dimensions();

    for level in 1..=max_level {
        let downscale = 1u32 << level;
        println!("Creating level {level} (1/{downscale} original size)");

        let image = <P as ColorMapping>::dynamic_to_specific(DynamicImage::from(
            resize(original,
                   original_width / downscale,
                   original_height / downscale,
                   FILTER_TYPE)
        ));

        let (width, height) = image.dimensions();

        let level_path = image_root.join(format!("{internal_destination}/{level}"));
        fs::create_dir_all(level_path.clone())
            .expect(format!("Failed to create path {:?}", level_path).as_str());

        image::save_buffer(
            level_path.join("full.png"),
            image.as_bytes(),
            width,
            height,
            <P as ColorMapping>::color_type(),
        ).expect(format!("Failed to write level {level}").as_str());

        println!("Level {level} succeeded: {width} x {height}");
    }
}

fn create_subimages<Map: ColorMapping + 'static>(
            image_root: &Path,
            internal_destination: &str,
            max_level: u32,
            num_columns: u32,
            num_rows: u32
        ) {
    for level in 0..=max_level {
        println!("Creating {}x{} subimages for level {}", num_columns, num_rows, level);

        let level_path = image_root.join(format!("{internal_destination}/{level}"));

        let full_image = Map::dynamic_to_specific(read_image(&level_path.join("full.png")));
        let (full_width, full_height) = full_image.dimensions();

        let subimages_path = level_path.join(format!("{num_columns}x{num_rows}"));
        fs::create_dir_all(subimages_path.clone())
            .expect(format!("Failed to create path {:?}", subimages_path).as_str());

        if full_width % num_columns != 0 || full_height % num_columns != 0 {
            println!(
                "WARNING: Imaged does not divide evenly. {full_width} x {full_height}, divided into {num_columns} columns and {num_rows} rows."
            );
        }

        let subimage_width = full_width / num_columns;
        let subimage_height = full_height / num_rows;

        for column in 0..num_columns {
            for row in 0..num_rows {
                println!("  Creating subimage {column}.{row}");

                let subimage = full_image.view(
                    column * subimage_width,
                    row * subimage_height,
                    subimage_width,
                    subimage_height,
                ).to_image();

                image::save_buffer(
                    subimages_path.join(format!("{column}.{row}.png")),
                    // subimage.as_bytes(),
                    Map::force_bytes(&subimage),
                    subimage_width,
                    subimage_height,
                    Map::color_type(),
                ).expect(format!("  Failed to write level {level} subimage at {column}.{row}").as_str());
            }
        }

        println!("Level {level} subimages succeeded.");
    }
}

enum WhichToGenerate {
    Color,
    Height,
}

fn main() {
    // Expects to be run with CWD in the project root
    let image_root = Path::new("./www/images");

    const GENERATE: WhichToGenerate = WhichToGenerate::Height;
    match GENERATE {
        WhichToGenerate::Color => {
            let color_image = read_image(&image_root.join("earth_color/0/full.png"));
            let original = color_image.as_rgb8().expect("Wrong color type!");

            let (width_0, height_0) = original.dimensions();
            println!("Loaded image: {} x {}", width_0, height_0);

            create_downscaled_originals(image_root, "earth_color", 2, original);
            create_subimages::<Rgb<u8>>(image_root, "earth_color", 2, 4, 2);
        }
        WhichToGenerate::Height => {
            let original_dynamic = image::open(image_root.join("earth_height/0/full.png"))
                .map_err(|e| e.to_string()).expect("Failed to load image!");

            let original = original_dynamic
                .as_luma8()
                .expect("Failed to get Luma8 Image!");

            let (width_0, height_0) = original.dimensions();
            println!("Loaded image: {} x {}", width_0, height_0);

            create_downscaled_originals(image_root, "earth_height", 2, original);
            create_subimages::<Luma<u8>>(image_root, "earth_height", 2, 4, 2);
        }
    }
}
