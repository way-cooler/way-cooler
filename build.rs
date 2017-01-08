use std::env;
use std::process::Command;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    dump_git_version();
}

/// Writes the current git hash to a file that is read by Way Cooler
///
/// If this is a release build, the file will be empty.
fn dump_git_version() {
    if let Some(git_version) = git_version() {
        let out_dir = env::var("OUT_DIR")
            .expect("Could not find out directory!");
        let dest_path = Path::new(&out_dir).join("git-version.txt");
        let mut f = File::create(&dest_path)
            .expect("Could not write git version to out directory");

        f.write_all(git_version.as_ref())
            .expect("Could not write to git version file");
    }
}

/// Gets the current hash of HEAD. Returns None if for some reason
/// that could not be retrieved (e.g not in a git repository)
fn git_version() -> Option<String> {
    Command::new("git")
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .ok()
        .map(|output| output.stdout)
        .map(|hash| String::from_utf8_lossy(&hash).trim().into())
}
