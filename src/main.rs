use clap::{arg, Command};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/*
    Includes the file generated by the build.rs script, which looks like:
    pub const GIT_COMMIT: &str = "c5f7034f79bc3d49c1a9fb81c7cac6a8a778c5c3";
*/
include!(concat!(env!("OUT_DIR"), "/git_commit.rs")); // OUT_DIR is set by cargo; is the target dir; and is only available during build process

fn find_json_files<P: AsRef<Path>>(path: P) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for entry in WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file() && e.path().extension().unwrap_or_default() == "json")
    {
        paths.push(entry.into_path());
    }
    println!("paths: {:?}", paths);
    paths
}

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
    // call worker function -----------------------------------------
    find_json_files(source_dir);
}

// let zz: () = the_var;
