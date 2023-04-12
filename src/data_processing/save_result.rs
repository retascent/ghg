#[macro_export]
macro_rules! save_channels {
    ($output_name:expr, $($channels:expr),+) => {
        const NUM_CHANNELS: usize = ghg::count!($($channels)+);
        let output_channels: [Data2dStatistics<f64>; NUM_CHANNELS] = [$($channels),+];
        output_channels.to_image()
            .save($output_name.clone())
            .expect("Failed to save data as image!");

        println!("Saved image: {:?}", $output_name);

        let metadata_name = $output_name.with_extension("metadata");
        let mut metadata_file = File::create(metadata_name.clone()).expect("Failed to create metadata file");
        let metadata = serde_json::to_string(&output_channels.to_metadata()).expect("Failed to serialize metadata");
        write!(metadata_file, "{}", metadata).expect("Failed to write metadata");

        println!("Saved metadata: {:?}", metadata_name);
    };
}

pub use save_channels;
