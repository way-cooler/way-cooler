extern crate cc;
extern crate pkg_config;
extern crate wayland_scanner;

use std::{env, fs, io::Write, path::{Path, PathBuf}, process::Command};

const PROTOCOL_PATH: &'static str = "../protocols";

fn main() {
    dump_git_version();
    build_wayland_glib_interface();
    build_wayland_protcols();
}

/// Writes the current git hash to a file that is read by Way Cooler
///
/// If this is a release build, the file will be empty.
fn dump_git_version() {
    let out_dir = env::var("OUT_DIR").expect("Could not find out directory!");
    let dest_path = Path::new(&out_dir).join("git-version.txt");
    let mut f = fs::File::create(&dest_path)
        .expect("Could not write git version to out directory");
    if let Some(git_version) = git_version() {
        f.write_all(git_version.as_ref())
            .expect("Could not write to git version file");
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

/// Build the wayland-glib interface as a static library
fn build_wayland_glib_interface() {
    let glib = pkg_config::probe_library("glib-2.0").unwrap();
    let dbus = pkg_config::probe_library("dbus-1").unwrap();
    let mut builder = cc::Build::new();

    let include_paths = glib.include_paths.iter()
        .chain(dbus.include_paths.iter());

    for path in include_paths  {
        builder.include(path);
    }

    builder.file("src/wayland_glib_interface.c")
        .compile("wayland_glib_interface");
}

/// Build the wayland protcols that Awesome will use to talk to Way Cooler.
fn build_wayland_protcols() {
    let protocols = fs::read_dir(PROTOCOL_PATH)
        .expect("Protocol build path invalid.");
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);
    for protocol_path in protocols {
        let protocol_path: fs::DirEntry = protocol_path.unwrap();
        let path: PathBuf = protocol_path.path().into();
        let mut file_name: String = protocol_path
            .file_name().into_string().unwrap();
        if let Some(extension) = file_name.find(".xml") {
            file_name.truncate(extension);
        }
        wayland_scanner::generate_c_code(path.clone(),
                      out_dir.join(file_name.clone() + "_api.rs"),
                      wayland_scanner::Side::Client);
        wayland_scanner::generate_c_interfaces(path,
                            out_dir.join(file_name + "_interface.rs"));
    }
}
