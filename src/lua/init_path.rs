//! Contains methods for initializing the lua config
#![allow(dead_code)]

use std::fs::{OpenOptions, File};
use std::env;
use std::path::Path;

use std::io::Result as IOResult;

pub const INIT_FILE: &'static str = "init.lua";
pub const INIT_FILE_FALLBACK_PATH: &'static str = "/etc/way-cooler/";

#[inline]
fn read_file<P: AsRef<Path>>(path: P) -> IOResult<File> {
    OpenOptions::new().read(true).open(path)
}

#[cfg(test)]
pub fn get_config() -> (bool, Result<File, &'static str>) {
    (false, Err("Loading config should be ignored during tests for now"))
}

#[cfg(not(test))]
pub fn get_config() -> (bool, Result<File, &'static str>) {
    (true, get_config_file())
}

/// Parses environment variables
fn get_config_file() -> Result<File, &'static str> {
    let home_var = env::var("HOME").expect("HOME environment variable not defined!");
    let home = home_var.as_str();
    if let Ok(path_env) = env::var("WAY_COOLER_INIT_FILE") {
        if let Ok(file) = read_file(Path::new(&path_env)) {
            info!("Reading init file from $WAY_COOLER_INIT_FILE: {}",
                  path_env.as_str().replace(home, "~"));
            return Ok(file)
        }
        warn!("Looking for init file: $WAY_COOLER_INIT_FILE={}, was not a valid path to a config file.",
              path_env.as_str().replace(home, "~"));
    }

    if let Ok(xdg) = env::var("XDG_CONFIG_HOME") {
        let init_file_path = Path::new(&xdg).join("way-cooler").join(INIT_FILE);
        if let Ok(file) = read_file(&init_file_path) {
            info!("Reading init file from $XDG_CONFIG_HOME");
            return Ok(file)
        }
        else {
            warn!("Looking for init file: nothing at $XDG_CONFIG_HOME, no file {}.",
                  &init_file_path.to_string_lossy().replace(home, "~"));
        }
    }
    let dot_config = Path::new(home).join(".config").join("way-cooler").join(INIT_FILE);

    trace!("Looking for init file at {:?}", dot_config.to_string_lossy().replace(home, "~"));
    if let Ok(file) = read_file(&dot_config) {
        info!("Reading init file from {:?}", &dot_config.to_string_lossy().replace(home, "~"));
        return Ok(file)
    }

    let etc_config = Path::new(INIT_FILE_FALLBACK_PATH).join(INIT_FILE);

    if let Ok(file) = read_file(&etc_config) {
        info!("Reading init file from fallback {:?}", &etc_config);
        return Ok(file)
    }

    return Err("Could not find an init file in any path!")
}
