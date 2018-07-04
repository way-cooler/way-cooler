//! Contains methods for initializing the lua config

use std::env;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};

const INIT_FILE: &'static str = "rc.lua";
const INIT_FILE_FALLBACK_PATH: &'static str = "/etc/way-cooler/";

pub const DEFAULT_CONFIG: &'static str = include_str!("../../../config/rc.lua");

pub fn get_config() -> Option<(PathBuf, File)> {
    let home_var = env::var("HOME").expect("HOME environment variable not defined!");
    let home = home_var.as_str();

    let mut paths: [Option<PathBuf>; 4] = [None,
                                           None,
                                           Some(Path::new(home).join(".config")
                                                               .join("way-cooler")),
                                           Some(INIT_FILE_FALLBACK_PATH.into())];

    if let Ok(path) = env::var("WAY_COOLER_INIT_FILE").map(|path| PathBuf::from(path)) {
        let path = path.parent().map_or(PathBuf::new(), Path::to_path_buf);
        paths[0] = Some(path);
    }

    if let Ok(path) = env::var("XDG_CONFIG_HOME").map(|path| PathBuf::from(path)) {
        paths[1] = Some(path);
    }

    for path in paths.iter_mut() {
        let (original_path, path) = match path.take() {
            Some(path) => (path.clone(), path.join(INIT_FILE)),
            None => continue
        };
        if let Ok(file) = OpenOptions::new().read(true).open(path.clone()) {
            info!("Found init file @ {:?}", path);
            return Some((original_path, file))
        }
    }
    return None
}
