#![feature(trait_alias)]

use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use ghg_data_processing::data_model::{Data2dStatistics, DataType, ToImage, ToMetadata};
use ghg_data_processing::file_type::{CdfMetadata, DataFile, Nc4};
use ghg_data_processing::read_data::find_data_files;
use ghg_data_processing::save_result::save_channels;
use rayon::prelude::*;

/// If you have a problem finding HDF5 or netCDF, make sure to run this with
/// these flags:     --all-features --features hdf5-sys/static,netcdf/static
/// This will ensure the HDF5 and netCDF projects are built statically, instead
/// of looking for them to be installed. The `--all-features` flag is always
/// needed, but the latter flags are only needed if the build can't locate the
/// installs. This is a loader for the 2m air temperature data source.
/// More information about the data:
///  - [doi:10.5067/5ESKGQTZG7FO](https://doi.org/10.5067/5ESKGQTZG7FO)
///  - [Data format specification](https://gmao.gsfc.nasa.gov/pubs/docs/Bosilovich785.pdf)
///  - [Data information](https://cmr.earthdata.nasa.gov/search/concepts/C1276812823-GES_DISC.html)
///  - [1980 data download](https://search.earthdata.nasa.gov/search/granules?p=C1276812823-GES_DISC&pg[0][v]=f&pg[0][gsk]=-start_date&q=C1276812823-GES_DISC&qt=1980-01-01T00%3A00%3A00.000Z%2C1980-12-31T23%3A59%3A59.999Z&tl=1659554690.391!3!!)
///  - [2021 data download](https://search.earthdata.nasa.gov/search/granules?p=C1276812823-GES_DISC&pg[0][v]=f&pg[0][gsk]=-start_date&q=C1276812823-GES_DISC&qt=2021-01-01T00%3A00%3A00.000Z%2C2021-12-31T23%3A59%3A59.999Z&tl=1659554690.391!3!!)
///
/// All data should be downloaded into /raw_data/merra2_1980_2021 within this
/// repo.

// Instantaneous Two-Dimensional Collections: instM_2d_asm_Nx (M2IMNXASM):
// Single-Level Diagnostics

fn main() -> std::io::Result<()> {
	let output_root = Path::new("ghg/www/images/earth_temp");
	assert!(output_root.exists());

	let data_source = Path::new("raw_data/merra2_1980_2021");
	assert!(data_source.exists());

	let data_paths = find_data_files(data_source, &[Nc4::<f64>::extension()]);

	let metadata = CdfMetadata { width_dimension: 2, height_dimension: 1 };
	let variables = ["T2M".to_owned()];

	for year in 1980..=2021 {
		println!(">>> Starting year {year} <<<");
		let year_files = paths_from_year(&data_paths, year);
		println!("  Files: {year_files:?}");

		(0..3).into_iter().for_each(|month_stride| {
			produce_stride_image(output_root, year, &year_files, metadata, &variables, month_stride)
		});
	}

	Ok(())
}

fn produce_stride_image(
	output_root: &Path,
	year: i32,
	year_files: &Vec<PathBuf>,
	metadata: CdfMetadata,
	variables: &[String],
	month_stride: i32,
) {
	let mut stride_data: Vec<Data2dStatistics<f64>> = Vec::new();

	const MONTH_STRIDE_LENGTH: i32 = 4;
	for month_cursor in 0..MONTH_STRIDE_LENGTH {
		let month = month_stride * MONTH_STRIDE_LENGTH + month_cursor;
		println!("  Month: {}", month + 1);

		let files = paths_from_month(year_files, month);
		assert_eq!(files.len(), 1);

		let mut data = Nc4::<f64>::open(&files[0], metadata)
			.expect(format!("Failed to read file {:?}", files[0].file_name().unwrap()).as_str())
			.read_variables(&variables);
		assert_eq!(data.len(), 1);

		stride_data.push(data.remove(0));
	}
	let output_name = output_root.join(format!(
		"{:0>4}.{:0>2}.{:0>2}.png",
		year,
		month_stride * MONTH_STRIDE_LENGTH + 1,
		month_stride * MONTH_STRIDE_LENGTH + MONTH_STRIDE_LENGTH
	));
	save_channels!(output_name, to_array::<Data2dStatistics<f64>, 4>(stride_data.clone()));
}

fn to_array<T, const N: usize>(v: Vec<T>) -> [T; N] {
	v.try_into()
		.unwrap_or_else(|v: Vec<T>| panic!("Expected a Vec of length {} but it was {}", N, v.len()))
}

fn paths_from_month(all_paths: &Vec<PathBuf>, month: i32) -> Vec<PathBuf> {
	all_paths
		.iter()
		.filter(|p| {
			let filename = p.file_stem().unwrap();
			let name_str = filename.to_str().unwrap();
			name_str.ends_with(format!("{:0>2}", month + 1).as_str())
		})
		.cloned()
		.collect()
}

fn paths_from_year(all_paths: &Vec<PathBuf>, year: i32) -> Vec<PathBuf> {
	all_paths
		.iter()
		.filter(|p| {
			let filename = p.file_stem().unwrap();
			let name_str = filename.to_str().unwrap();
			name_str.contains(format!("Nx.{:0>4}", year).as_str())
		})
		.cloned()
		.collect()
}
