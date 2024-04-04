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
    // get args
    let matches = Command::new("MyApp")
        .version("1.0")
        .about("Does awesome things")
        .arg(arg!(--two <VALUE>).required(true))
        .arg(arg!(--one <VALUE>).required(true))
        .get_matches();

    println!(
        "two: {:?}",
        matches.get_one::<String>("two").expect("required")
    );
    println!(
        "one: {:?}",
        matches.get_one::<String>("one").expect("required")
    );

    // Replace "path/to/directory" with the actual path you want to search.
    find_json_files("./");
}

// fn main() {
//     println!("Hello, world!");
// }
