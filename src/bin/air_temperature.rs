#![feature(trait_alias)]

use std::ffi::OsStr;
use std::fs::File;
use std::ops::Sub;
use std::path::{Path, PathBuf};
use image::{GrayAlphaImage, GrayImage, ImageBuffer, Luma, LumaA, Pixel, Rgb, RgbImage};
use itertools::izip;
use netcdf;
use netcdf::{Numeric, Variable};
use ghg::data_core::metadata::{ChannelMetadata, Metadata};

use std::io::prelude::*;

/// If you have a problem finding HDF5 or netCDF, make sure to run this with these flags:
///     --all-features --features hdf5-sys/static,netcdf/static
/// This will ensure the HDF5 and netCDF projects are built statically, instead of looking for them to be installed.
/// The `--all-features` flag is always needed, but the latter flags are only needed if the build can't locate the installs.
/// This is a loader for the 2m air temperature data source.
/// More information about the data:
///  - [doi:10.5067/5ESKGQTZG7FO](https://doi.org/10.5067/5ESKGQTZG7FO)
///  - [Data format specification](https://gmao.gsfc.nasa.gov/pubs/docs/Bosilovich785.pdf)
///  - [Data information](https://cmr.earthdata.nasa.gov/search/concepts/C1276812823-GES_DISC.html)
///  - [1980 data download](https://search.earthdata.nasa.gov/search/granules?p=C1276812823-GES_DISC&pg[0][v]=f&pg[0][gsk]=-start_date&q=C1276812823-GES_DISC&qt=1980-01-01T00%3A00%3A00.000Z%2C1980-12-31T23%3A59%3A59.999Z&tl=1659554690.391!3!!)
///  - [2021 data download](https://search.earthdata.nasa.gov/search/granules?p=C1276812823-GES_DISC&pg[0][v]=f&pg[0][gsk]=-start_date&q=C1276812823-GES_DISC&qt=2021-01-01T00%3A00%3A00.000Z%2C2021-12-31T23%3A59%3A59.999Z&tl=1659554690.391!3!!)
///
/// All data should be downloaded into /raw_data/merra2_1980_2021 within this repo.

// Instantaneous Two-Dimensional Collections: instM_1d_asm_Nx (M2IMNXASM): Single-Level Diagnostics
const TIME_DIMENSION: usize = 0;
const LATITUDE_DIMENSION: usize = 1;
const LONGITUDE_DIMENSION: usize = 2;

const LONGITUDE_POINTS: usize = 576;
const LATITUDE_POINTS: usize = 361;

trait DataType = Copy + Clone + Default + Numeric + PartialOrd + Sub;

macro_rules! count {
    () => (0usize);
    ( $x:tt $($xs:tt)* ) => (1usize + count!($($xs)*));
}

macro_rules! save_channels {
    ($output_name:expr, $($channels:expr),+) => {
        const NUM_CHANNELS: usize = count!($($channels)+);
        let output_channels: [Data2dStatistics<f64>; NUM_CHANNELS] = [$($channels),+];
        output_channels.to_image()
            .save($output_name.clone())
            .expect("Failed to save data as image!");

        let metadata_name = $output_name.with_extension("metadata");
        let mut metadata_file = File::create(metadata_name)?;
        let metadata = serde_json::to_string(&output_channels.to_metadata()).expect("Failed to serialize metadata");
        write!(metadata_file, "{}", metadata)?;

        // WIP trying to use metadata instead of a separate file.
        // let output_file = File::create(difference_output).unwrap();
        // let ref mut buf_writer = BufWriter::new(output_file);
        // let mut encoder = Encoder::new(
        //     buf_writer, LONGITUDE_POINTS as u32, LATITUDE_POINTS as u32
        // );
        // encoder.set_color(png::ColorType::Grayscale);
        // encoder.set_depth(png::BitDepth::Eight);
        // encoder.add_text_chunk("Description".to_string(),
        //     format!("min={}, max={}", differences.min.unwrap(), differences.max.unwrap()),
        // ).unwrap();
        // let mut writer = encoder.write_header().expect("Failed to write header!");
        // writer.write_image_data(&output_buffer).expect("Failed to write image data!");
    };
}

fn main() -> std::io::Result<()> {
    let output_root = Path::new("www/images/earth_temp");
    let data_source = Path::new("raw_data/merra2_1980_2021");
    let data_paths = find_data_files(data_source, OsStr::new("nc4"));

    for month in 0..12 {
        println!("Month {month}");
        let files = get_data_from_month(&data_paths, month);
        assert_eq!(files.len(), 2);
        println!("Files: {files:?}");

        let (data_1980, data_2021, differences) = {
            let variables = ["T2M"];
            let mut data_1980 = read_data(&files[0], &variables);
            let mut data_2021 = read_data(&files[1], &variables);

            assert_eq!(data_1980.len(), 1);
            assert_eq!(data_2021.len(), 1);

            let differences = &data_2021[0] - &data_1980[0];
            (data_1980.remove(0), data_2021.remove(0), differences)
        };

        let output_name = output_root.join(format!("2021-1980.{:0>2}.png", month + 1));
        save_channels!(output_name, data_1980, data_2021, differences);
    }

    Ok(())
}

fn get_data_from_month(all_paths: &Vec<PathBuf>, month: i32) -> Vec<PathBuf> {
    all_paths.iter()
        .filter(|p| {
            let filename = p.file_stem().unwrap();
            let name_str = filename.to_str().unwrap();
            name_str.ends_with(format!("{:0>2}", month + 1).as_str())
        }).cloned().collect()
}

fn find_data_files(root_path: &Path, extension: &OsStr) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = Vec::new();

    for filename in root_path.read_dir().unwrap() {
        let path = filename.unwrap().path();
        if let Some(current_extension) = path.extension() {
            if current_extension == extension {
                paths.push(path);
            }
        }
    }

    paths.sort();
    paths
}

fn read_data<T: DataType>(path: &PathBuf, variable_names: &[&str]) -> Vec<Data2dStatistics<T>> {
    println!("Reading data from {:?}", path.file_name().unwrap());
    let mut all_data = Vec::new();

    if let Ok(contents) = netcdf::open(path) {
        for &name in variable_names {
            if let Some(v) = contents.variable(name) {
                all_data.push(read_variable(&v));
            }
            else {
                panic!("Unknown variable {name}");
            }
        }
        assert_eq!(all_data.len(), variable_names.len());
    }

    all_data
}

fn read_variable<T: DataType>(v: &Variable) -> Data2dStatistics<T> {
    println!("Reading variable: {:?} (length = {})", v.name(), v.len());

    match v.name().as_str() {
        "T2M" => Data2dStatistics::<T>::from_variable(v), // TODO: This mapping is pretty bad
        _ => panic!("Unsupported variable!"),
    }
}

#[derive(Clone)]
struct Horizontal<T: DataType> {
    columns: Vec<T>,
}

impl<T: DataType> Default for Horizontal<T> {
    fn default() -> Self {
        let mut columns = Vec::with_capacity(LONGITUDE_POINTS);
        columns.resize(LONGITUDE_POINTS, T::default());
        Self {
            columns
        }
    }
}

struct Data2d<T: DataType> {
    rows: Vec<Horizontal<T>>,
}

impl<T: DataType> Default for Data2d<T> {
    fn default() -> Self {
        let mut rows = Vec::with_capacity(LATITUDE_POINTS);
        rows.resize(LATITUDE_POINTS, Horizontal::default());
        Self {
            rows,
        }
    }
}

#[derive(Default)]
struct Data2dStatistics<T: DataType> {
    data: Data2d<T>,
    min: Option<T>,
    max: Option<T>,
}

trait PixelMappable<T: DataType> {
    fn get_pixel_map(&self) -> Box<dyn Fn(&T) -> u8>;
}

impl<T: DataType> Data2dStatistics<T> {
    fn from_variable(v: &Variable) -> Self {
        let mut data = Data2d::<T>::default();
        let mut min = None;
        let mut max = None;

        let dim = v.dimensions();
        assert_eq!(dim.len(), 3);
        assert_eq!(dim[TIME_DIMENSION].len(), 1);
        assert_eq!(dim[LATITUDE_DIMENSION].len(), LATITUDE_POINTS);
        assert_eq!(dim[LONGITUDE_DIMENSION].len(), LONGITUDE_POINTS);

        let mut valid: usize = 0;
        for row in 0..LATITUDE_POINTS {
            for column in 0..LONGITUDE_POINTS {
                if let Ok(val) = v.value::<T>(Some(&[0, row, column])) {
                    if max.is_none() || val > max.unwrap() {
                        max = Some(val);
                    }
                    if min.is_none() || val < min.unwrap() {
                        min = Some(val);
                    }
                    data.rows[row].columns[column] = val;
                    valid += 1;
                }
            }
        }

        assert_eq!(valid, data.rows.len() * data.rows[0].columns.len());
        Self { data, min, max, }
    }
}

impl PixelMappable<f64> for Data2dStatistics<f64> {
    fn get_pixel_map(&self) -> Box<dyn Fn(&f64) -> u8> {
        let range = self.max.unwrap() - self.min.unwrap();
        let offset = self.min.unwrap();
        Box::new(move |value: &f64| {
            let portion = (*value - offset) / range;
            (255.0 * portion) as u8
        })
    }
}

impl<T: DataType> Sub for &Data2dStatistics<T> where T: Sub<Output = T> {
    type Output = Data2dStatistics<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut difference = Data2d::default();
        let mut min = None;
        let mut max = None;

        for (row, (a_row, b_row)) in self.data.rows.iter().zip(rhs.data.rows.iter()).enumerate() {
            for (col, (a_val, b_val)) in a_row.columns.iter().zip(b_row.columns.iter()).enumerate() {
                let val_difference = *b_val - *a_val;
                if max.is_none() || val_difference > max.unwrap() {
                    max = Some(val_difference);
                }
                if min.is_none() || val_difference < min.unwrap() {
                    min = Some(val_difference);
                }
                difference.rows[row].columns[col] = val_difference;
            }
        }

        Data2dStatistics {
            data: difference,
            min,
            max
        }
    }
}


trait ToImage<P: Pixel> {
    type Data;
    fn to_image(&self) -> ImageBuffer<P, Vec<P::Subpixel>>;
}

impl<T: DataType> ToImage<Luma<u8>> for Data2dStatistics<T> where Data2dStatistics<T>: PixelMappable<T> {
    type Data = T;

    fn to_image(&self) -> ImageBuffer<Luma<u8>, Vec<u8>> {
        let pixel_map = PixelMappable::<T>::get_pixel_map(self);
        let mut output_buffer = Vec::with_capacity(LONGITUDE_POINTS * LATITUDE_POINTS);
        for row in self.data.rows.iter().rev() {
            for val in row.columns.iter() {
                output_buffer.push(pixel_map(val));
            }
        }

        GrayImage::from_raw(LONGITUDE_POINTS as u32, LATITUDE_POINTS as u32, output_buffer)
            .expect("Failed to create image!")
    }
}

impl<T: DataType> ToImage<LumaA<u8>> for [Data2dStatistics<T>; 2] where Data2dStatistics<T>: PixelMappable<T> {
    type Data = T;

    fn to_image(&self) -> ImageBuffer<LumaA<u8>, Vec<u8>> {
        let pixel_maps: Vec<Box<dyn Fn(&T) -> u8>> = self.iter()
            .map(|ds| PixelMappable::<T>::get_pixel_map(ds)).collect();
        let mut output_buffer = Vec::with_capacity(2 * LONGITUDE_POINTS * LATITUDE_POINTS);
        for (row1, row2) in self[0].data.rows.iter().rev().zip(self[1].data.rows.iter().rev()) {
            for (val0, val1) in row1.columns.iter().zip(row2.columns.iter()) {
                output_buffer.push(pixel_maps[0](val0));
                output_buffer.push(pixel_maps[1](val1));
            }
        }

        GrayAlphaImage::from_raw(LONGITUDE_POINTS as u32, LATITUDE_POINTS as u32, output_buffer)
            .expect("Failed to create image!")
    }
}

impl<T: DataType> ToImage<Rgb<u8>> for [Data2dStatistics<T>; 3] where Data2dStatistics<T>: PixelMappable<T> {
    type Data = T;

    fn to_image(&self) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        let pixel_maps: Vec<Box<dyn Fn(&T) -> u8>> = self.iter()
            .map(|ds| PixelMappable::<T>::get_pixel_map(ds)).collect();
        let mut output_buffer = Vec::with_capacity(3 * LONGITUDE_POINTS * LATITUDE_POINTS);
        for rows in izip!(self[0].data.rows.iter().rev(), self[1].data.rows.iter().rev(), self[2].data.rows.iter().rev()) {
            for vals in izip!(rows.0.columns.iter(), rows.1.columns.iter(), rows.2.columns.iter()) {
                output_buffer.push(pixel_maps[0](vals.0));
                output_buffer.push(pixel_maps[1](vals.1));
                output_buffer.push(pixel_maps[2](vals.2));
            }
        }

        RgbImage::from_raw(LONGITUDE_POINTS as u32, LATITUDE_POINTS as u32, output_buffer)
            .expect("Failed to create image!")
    }
}

trait ToMetadata {
    fn to_metadata(&self) -> Metadata;
}

impl ToMetadata for [Data2dStatistics<f64>; 3] {
    fn to_metadata(&self) -> Metadata {
        self.iter()
            .map(|ds| ChannelMetadata{ min: ds.min.unwrap(), max: ds.max.unwrap() } )
            .collect()
    }
}
