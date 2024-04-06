use std::path::Path;
use walkdir::WalkDir;

use clap::{arg, Command};

fn find_json_files<P: AsRef<Path>>(path: P) {
    for entry in WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file() && e.path().extension().unwrap_or_default() == "json")
    {
        println!("{}", entry.path().display());
    }
}

fn main() {
    // get args -----------------------------------------------------
    let matches = Command::new("MyApp")
        .version("1.0z")
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
    find_json_files("./");
}

// let zz: () = the_var;
