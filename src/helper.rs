use crate::{log_debug, log_info}; // requires `logger` to be declared as `pub mod logger;` in `main.rs
use indexmap::IndexMap;
use rayon::{iter::Either, prelude::*};
use serde::{Deserialize, Serialize};
use serde_json::{json, Result as JsonResult, Value};
use std::{
    collections::BTreeMap,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    time::Instant,
};
use walkdir::WalkDir;

/*  -----------------------------------------------------------------
    Represents the structure of the -->OCR<-- JSON tracker files being parsed.
    Note that the `pid` and `pid_url` fields are not part of the original JSON files; they're populated later.
    -----------------------------------------------------------------
*/
#[derive(Debug, Deserialize, Serialize)]
pub struct Record {
    orientation: i32,
    orientation_conf: f64,
    script: String,
    script_conf: f64,
    image_name: String,
    word_count: i32,
    avg_confidence: f64,
    below_90: f64,
    below_60: f64,
    below_30: f64,
    pid: Option<String>,     // populated later
    pid_url: Option<String>, // populated later
}

/*  -----------------------------------------------------------------
    Represents the structure of the -->ingestion<-- JSON tracker files that just have a pid.
    The `id` field will be populated by parsing the local-id from the filepath.
    -----------------------------------------------------------------
*/
#[derive(Debug, Deserialize, Serialize)]
struct IdToPidInfo {
    id: Option<String>, // populated later
    pid: String,
}

/*  -----------------------------------------------------------------
    Finds all files in the given directory that end with "ocr_complete.json" or "ingest_complete.json".
    -----------------------------------------------------------------
*/
pub fn find_json_files<P: AsRef<Path>>(path: P) -> (Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>) {
    log_debug!("starting find_json_files()");

    let entries: Vec<PathBuf> = WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .map(|e| e.into_path())
        .collect();

    // Use into_par_iter to consume entries and yield owned PathBufs
    let (mut ocr_complete_paths, others): (Vec<PathBuf>, Vec<PathBuf>) =
        entries.into_par_iter().partition(|path| {
            path.file_name().and_then(|n| n.to_str()).map_or(false, |n| n.ends_with("ocr_complete.json"))
        });

    let (mut ingest_complete_paths, remaining): (Vec<PathBuf>, Vec<PathBuf>) =
        others.into_par_iter().partition(|path| {
            path.file_name().and_then(|n| n.to_str()).map_or(false, |n| n.ends_with("ingest_complete.json"))
        });

    let (mut error_paths, mut other_paths): (Vec<PathBuf>, Vec<PathBuf>) = remaining
        .into_par_iter()
        .partition(|path| path.file_name().and_then(|n| n.to_str()).map_or(false, |n| n.contains("error")));

    // Optionally, sort the paths; note sorting is not parallelized
    ocr_complete_paths.par_sort_unstable();
    ingest_complete_paths.par_sort_unstable();
    error_paths.par_sort_unstable();
    other_paths.par_sort_unstable();

    log_info!("len-ocr_complete_paths: {}", ocr_complete_paths.len());
    log_info!("len-ingest_complete_paths: {}", ingest_complete_paths.len());
    log_info!("len-error_paths: {}", error_paths.len());
    log_info!("len-other_paths: {}", other_paths.len());

    (
        ocr_complete_paths,
        ingest_complete_paths,
        error_paths,
        other_paths,
    )
}

// pub fn find_json_files<P: AsRef<Path>>(path: P) -> (Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>) {
//     log_debug!("starting find_json_files()");
//     // -- setup data-vectors
//     let mut ocr_complete_paths = Vec::new();
//     let mut ingest_complete_paths = Vec::new();
//     let mut error_paths = Vec::new();
//     let mut other_paths = Vec::new();
//     // -- take a walk
//     for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()).filter(|e| e.path().is_file()) {
//         let path = entry.into_path();
//         if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
//             if file_name.ends_with("ocr_complete.json") {
//                 ocr_complete_paths.push(path);
//             } else if file_name.ends_with("ingest_complete.json") {
//                 ingest_complete_paths.push(path);
//             } else if file_name.contains("error") {
//                 error_paths.push(path);
//             } else {
//                 other_paths.push(path);
//             }
//         }
//     }
//     // -- sort the vectors
//     ocr_complete_paths.sort_by(|a, b| a.as_path().cmp(b.as_path()));
//     ingest_complete_paths.sort_by(|a, b| a.as_path().cmp(b.as_path()));
//     // output counts
//     log_info!("len-ocr_complete_paths: {}", ocr_complete_paths.len());
//     log_info!("len-ingest_complete_paths: {}", ingest_complete_paths.len());
//     log_info!("len-error_paths: {}", error_paths.len());
//     log_info!("len-other_paths: {}", other_paths.len());
//     // -- return
//     (
//         ocr_complete_paths,
//         ingest_complete_paths,
//         error_paths,
//         other_paths,
//     )
// } // end fn find_json_files()

/*  -----------------------------------------------------------------
    Creates a hashmap of id-to-pid.
    (Ok ok, it's a BTreeMap, not a hashmap, cuz I wanted it sorted.)
    -----------------------------------------------------------------
*/
pub fn make_id_to_pid_map(file_paths: Vec<PathBuf>) -> BTreeMap<String, String> {
    let id_to_pid_map: BTreeMap<String, String> = file_paths
        .par_iter()
        .filter_map(|path_buf| {
            let path = path_buf.as_path();
            let key = parse_key_from_path(&path);
            let mut file = match File::open(&path) {
                Ok(file) => file,
                Err(e) => {
                    log_debug!("Error opening file {:?}: {}", path, e);
                    return None;
                }
            };
            let mut contents = String::new();
            if let Err(e) = file.read_to_string(&mut contents) {
                log_debug!("Error reading file to string {:?}: {}", path, e);
                return None;
            }
            let record: JsonResult<IdToPidInfo> = serde_json::from_str(&contents);
            match record {
                Ok(rec) => Some((key, rec.pid)),
                Err(e) => {
                    log_debug!("Error parsing JSON from {:?}: {}", path, e);
                    None
                }
            }
        })
        .collect();

    log_debug!("id_to_pid_map, ``{:#?}``", id_to_pid_map);
    id_to_pid_map
}

// pub fn make_id_to_pid_map(file_paths: Vec<PathBuf>) -> BTreeMap<String, String> {
//     let mut id_to_pid_map = BTreeMap::new();
//     for path_buf in file_paths {
//         let path = path_buf.as_path();
//         let key = parse_key_from_path(&path);
//         let mut file = match File::open(&path) {
//             Ok(file) => file,
//             Err(e) => {
//                 log_debug!("Error opening file {:?}: {}", path, e);
//                 continue;
//             }
//         };
//         let mut contents = String::new();
//         if let Err(e) = file.read_to_string(&mut contents) {
//             log_debug!("Error reading file to string {:?}: {}", path, e);
//             continue;
//         }
//         let record: JsonResult<IdToPidInfo> = serde_json::from_str(&contents);
//         match record {
//             Ok(rec) => {
//                 let id = key;
//                 let pid = rec.pid;
//                 id_to_pid_map.insert(id, pid);
//             }
//             Err(e) => log_debug!("Error parsing JSON from {:?}: {}", path, e),
//         }
//     }
//     log_debug!("id_to_pid_map, ``{:#?}``", id_to_pid_map);
//     id_to_pid_map
// }

/*  -----------------------------------------------------------------
    Parses out `HH001545_0001` from a path like: `/path/to/HH001545/HH001545_0001/HH001545_0001-ingest_complete.json`
    Called by:
        - make_id_to_pid_map() to create the hashmap
        - and then by process_files() to get the key to do the hashmap lookup.
    -----------------------------------------------------------------
*/
pub fn parse_key_from_path(path: &Path) -> String {
    let key = path
        .file_stem() // Get the file stem from the path
        .and_then(|s| s.to_str()) // Convert OsStr to &str
        .map(|s| s.split('-').next()) // Split at '-' and take the first part
        .flatten() // Option<&str> from Option<Option<&str>>
        .map(|s| s.to_string()) // Convert &str to String
        .unwrap_or_else(|| "unknown_key".to_string()); // Provide default value on error
    log_debug!("key, ``{}``", key);
    key
}

/*  -----------------------------------------------------------------
    Processes the JSON files
    - creates the data-vector that'll be used to create the CSV
    - creates the vector of rejected paths -- basically organization-tracker-files
    -----------------------------------------------------------------
*/
pub struct PathResults {
    // just a struct to hold and return the two vectors below
    pub extracted_data_files: Vec<Record>,
    pub rejected_paths: Vec<PathBuf>,
}

pub fn process_files(
    ocr_tracker_filepaths: Vec<PathBuf>, id_to_pid_map: &BTreeMap<String, String>,
) -> Result<PathResults, std::io::Error> {
    let (temp_tracker_data_vector, temp_rejected_paths): (Vec<_>, Vec<_>) = ocr_tracker_filepaths
        .par_iter() // Use parallel iterator
        .map(|ocr_tracker_filepath_buf| {
            let ocr_tracker_filepath: &Path = ocr_tracker_filepath_buf.as_path();
            let item_num_key: String = parse_key_from_path(&ocr_tracker_filepath);

            match File::open(&ocr_tracker_filepath)
                .and_then(|mut file| {
                    let mut contents = String::new();
                    file.read_to_string(&mut contents)?;
                    Ok(contents)
                })
                .map_err(|e| e.to_string())
                .and_then(|contents| serde_json::from_str::<Record>(&contents).map_err(|e| e.to_string()))
            {
                Ok(mut rec) => {
                    let pid = id_to_pid_map.get(&item_num_key).cloned();
                    let url = pid
                        .as_ref()
                        .map(|p| format!(" https://repository.library.brown.edu/studio/item/{}/", p));
                    rec.pid = pid;
                    rec.pid_url = url;
                    Ok(rec)
                }
                Err(e) => {
                    log_debug!(
                        "error parsing ocr-json from ``{:?}``: ``{}`` -- likely an organization-file",
                        ocr_tracker_filepath,
                        e
                    );
                    Err(ocr_tracker_filepath_buf.clone())
                }
            }
        })
        .partition_map(|result| match result {
            Ok(rec) => Either::Left(rec),
            Err(path) => Either::Right(path),
        });

    Ok(PathResults {
        extracted_data_files: temp_tracker_data_vector,
        rejected_paths: temp_rejected_paths,
    })
}

// pub fn process_files(
//     ocr_tracker_filepaths: Vec<PathBuf>, id_to_pid_map: &BTreeMap<String, String>,
// ) -> Result<PathResults, std::io::Error> {
//     // set up the vectors to hold the return-data -------------------
//     let mut temp_tracker_data_vector: Vec<Record> = Vec::new();
//     let mut temp_rejected_paths: Vec<PathBuf> = Vec::new();
//     // loop through the ocr-tracker-files ---------------------------
//     for ocr_tracker_filepath_buf in ocr_tracker_filepaths {
//         let ocr_tracker_filepath: &Path = ocr_tracker_filepath_buf.as_path();
//         let item_num_key: String = parse_key_from_path(&ocr_tracker_filepath); // get the key for the hashmap lookiup
//                                                                                // read ocr-data --------------------------------------------
//         let mut ocr_tracker_file_obj = File::open(&ocr_tracker_filepath)?;
//         let mut ocr_tracker_contents = String::new();
//         ocr_tracker_file_obj.read_to_string(&mut ocr_tracker_contents)?;
//         let record: JsonResult<Record> = serde_json::from_str(&ocr_tracker_contents);
//         match record {
//             Ok(mut rec) => {
//                 // look up pid and url from hashmap -----------------
//                 let pid: Option<&String> = id_to_pid_map.get(&item_num_key);
//                 let url: Option<String> =
//                     pid.map(|p| format!(" https://repository.library.brown.edu/studio/item/{}/", p));
//                 rec.pid = pid.cloned();
//                 rec.pid_url = url;
//                 // append record to data-vector ---------------------
//                 temp_tracker_data_vector.push(rec);
//             }
//             Err(e) => {
//                 log_debug!(
//                     "error parsing ocr-json from ``{:?}``: ``{}`` -- likely an organization-file",
//                     ocr_tracker_filepath,
//                     e
//                 );
//                 temp_rejected_paths.push(ocr_tracker_filepath_buf);
//             }
//         }
//     }

//     Ok(PathResults {
//         extracted_data_files: temp_tracker_data_vector,
//         rejected_paths: temp_rejected_paths,
//     })
// } // end fn process_files()

/*  -----------------------------------------------------------------
    Saves the data-vector to a CSV file.
    -----------------------------------------------------------------
*/
pub fn save_to_csv(data: &[Record], output_dir: &str, formatted_date_time: &str) -> Result<String, String> {
    // -- update the formatted_date_time
    let formatted_date_time: &str = formatted_date_time.split_whitespace().next().unwrap();
    let trimmed_datetime: &str = &formatted_date_time[0..19]; // slice up to the excluded timezone
    let date_for_filename: String = trimmed_datetime.replace(":", "-"); // replaces colons with hyphens
    log_debug!("date_for_filename: {}", &date_for_filename);
    let file_path: String = format!("{}/tracker_output_{}.csv", output_dir, date_for_filename);
    // -- create the file
    let file = match File::create(&file_path) {
        Ok(file) => file,
        Err(e) => return Err(format!("Failed to create file: {}", e)),
    };
    let mut wrtr = csv::Writer::from_writer(file);
    // -- write the data
    for record in data {
        if let Err(e) = wrtr.serialize(record) {
            return Err(format!("Failed to serialize record: {}", e));
        }
    }
    if let Err(e) = wrtr.flush() {
        return Err(format!("Failed to flush writer: {}", e));
    }
    // -- return the file-path
    Ok(file_path)
}

/*  -----------------------------------------------------------------
    Prepares a JSON file with datestamp, elapsed, source and output paths, and error-paths.
    -----------------------------------------------------------------
*/
pub fn prepare_json(
    source_dir: &str, output_dir: &str, log_level: String, csv_file_path: Option<String>,
    ocr_data_vector_count: usize, rejected_files_count: usize, error_paths: Vec<PathBuf>,
    start_instant: Instant, formatted_date_time: String,
) -> String {
    // -- create the main Map
    let mut map = IndexMap::<String, Value>::new();

    // -- convert UTC-Time to Eastern-Time (automatically handles DST)
    // -- update times
    // let eastern_time = utc_now_time.with_timezone(&Eastern);
    // let formatted_date_time = eastern_time.format("%Y-%m-%d_%H:%M:%S_%:z").to_string();
    map.insert("datetime_stamp".to_string(), json!(formatted_date_time));
    map.insert("time_taken".to_string(), json!("temp_holder")); // the same insert-key will update it later

    // -- basic data
    map.insert("source_dir_path".to_string(), json!(source_dir));
    map.insert("output_dir_path".to_string(), json!(output_dir));
    let log_level_str = format!("`{}`; see `--help` for more info", log_level);
    map.insert("log_level".to_string(), json!(log_level_str));

    // -- tracker-csv path
    map.insert("tracker_output_csv_path".to_string(), json!(csv_file_path));
    map.insert("ocr_data_vector_count".to_string(), json!(ocr_data_vector_count));

    // -- rejected-files count
    map.insert(
        "rejected_files_count_(org_tracker_files)".to_string(),
        json!(rejected_files_count),
    );

    // -- error-paths
    let mut error_paths_vec: Vec<String> = Vec::new();
    for path in error_paths {
        let path_str = path.to_string_lossy().to_string();
        error_paths_vec.push(path_str);
    }
    map.insert("error_paths".to_string(), json!(error_paths_vec));

    // -- finally, update elapsed time value (the key was created above)
    let elapsed_seconds: f64 = start_instant.elapsed().as_secs_f64(); // uses monotonic clock
    let elapsed_string: String = if elapsed_seconds < 60.0 {
        format!("{:.1} seconds", elapsed_seconds)
    } else {
        let elapsed_minutes = elapsed_seconds / 60.0;
        format!("{:.1} minutes", elapsed_minutes)
    };
    map.insert("time_taken".to_string(), json!(elapsed_string));

    // -- convert the map into a JSON string
    match serde_json::to_string_pretty(&map) {
        Ok(json) => json,
        Err(e) => format!("Error serializing output-JSON: {}", e),
    }
}
