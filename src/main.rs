mod helper;
pub mod logger; // enables the log_debug!() and log_info!() macros

use crate::helper::Record;
use chrono::Utc;
use clap::{arg, Command};
use std::env;
use std::path::PathBuf;
use std::time::Instant;

/*  -----------------------------------------------------------------
    Includes the file generated by the build.rs script, which looks like:
        pub const GIT_COMMIT: &str = "c5f7034f79bc3d49c1a9fb81c7cac6a8a778c5c3";
    Purpose: to enable `parse_ocr_tracker --version` to show the git commit hash.
    -----------------------------------------------------------------
*/
include!(concat!(env!("OUT_DIR"), "/git_commit.rs")); // OUT_DIR is set by cargo; is the target dir; and is only available during build process

/*  -----------------------------------------------------------------
    Main manager function.
    -----------------------------------------------------------------
*/
fn main() {
    // -- create start-times ----------------------------------------
    let start_instant = Instant::now(); // monotonic clock starts
    let datestamp_time = Utc::now(); // for time-zone aware datestamp for output json

    // init logger --------------------------------------------------
    logger::init_logger().unwrap();

    // grab log-level -----------------------------------------------
    // -- only needed for json output -- grabbed by logger::init_logger()
    let mut log_level: String = env::var("LOG_LEVEL").unwrap_or_else(|_| "warn".to_string());
    log_level = log_level.to_lowercase();
    if log_level != "debug" && log_level != "info" && log_level != "warn" {
        log_level = "warn".to_string();
    }

    // setup and read cli-args --------------------------------------
    let about_text = r#"Info...
  - Walks `source_dir_path` and creates `(output_dir_path)/tracker_output.csv`.
  - Logs to console only; default log-level is 'warn'; use `export LOG_LEVEL="debug"` or "info" to see more output.
  - Useful json is returned, and saved to `(output_dir_path)/tracker_info.json`."#;
    let matches = Command::new("parse_ocr_tracker")
        .version(GIT_COMMIT)
        .about(about_text)
        .arg(arg!(-s --source_dir_path <VALUE>).required(true))
        .arg(arg!(-o --output_dir_path <VALUE>).required(true))
        .get_matches();

    // get source_dir -----------------------------------------------
    let source_dir_temp_ref: &String = matches
        .get_one::<String>("source_dir_path")
        .expect("Failed to get required 'source_dir_path' argument.");
    let source_dir: &str = source_dir_temp_ref.as_str(); // or... let source_dir: String = source_dir_temp_ref.to_string();
                                                         // log_debug!("source-arg: {:?}", source_dir);
    log_info!("source-arg: {:?}", source_dir);

    // get output_dir -----------------------------------------------
    let output_dir_temp_ref: &String = matches
        .get_one::<String>("output_dir_path")
        .expect("Failed to get required 'output_dir_path' argument.");
    let output_dir: &str = output_dir_temp_ref.as_str();
    log_info!("output-arg: {:?}", output_dir);

    // get paths ----------------------------------------------------
    let (ocr_paths, ingest_paths, error_paths, other_paths) = helper::find_json_files(source_dir);
    let ocr_tracker_paths_count = ocr_paths.len();
    log_warn!("len(ocr_paths): {}", ocr_tracker_paths_count);
    let _ingest_tracker_paths_count = ingest_paths.len();
    let _error_tracker_paths_count = error_paths.len();
    let _other_json_paths_count = other_paths.len();

    // make a map of id-to-pid --------------------------------------
    let id_to_pid_map = helper::make_id_to_pid_map(ingest_paths);

    // -- process ocr-tracker-files ---------------------------------
    let path_results: helper::PathResults = helper::process_files(ocr_paths, &id_to_pid_map) // PathResults is a struct just to hold and return the two vectors
        .unwrap_or_else(|e| {
            eprintln!("Failed to process the ocr-tracker-files: {}", e);
            std::process::exit(1); // Exit or handle the error by returning a default value or performing other actions
        });
    let data_vector: Vec<Record> = path_results.extracted_data_files;
    let ocr_data_vector_count: usize = data_vector.len();
    let rejected_files: Vec<PathBuf> = path_results.rejected_paths;
    log_debug!("all rejected_file paths...");
    for file in &rejected_files {
        log_debug!("{:?}", file);
    }
    let rejected_files_count: usize = rejected_files.len();

    // -- save csv --------------------------------------------------
    let csv_file_path = helper::save_to_csv(&data_vector, output_dir);
    let csv_file_path: Option<String> = match csv_file_path {
        Ok(file_path) => {
            log_info!("CSV saved successfully at: {}", file_path);
            Some(file_path)
        }
        Err(e) => {
            log_info!("Error saving to CSV: {}", e);
            None // or handle the error as needed
        }
    };

    // prepare json -------------------------------------------------
    let return_json: String =
        // helper::prepare_json(csv_file_path, &_error_paths, start_instant, datestamp_time);
        helper::prepare_json(source_dir, output_dir, log_level, csv_file_path, ocr_data_vector_count, rejected_files_count, error_paths, start_instant, datestamp_time);
    println!("{}", return_json);
}

// let zz: () = the_var; // for reference -- hack to inspect the type of the_var
