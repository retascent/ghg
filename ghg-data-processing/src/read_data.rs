use std::ffi::OsStr;
use std::path::{Path, PathBuf};

pub fn find_data_files(root_path: &Path, supported_extensions: &[&OsStr]) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = Vec::new();

    for filename in root_path.read_dir().unwrap() {
        let path = filename.unwrap().path();
        if let Some(current_extension) = path.extension() {
            if is_supported(current_extension, supported_extensions) {
                paths.push(path);
            }
        }
    }

    paths.sort();
    paths
}

fn is_supported(extension: &OsStr, supported_extensions: &[&OsStr]) -> bool {
    // Lazy
    for &supported in supported_extensions {
        if extension == supported {
            return true;
        }
    }
    false
}
