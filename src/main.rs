// use std::path::Path;
// use walkdir::WalkDir;

use clap::{arg, Command};

// fn find_json_files<P: AsRef<Path>>(path: P) {
//     for entry in WalkDir::new(path)
//         .into_iter()
//         .filter_map(|e| e.ok())
//         .filter(|e| e.path().is_file() && e.path().extension().unwrap_or_default() == "json")
//     {
//         println!("{}", entry.path().display());
//     }
// }

fn main() {
    // get args -----------------------------------------------------
    let matches = Command::new("MyApp")
        .version("1.0z")
        .about("Walks source_dir_path and lists all json files.")
        .arg(arg!(-s --source_dir_path <VALUE>).required(true)) 
        .arg(arg!(-o --output_dir_path <VALUE>).required(true))
        .get_matches();

    println!(
        "source-arg: {:?}",
        matches.get_one::<String>("source_dir_path").expect("required")
    );
    println!(
        "output-arg: {:?}",
        matches.get_one::<String>("output_dir_path").expect("required")
    );

    // Replace "path/to/directory" with the actual path you want to search.
    // find_json_files("./");
}

// fn main() {
//     println!("Hello, world!");
// }
