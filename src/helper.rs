use crate::{log_debug, log_info}; // requires `logger` to be declared as `pub mod logger;` in `main.rs

use serde::{Deserialize, Serialize};
use serde_json::Result as JsonResult;
use std::{
    collections::BTreeMap,
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
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
} // end fn find_json_files()

/*  -----------------------------------------------------------------
    Creates a hashmap of id-to-pid.
    (Ok ok, it's a BTreeMap, not a hashmap, cuz I wanted it sorted.)
    -----------------------------------------------------------------
*/
pub fn make_id_to_pid_map(file_paths: Vec<PathBuf>) -> BTreeMap<String, String> {
    let mut id_to_pid_map = BTreeMap::new();
    for path_buf in file_paths {
        let path = path_buf.as_path();
        let key = parse_key_from_path(&path);
        let mut file = match File::open(&path) {
            Ok(file) => file,
            Err(e) => {
                log_debug!("Error opening file {:?}: {}", path, e);
                continue;
            }
        };
        let mut contents = String::new();
        if let Err(e) = file.read_to_string(&mut contents) {
            log_debug!("Error reading file to string {:?}: {}", path, e);
            continue;
        }
        let record: JsonResult<IdToPidInfo> = serde_json::from_str(&contents);
        match record {
            Ok(rec) => {
                let id = key;
                let pid = rec.pid;
                id_to_pid_map.insert(id, pid);
            }
            Err(e) => log_debug!("Error parsing JSON from {:?}: {}", path, e),
        }
    }
    log_debug!("id_to_pid_map, ``{:#?}``", id_to_pid_map);
    id_to_pid_map
}

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
    Processes the JSON files, creating the data-vector that'll be used to create the CSV.
    -----------------------------------------------------------------
*/
pub fn process_files(
    file_paths: Vec<PathBuf>,
    id_to_pid_map: &BTreeMap<String, String>,
) -> io::Result<Vec<Record>> {
    let mut data_vector: Vec<Record> = Vec::new();
    for path_buf in file_paths {
        let path: &Path = path_buf.as_path();
        // let key: String = parse_key_from_path(&path);
        // let key: String = helper::parse_key_from_path(&path);
        let key: String = parse_key_from_path(&path);
        // reads ocr-data -------------------------------------------
        let mut file = File::open(&path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let record: JsonResult<Record> = serde_json::from_str(&contents);
        match record {
            Ok(mut rec) => {
                // looks up pid and url from hashmap ----------------
                let pid: Option<&String> = id_to_pid_map.get(&key);
                let url: Option<String> = pid
                    .map(|p| format!(" https://repository.library.brown.edu/studio/item/{}/", p));
                rec.pid = pid.cloned();
                rec.pid_url = url;
                // appends record to data-vector --------------------
                data_vector.push(rec);
            }
            Err(e) => log_debug!(
                "error parsing ocr-json from ``{:?}``: ``{}`` -- likely an organization-file",
                path,
                e
            ),
        }
    }

    Ok(data_vector)
} // end fn process_files()

/*  -----------------------------------------------------------------
    Saves the data-vector to a CSV file.
    -----------------------------------------------------------------
*/
pub fn save_to_csv(data: &[Record], output_dir: &str) -> io::Result<()> {
    let file_path = format!("{}/tracker_output.csv", output_dir); // Consider more sophisticated file naming
    let file = File::create(file_path)?;
    let mut wrtr = csv::Writer::from_writer(file);
    for record in data {
        wrtr.serialize(record)?;
    }
    wrtr.flush()?;
    Ok(())
}

/*  -----------------------------------------------------------------
    Prepares a JSON file with the error-paths.
    -----------------------------------------------------------------
*/
// use chrono::{DateTime, Utc, FixedOffset};
use chrono::{DateTime, Utc, FixedOffset};

pub fn prepare_json(_error_paths: &[PathBuf]) -> String {
    // -- create the main BTreeMap
    let mut map = BTreeMap::new();

    // -- prep current datetime with timezone
    let offset = FixedOffset::west_opt(5 * 3600).expect("Invalid timezone offset");
    let local_date_time: DateTime<FixedOffset> = Utc::now().with_timezone(&offset);
    let formatted_date_time = local_date_time.format("%Y-%m-%d_%H:%M:%S_%:z").to_string();
    map.insert("datetime_stamp", formatted_date_time);

    // Add other paths
    map.insert("source_path", "foo".to_string());
    map.insert("output_path", "bar".to_string());

    // Convert the BTreeMap into a JSON string
    match serde_json::to_string_pretty(&map) {
        Ok(json) => json,
        Err(e) => format!("Error serializing JSON: {}", e)
    }

    
    // // -- create a vector of strings
    // let mut error_paths_vec: Vec<String> = Vec::new();
    // for path in error_paths {
    //     let path_str = path.to_string_lossy().to_string();
    //     error_paths_vec.push(path_str);
    // }
    // let json = serde_json::to_string_pretty(&error_paths_vec)?;
    // println!("{}", json);
    // Ok(())
}
