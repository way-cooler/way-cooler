use std::fs;
use std::env;

//! Contains methods for initializing the lua config

pub const INIT_FILE_DEFAULT_PATH: &'static str = ""
pub const INIT_FILE_FALLBACK_PATH: &'static str = "/etc/way-cooler/init.lua";

fn file_exists() {
    
}

pub fn get_config_path() -> Result<Path, &'static str> {
    if let Ok(path_env) = env::var("WAY_COOLER_INIT_FILE") {
        info!("Reading init file from $WAY_COOLER: {}", path_env);
        Ok(Path::new(path_env))
    }
    else { // Use predestined path
        
    }
}
