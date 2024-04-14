// Assuming `logger` is declared as `pub mod logger;` in `main.rs`
use crate::{log_info, log_debug}; // Import logging functions

use std::{
    // fs::File,
    // io::{self, Read},
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

/*  -----------------------------------------------------------------
    Finds all files in the given directory that end with "ocr_complete.json" or "ingest_complete.json".
    -----------------------------------------------------------------
*/
pub fn find_json_files<P: AsRef<Path>>(
    path: P,
) -> (Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>) {
    log_debug!("starting find_json_files()");
    // -- setup data-vectors
    let mut ocr_complete_paths = Vec::new();
    let mut ingest_complete_paths = Vec::new();
    let mut error_paths = Vec::new();
    let mut other_paths = Vec::new();
    // -- take a walk
    for entry in WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
    {
        let path = entry.into_path();
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            if file_name.ends_with("ocr_complete.json") {
                ocr_complete_paths.push(path);
            } else if file_name.ends_with("ingest_complete.json") {
                ingest_complete_paths.push(path);
            } else if file_name.contains("error") {
                error_paths.push(path);
            } else {
                other_paths.push(path);
            }
        }
    }
    // -- sort the vectors
    ocr_complete_paths.sort_by(|a, b| a.as_path().cmp(b.as_path()));
    ingest_complete_paths.sort_by(|a, b| a.as_path().cmp(b.as_path()));
    // output counts
    log_info!("len-ocr_complete_paths: {}", ocr_complete_paths.len());
    log_info!("len-ingest_complete_paths: {}", ingest_complete_paths.len());
    log_info!("len-error_paths: {}", error_paths.len());
    log_info!("len-other_paths: {}", other_paths.len());
    // -- return
    (
        ocr_complete_paths,
        ingest_complete_paths,
        error_paths,
        other_paths,
    )
}
