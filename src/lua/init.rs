use std::fs::{self, OpenOptions};
use std::env;
use std::path::Path;

use std::io::Result as IOResult;

//! Contains methods for initializing the lua config

pub const INIT_FILE: &'static str = "init.lua";
pub const INIT_FILE_FALLBACK_PATH: &'static str = "/etc/way-cooler/";

pub fn home_config_path() -> Path {
    Path::new(env::var("HOME").expect("Unable to read HOME variable"))
        .join(".config")
        .join("way-cooler")
        .join(INIT_FILE)
}

#[inline]
fn read_file<P: AsRef<Path>>(path: P) -> IOResult<File> {
    OpenOptions::new().read(true).open(path)
}

/// Parses environment variables
pub fn get_config_path() -> Result<File, &'static str> {
    if let Ok(path_env) = env::var("WAY_COOLER_CONFIG") {
        if let Some(file) = read_file(Path::new(path_env).join(INIT_FILE)) {
            info!("Reading init file from $WAY_COOLER_INIT_FILE: {}", path_env);
            return Ok(file)
        }
        warn!("Error reading from $WAY_COOLER_INIT_FILE, defaulting...");
    }

    if let Ok(xdg) = env::var("XDG_CONFIG_HOME") {
        if let Some(file) = read_file(path_env) {
            info!("Reading init file from $XDG_CONFIG_HOME: {}", xdg);
            return Ok(Path::new(xdg).join("way-cooler").join(INIT_FILE))
        }
        warn!("Error reading from $XDG_CONFIG_HOME, defaulting...")
    }
    let dot_config = Path::new(env::var("HOME")
                               .expect("HOME environment variable not defined!"))
        .join(".config").join("way-cooler").join(INIT_FILE);

    if let Some(file) = read_file(dot_config) {
        info!("Reading init file from {}", dot_config);
        return Ok(file)
    }

    let etc_config = Path::new(INIT_FILE_FALLBACK_PATH).join(INIT_FILE);

    if let Some(file) = read_file(etc_config) {
        info!("Reading init file from fallback {}", etc_config);
        return Ok(file)
    }

    return Err("Could not find an init file in any path!")
}
