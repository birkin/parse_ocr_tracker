use clap::{arg, Command};
use serde::{Deserialize, Serialize};
use serde_json::Result as JsonResult;
// use std::io::{Seek, SeekFrom};
use std::{
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

/*
    Includes the file generated by the build.rs script, which looks like:
    pub const GIT_COMMIT: &str = "c5f7034f79bc3d49c1a9fb81c7cac6a8a778c5c3";
*/
include!(concat!(env!("OUT_DIR"), "/git_commit.rs")); // OUT_DIR is set by cargo; is the target dir; and is only available during build process

// fn find_json_files<P: AsRef<Path>>(path: P) -> Vec<PathBuf> {
//     let mut paths = Vec::new();
//     for entry in WalkDir::new(path)
//         .into_iter()
//         .filter_map(|e| e.ok())
//         .filter(|e| e.path().is_file() && e.path().extension().unwrap_or_default() == "json")
//     {
//         paths.push(entry.into_path());
//     }
//     paths.sort_by(|a, b| a.as_path().cmp(b.as_path())); // sort uses `cmp` for safe comparison without assuming UTF-8 encoding.
//                                                         // for path in &paths {
//                                                         //     // pretty-print each path
//                                                         //     println!("{}", path.display());
//                                                         // }
//     println!("len-paths: {}", paths.len());
//     paths
// }

// fn find_specific_json_files<P: AsRef<Path>>(path: P) -> (Vec<PathBuf>, Vec<PathBuf>) {
// fn find_json_files<P: AsRef<Path>>(path: P) -> Vec<PathBuf> {
fn find_json_files<P: AsRef<Path>>(path: P) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let mut ocr_complete_paths = Vec::new();
    let mut ingest_complete_paths = Vec::new();

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
            } else {
                println!("Ignoring file: {}", file_name);
            }
        }
    }

    ocr_complete_paths.sort_by(|a, b| a.as_path().cmp(b.as_path()));
    ingest_complete_paths.sort_by(|a, b| a.as_path().cmp(b.as_path()));

    println!("len-ocr_complete_paths: {}", ocr_complete_paths.len());
    println!("len-ingest_complete_paths: {}", ingest_complete_paths.len());

    (ocr_complete_paths, ingest_complete_paths)
    // ocr_complete_paths
}

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

// fn process_files(file_paths: Vec<PathBuf>, output_dir: &str) -> io::Result<()> {
//     let mut data_vector: Vec<Record> = Vec::new();

//     for path_buf in file_paths {
//         let path = path_buf.as_path();
//         let mut file = File::open(&path)?;
//         let mut contents = String::new();
//         file.read_to_string(&mut contents)?;

//         // Deserialize the JSON data
//         let record: JsonResult<Record> = serde_json::from_str(&contents);
//         match record {
//             Ok(rec) => {
//                 data_vector.push(rec);
//                 // When the vector has a length of 100 items, append to a CSV and clear the vector
//                 if data_vector.len() >= 100 {
//                     println!("appending to CSV");
//                     append_to_csv(&data_vector, output_dir)?;
//                     data_vector.clear();
//                 }
//             }
//             Err(e) => println!("Error parsing JSON from {:?}: {}", path, e),
//         }
//     }

//     // Append any remaining data to the CSV
//     if !data_vector.is_empty() {
//         append_to_csv(&data_vector, output_dir)?;
//     }

//     Ok(())
// }

fn process_files(file_paths: Vec<PathBuf>, output_dir: &str) -> io::Result<()> {
    let mut data_vector: Vec<Record> = Vec::new();

    for path_buf in file_paths {
        let path = path_buf.as_path();
        let mut file = File::open(&path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let record: JsonResult<Record> = serde_json::from_str(&contents);
        match record {
            Ok(rec) => {
                data_vector.push(rec);
            }
            Err(e) => println!("Error parsing JSON from {:?}: {}", path, e),
        }
    }

    // After all files have been processed, check if there's any data to append
    if !data_vector.is_empty() {
        println!("saving to CSV");
        save_to_csv(&data_vector, output_dir)?;
    }

    Ok(())
}

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

// fn append_to_csv(data: &[Record], output_dir: &str) -> io::Result<()> {
//     let file_path = format!("{}/output.csv", output_dir);
//     let file = OpenOptions::new()
//         .write(true)
//         .create(true)
//         .append(true)
//         .open(Path::new(&file_path))?;
//     let mut wtr = csv::Writer::from_writer(file);

//     for record in data {
//         wtr.serialize(record)?;
//     }

//     wtr.flush()?;
//     Ok(())
// }

fn main() {
    // get args -----------------------------------------------------
    let matches = Command::new("parse_ocr_tracker")
        // .version("1.0z")
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
    println!("source-arg: {:?}", source_dir);

    // get output_dir -----------------------------------------------
    let output_dir_temp_ref: &String = matches
        .get_one::<String>("output_dir_path")
        .expect("Failed to get required 'output_dir_path' argument.");
    let output_dir: &str = output_dir_temp_ref.as_str();
    println!("output-arg: {:?}", output_dir);

    // get paths ----------------------------------------------------
    // let paths_vector: Vec<PathBuf> = find_json_files(source_dir);
    let (ocr_paths, ingest_paths): (Vec<PathBuf>, Vec<PathBuf>) = find_json_files(source_dir);
    println!("ocr_paths.len(): {}", ocr_paths.len());
    println!("ingest_paths.len(): {}", ingest_paths.len());
    for path in &ocr_paths {
        // pretty-print each path
        println!("{}", path.display());
    }

    // make a map of id-to-pid --------------------------------------

    // process files ------------------------------------------------
    if let Err(e) = process_files(ocr_paths, &output_dir) {
        eprintln!("Error processing files: {}", e);
    }
}

// let zz: () = the_var;
