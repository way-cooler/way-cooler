extern crate wayland_scanner;

use wayland_scanner::{generate_code, generate_interfaces, Side};

use std::{env, fs, io::Write, path::{Path, PathBuf}, process::Command};

fn main() {
    dump_git_version();
    generate_wayland_protocols();
}

/// Writes the current git hash to a file that is read by Way Cooler
///
/// If this is a release build, the file will be empty.
fn dump_git_version() {
    let out_dir = env::var("OUT_DIR").expect("Could not find out directory!");
    let dest_path = Path::new(&out_dir).join("git-version.txt");
    let mut f = fs::File::create(&dest_path).expect("Could not write git version to out directory");
    if let Some(git_version) = git_version() {
        f.write_all(git_version.as_ref()).expect("Could not write to git version file");
    }
}

/// Gets the current hash of HEAD. Returns None if for some reason
/// that could not be retrieved (e.g not in a git repository)
fn git_version() -> Option<String> {
    if !in_release_commit() {
        Command::new("git").arg("rev-parse")
                           .arg("HEAD")
                           .output()
                           .ok()
                           .map(|output| output.stdout)
                           .map(|hash| String::from_utf8_lossy(&hash).trim().into())
    } else {
        None
    }
}

/// Determines if the current HEAD is tagged with a release
fn in_release_commit() -> bool {
    let result = Command::new("git").arg("describe")
                                    .arg("--exact-match")
                                    .arg("--tags")
                                    .arg("HEAD")
                                    .output()
                                    .unwrap();
    result.status.success()
}

fn generate_wayland_protocols() {
    let protocols = fs::read_dir("./protocols").expect("No <Way Cooler>/protocols/ directory");
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);

    for protocol_path in protocols {
        let protocol_path: fs::DirEntry = protocol_path.unwrap();
        let path: PathBuf = protocol_path.path().into();
        let mut file_name: String = protocol_path.file_name().into_string().unwrap();
        if let Some(extension) = file_name.find(".xml") {
            file_name.truncate(extension);
        }
        generate_code(path.clone(),
                      out_dir.join(file_name.clone() + "_api.rs"),
                      Side::Server);
        generate_interfaces(path, out_dir.join(file_name + "_interface.rs"));
    }
}
