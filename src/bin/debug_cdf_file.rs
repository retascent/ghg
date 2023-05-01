use std::env;
use std::path::Path;
use ghg::data_processing::data_model::ToImage;
use ghg::data_processing::file_type::{CdfMetadata, DataFile, Nc, Nc4};
use ghg::data_processing::read_data::find_data_files;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    assert_eq!(args.len(), 2);

    let data_source = Path::new(&args[1]);

    let data_files = find_data_files(
        data_source,
        &[Nc::<f64>::extension(), Nc4::<f64>::extension()]
    );
    if data_files.len() == 0 {
        println!("No data files found in path {:?}", data_source);
        return Ok(());
    }

    println!("Found files in path {:?}", data_source);
    for file in &data_files {
        println!("  - {file:?}");
    }

    let metadata = CdfMetadata{width_dimension: 0, height_dimension: 0};

    for file in &data_files {
        println!("\n\nReading file {file:?}\n");

        let data = if file.extension() == Some(Nc::<f64>::extension()) {
            Nc::<f64>::open(&data_files[0], metadata)
                .expect(format!("Failed to read file {:?}", data_files[0].file_name().unwrap()).as_str())
                .read_variables(&[])
        } else if file.extension() == Some(Nc4::<f64>::extension()) {
            Nc4::<f64>::open(&data_files[0], metadata)
                .expect(format!("Failed to read file {:?}", data_files[0].file_name().unwrap()).as_str())
                .read_variables(&[])
        } else {
            panic!("Unexpected file extension: {:?}", file.extension())
        };

        println!("\n\n>>> Results for {:?} <<<\n", file.file_name().unwrap());
        for stats in &data {
            println!("{}: min={:?}, max={:?}, width={:?}, height={:?}",
                     stats.name, stats.min.unwrap(), stats.max.unwrap(),
                     stats.width(), stats.height());
        }
    }


    Ok(())
}
