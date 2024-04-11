mod logger; // enables the log_debug!() and log_info!() macros

use clap::{arg, Command};
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
    Includes the file generated by the build.rs script, which looks like:
    pub const GIT_COMMIT: &str = "c5f7034f79bc3d49c1a9fb81c7cac6a8a778c5c3";
    -----------------------------------------------------------------
*/
include!(concat!(env!("OUT_DIR"), "/git_commit.rs")); // OUT_DIR is set by cargo; is the target dir; and is only available during build process

/*  -----------------------------------------------------------------
    Finds all files in the given directory that end with "ocr_complete.json" or "ingest_complete.json".
    -----------------------------------------------------------------
*/
fn find_json_files<P: AsRef<Path>>(
    path: P,
) -> (Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>) {
    let mut ocr_complete_paths = Vec::new();
    let mut ingest_complete_paths = Vec::new();
    let mut error_paths = Vec::new();
    let mut other_paths = Vec::new();

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

    ocr_complete_paths.sort_by(|a, b| a.as_path().cmp(b.as_path()));
    ingest_complete_paths.sort_by(|a, b| a.as_path().cmp(b.as_path()));

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

/*  -----------------------------------------------------------------
    Represents the structure of the JSON files that are being parsed.
    Note that the `pid` and `pid_url` fields are not part of the original JSON files; they're populated later.
    -----------------------------------------------------------------
*/
#[derive(Debug, Deserialize, Serialize)]
struct Record {
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
    pid: Option<String>,
    pid_url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct IdToPidInfo {
    id: Option<String>,
    pid: String,
}

/*  -----------------------------------------------------------------
    Parses out `HH001545_0001` from a path like: `/path/to/HH001545/HH001545_0001/HH001545_0001-ingest_complete.json`
    -----------------------------------------------------------------
*/
fn parse_key_from_path(path: &Path) -> String {
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
    Creates a map of id-to-pid.
    -----------------------------------------------------------------
*/
fn make_id_to_pid_map(file_paths: Vec<PathBuf>) -> BTreeMap<String, String> {
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
    log_info!("id_to_pid_map, ``{:#?}``", id_to_pid_map);
    id_to_pid_map
}

/*  -----------------------------------------------------------------
    Processes the JSON files, creating a data-vector.
    -----------------------------------------------------------------
*/
fn process_files(
    file_paths: Vec<PathBuf>,
    id_to_pid_map: &BTreeMap<String, String>,
    output_dir: &str,
) -> io::Result<()> {
    let mut data_vector: Vec<Record> = Vec::new();

    for path_buf in file_paths {
        let path: &Path = path_buf.as_path();
        let key: String = parse_key_from_path(&path);
        let mut file = File::open(&path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let record: JsonResult<Record> = serde_json::from_str(&contents);
        match record {
            Ok(mut rec) => {
                let pid: Option<&String> = id_to_pid_map.get(&key);
                let url: Option<String> = pid
                    .map(|p| format!(" https://repository.library.brown.edu/studio/item/{}/", p));
                rec.pid = pid.cloned();
                rec.pid_url = url;
                data_vector.push(rec);
            }
            Err(e) => log_debug!("Error parsing JSON from {:?}: {}", path, e),
        }
    }

    // After all files have been processed, check if there's any data to append
    if !data_vector.is_empty() {
        log_debug!("saving to CSV");
        save_to_csv(&data_vector, output_dir)?;
    }

    Ok(())
}

/*  -----------------------------------------------------------------
    Saves the data-vector to a CSV file.
    -----------------------------------------------------------------
*/
fn save_to_csv(data: &[Record], output_dir: &str) -> io::Result<()> {
    let file_path = format!("{}/output.csv", output_dir); // Consider more sophisticated file naming
    let file = File::create(file_path)?;
    let mut wtr = csv::Writer::from_writer(file);

    for record in data {
        wtr.serialize(record)?;
    }
    wtr.flush()?;
    Ok(())
}

/*  -----------------------------------------------------------------
    Main function.
    -----------------------------------------------------------------
*/
fn main() {
    // init logger --------------------------------------------------
    logger::init_logger().unwrap();

    // get args -----------------------------------------------------
    let matches = Command::new("parse_ocr_tracker")
        .version(GIT_COMMIT)
        .about("Walks source_dir_path and lists all json files.")
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
    log_debug!("output-arg: {:?}", output_dir);

    // get paths ----------------------------------------------------
    // let (ocr_paths, ingest_paths): (Vec<PathBuf>, Vec<PathBuf>) = find_json_files(source_dir);
    let (ocr_paths, ingest_paths, _error_paths, _other_paths) = find_json_files(source_dir);
    for path in &ocr_paths {
        // pretty-print each path
        log_debug!("{}", path.display());
    }

    // make a map of id-to-pid --------------------------------------
    let id_to_pid_map = make_id_to_pid_map(ingest_paths);

    // process files ------------------------------------------------
    if let Err(e) = process_files(ocr_paths, &id_to_pid_map, &output_dir) {
        log_debug!("Error processing files: {}", e);
    }
}

// let zz: () = the_var;
