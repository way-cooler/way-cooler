//! Contains methods for initializing the lua config

use std::fs::{OpenOptions, File};
use std::env;
use std::path::{Path,PathBuf};

use std::io::Result as IOResult;

pub const INIT_FILE: &'static str = "rc.lua";
pub const INIT_FILE_FALLBACK_PATH: &'static str = "/etc/way-cooler/";

pub const DEFAULT_CONFIG: &'static str = include_str!("../../config/rc.lua");

#[inline]
fn read_file<P: AsRef<Path>>(path: P) -> IOResult<File> {
    OpenOptions::new().read(true).open(path)
}

pub fn get_config() -> Result<(PathBuf, File), &'static str> {
    let home_var = env::var("HOME").expect("HOME environment variable not defined!");
    let home = home_var.as_str();

    if let Ok(path_env) = env::var("WAY_COOLER_INIT_FILE") {
        info!("Found $WAY_COOLER_INIT_FILE to be defined, will look for the init file there.");
        let path = Path::new(&path_env);
        if let Ok(file) = read_file(&path) {
            info!("Reading init file from $WAY_COOLER_INIT_FILE: {}",
                  path_env.as_str().replace(home, "~"));
            // If the parent doesn't exist it's just in the current directory.
            let dir = path.parent().map_or(PathBuf::new(), Path::to_path_buf);
            return Ok((dir, file))
        }
        warn!("Did not find an init file at $WAY_COOLER_INIT_FILE! It points to {}",
              path_env.as_str().replace(home, "~"));
    }

    if let Ok(xdg) = env::var("XDG_CONFIG_HOME") {
        let dir = Path::new(&xdg).join("way-cooler");
        let init_file_path = dir.join(INIT_FILE);
        info!("Found $XDG_CONFIG_DIR to be defined, will look for the init file at $XDG_CONFIG_DIR/way-cooler/init.lua");
        if let Ok(file) = read_file(&init_file_path) {
            info!("Reading init file from $XDG_CONFIG_HOME/way-cooler/init.lua");
            return Ok((dir, file))
        }
        else {
            warn!("Did not find an init file inside $XDG_CONFIG_HOME, no file {}.",
                  &init_file_path.to_string_lossy().replace(home, "~"));
        }
    }
    let dot_config_dir = Path::new(home).join(".config").join("way-cooler");
    let dot_config = dot_config_dir.join(INIT_FILE);

    trace!("Looking for init file at ~/.config/way-cooler/init.lua");
    if let Ok(file) = read_file(&dot_config) {
        info!("Reading init file from ~/.config/way-cooler/init.lua");
        return Ok((dot_config_dir, file))
    } else {
        warn!("No init file found in ~/.config, will default to /etc as a last resort.");
    }

    let etc_config_dir = PathBuf::from(INIT_FILE_FALLBACK_PATH);
    let etc_config = etc_config_dir.join(INIT_FILE);

    if let Ok(file) = read_file(&etc_config) {
        info!("Reading init file from fallback {:?}", &etc_config);
        return Ok((etc_config_dir, file))
    }

    warn!("way-cooler was unable to find an init file. \
           Using our default, pre-compiled init.lua.");
    warn!("Our default init.lua is available on GitHub, \
           it is included with the source.");
    warn!("You may place it in ~/.config/way-cooler/init.lua, \
           or a similar folder if you use $XDG_CONFIG_HOME, or /etc/way-cooler.");
    warn!("You may also use the environment variable $WAY_COOLER_INIT_FILE to point \
           directly to the file of your choice.");

    return Err("Could not find an init file in any path!")
}
