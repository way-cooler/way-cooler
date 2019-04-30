//! Contains methods for initializing the lua config

use std::{
    env,
    ffi::OsStr,
    fs::{File, OpenOptions},
    io::{self, Read},
    path::{Path, PathBuf},
    sync::Arc
};

use rlua::{self, Context, Table};

const INIT_FILE: &'static str = "rc.lua";
const INIT_FILE_FALLBACK_PATH: &'static str = "/etc/way-cooler/rc.lua";

pub const DEFAULT_CONFIG: &'static str = include_str!("../../../config/rc.lua");

/// Reads the init file and returns it's contents
pub fn load_config(lua: Context, cmdline_config: Option<&str>) -> io::Result<(String, String)> {
    let (init_path, mut init_file) = get_config(cmdline_config)?;
    if let Some(init_dir) = init_path.parent() {
        // Add the config directory to the package path.
        let globals = lua.globals();
        let package: Table = globals.get("package").expect("package not defined in Lua");
        let paths: String = package.get("path").expect("package.path not defined in Lua");
        let init_dir = init_dir
            .join("?.lua")
            .into_os_string()
            .into_string()
            .expect("init_dir not a valid UTF-8 string");
        trace!("Adding location of init file to package.path: {}", init_dir);
        package
            .set("path", format!("{};{}", paths, init_dir))
            .expect("Failed to set package.path");
    }
    let mut init_contents = String::new();
    init_file.read_to_string(&mut init_contents)?;
    let file_name = init_path.file_name().and_then(OsStr::to_str);

    Ok((file_name.unwrap_or(INIT_FILE).to_string(), init_contents))
}

pub fn exec_config(lua: rlua::Context, file_name: &str, content: &str) -> rlua::Result<()> {
    lua.load(content).set_name(file_name).unwrap().exec()?;
    crate::lua::emit_refresh(lua);

    Ok(())
}

/// Finds the configuration file in predefined locations and opens it for reading.
///
/// Returns the found path and the opened handle.
fn get_config(cmdline_path: Option<&str>) -> io::Result<(PathBuf, File)> {
    let cmdline_path = cmdline_path.map(PathBuf::from);
    let home_var = env::var("HOME").expect("HOME environment variable not defined!");
    let home = home_var.as_str();

    let mut paths: [Option<PathBuf>; 5] = [
        cmdline_path,
        None,
        None,
        Some(Path::new(home).join(".config").join("way-cooler").join(INIT_FILE)),
        Some(INIT_FILE_FALLBACK_PATH.into())
    ];

    paths[1] = env::var("WAY_COOLER_INIT_FILE").ok().map(PathBuf::from);

    paths[2] = env::var("XDG_CONFIG_HOME").ok().map(|path| {
        let mut path = PathBuf::from(path);
        path.push(INIT_FILE);
        path
    });

    for path in paths.into_iter() {
        let path = match path {
            Some(path) => path,
            None => continue
        };
        if let Ok(file) = OpenOptions::new().read(true).open(path) {
            info!("Found init file @ {:?}", path);
            return Ok((path.clone(), file));
        }
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "No configuration file found"
    ))
}

pub fn log_error(err: rlua::Error) {
    fn recursive_callback_print(error: Arc<rlua::Error>) {
        match *error {
            rlua::Error::CallbackError {
                traceback: ref err,
                ref cause
            } => {
                error!("{}", err);
                recursive_callback_print(cause.clone())
            },
            ref err => error!("{:?}", err)
        }
    }
    match err {
        rlua::Error::RuntimeError(ref err) => {
            error!("{}", err);
        },
        rlua::Error::CallbackError {
            traceback: ref err,
            ref cause
        } => {
            error!("traceback: {}", err);
            recursive_callback_print(cause.clone());
        },
        err => {
            error!("lua error: {}", err);
        }
    }
}
