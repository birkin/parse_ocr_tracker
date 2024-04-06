use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("start build.rs");
    // Get the current git commit hash
    let output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .expect("Failed to execute git command");

    let git_hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let version_string: String = format!("version-{}", git_hash);

    // Write the git hash to a file that can be included in the main binary
    let out_dir: String = env::var("OUT_DIR").expect("Failed to read OUT_DIR environment variable");  // OUT_DIR is a cargo environment variable that points to the target directory, and is only available during the build process
    let dest_path = Path::new(&out_dir).join("git_commit.rs");
    let mut f = File::create(&dest_path).expect("failed to create git_commit.rs file");

    // Write the git commit hash to the file
    writeln!(f, "pub const GIT_COMMIT: &str = \"{}\";", version_string)
        .expect("failed to write to git_commit.rs file");
    println!("leaving build.rs");
}
