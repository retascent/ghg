#![feature(trait_alias)]

use std::fs::File;
use std::path::{Path, PathBuf};
use ghg::data_processing::data_model::{Data2dStatistics, DataType, ToImage, ToMetadata};
use ghg::data_processing::read_data::{find_data_files};
use ghg::data_processing::save_result::save_channels;
use ghg::data_processing::file_type::{Nc4, DataFile, CdfMetadata};

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

// Instantaneous Two-Dimensional Collections: instM_2d_asm_Nx (M2IMNXASM): Single-Level Diagnostics

fn main() -> std::io::Result<()> {
    let output_root = Path::new("www/images/earth_ozone");
    let data_source = Path::new("raw_data/merra2_1980_2021");
    let data_paths = find_data_files(data_source, Nc4::<f64>::extension());

    for month in 0..12 {
        println!("Month {month}");
        let files = paths_from_month(&data_paths, month);
        assert_eq!(files.len(), 2);
        println!("Files: {files:?}");

        let metadata = CdfMetadata{width_dimension: 2, height_dimension: 1};

        let (data_1980, data_2021, differences) = {
            let variables = ["TO3".to_owned()];
            let mut data_1980 = Nc4::open(&files[0], metadata)
                .expect(format!("Failed to read file {:?}", files[0].file_name().unwrap()).as_str())
                .read_variables(&variables);
            let mut data_2021 = Nc4::open(&files[1], metadata)
                .expect(format!("Failed to read file {:?}", files[0].file_name().unwrap()).as_str())
                .read_variables(&variables);

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

fn paths_from_month(all_paths: &Vec<PathBuf>, month: i32) -> Vec<PathBuf> {
    all_paths.iter()
        .filter(|p| {
            let filename = p.file_stem().unwrap();
            let name_str = filename.to_str().unwrap();
            name_str.ends_with(format!("{:0>2}", month + 1).as_str())
        }).cloned().collect()
}
