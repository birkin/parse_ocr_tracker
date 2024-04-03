use walkdir::WalkDir;
use std::path::Path;

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
    // Replace "path/to/directory" with the actual path you want to search.
    find_json_files("./");
}


// fn main() {
//     println!("Hello, world!");
// }
