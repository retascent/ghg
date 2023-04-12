use std::ffi::OsStr;
use std::path::{Path, PathBuf};

pub fn find_data_files(root_path: &Path, extension: &OsStr) -> Vec<PathBuf> {
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
