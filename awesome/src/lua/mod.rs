//! Lua interpreter setup and configuration reading.

mod config;
mod utils;

use self::config::{get_config, load_config};
pub use self::{config::log_error, utils::*};

use clap::ArgMatches;
use glib::MainLoop;
use rlua::{self, AnyUserData, Lua, Table, Value};

use std::{
    cell::{Cell, RefCell},
    fs::File,
    io::{self, Read},
    path::PathBuf
};

use crate::common::signal;

/// Path to the Awesome shims.
const SHIMS_PATH: &str = "../../tests/awesome/tests/examples/shims/";
/// Shims to load
const SHIMS: [&str; 11] = [
    "awesome",
    "root",
    "tag",
    "screen",
    "client",
    "mouse",
    "drawin",
    "button",
    "keygrabber",
    "mousegrabber",
    "key"
];

thread_local! {
    // NOTE The debug library does some powerful reflection that can do crazy things,
    // which is why it's unsafe to load.

    /// Global Lua state.
    pub static LUA: RefCell<Lua> = RefCell::new(unsafe { Lua::new_with_debug() });

    /// If set then we have restarted the Lua thread. We need to replace LUA when it's not borrowed.
    pub static NEXT_LUA: Cell<bool> = Cell::new(false);

    /// Main GLib loop
    static MAIN_LOOP: RefCell<MainLoop> = RefCell::new(MainLoop::new(None, false));
}

/// Loads shim code to act like Awesome.
///
/// To be compatible this must eventually be removed.
///
/// Best way to help out: comment out one of these lines, fix what breaks.
fn load_shims(lua: &Lua) {
    let globals = lua.globals();
    let package: Table = globals.get("package").unwrap();
    let mut path = package.get::<_, String>("path").unwrap();
    let shims_path: PathBuf = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join(SHIMS_PATH);
    path.push_str(&format!(
        ";{0}/?.lua;{0}/?/init.lua",
        shims_path.to_str().unwrap()
    ));
    package.set("path", path).unwrap();
    for shim in SHIMS.iter() {
        let mut path = shims_path.clone();
        path.push(format!("{}.lua", shim));
        let shims_path_str = path.to_str().unwrap();
        let mut file = File::open(path.clone()).expect(&format!("Could not open {}", shims_path_str));
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect(&format!("Could not read {}", shims_path_str));
        let obj = lua
            .exec::<rlua::Value>(contents.as_str(), None)
            .expect(&format!("Could not read {}", shims_path_str));
        globals.set(*shim, obj).expect("Could not set global object");
    }
}

/// Sets up the Lua environment for the user code.
///
/// This environment is also necessary for some Wayland callbacks,
/// so it should be called as soon as possible.
pub fn init_awesome_libraries(lib_paths: &[&str]) {
    info!("Setting up Awesome libraries");
    LUA.with(|lua| {
        register_libraries(&*lua.borrow(), lib_paths).expect("Could not register lua libraries");
    });
}

/// Runs user code from the config through the awesome compatibility layer.
///
/// It then enters the glib/wayland main loop to listen for events.
pub fn run_awesome(matches: ArgMatches) {
    LUA.with(|lua| {
        let mut lua = lua.borrow_mut();
        info!("Loading Awesome configuration...");
        let lib_paths = matches
            .values_of("lua lib search")
            .unwrap_or_default()
            .collect::<Vec<_>>();
        let config = matches.value_of("config");
        load_config(&mut *lua, config, lib_paths.as_slice());
        load_shims(&mut *lua);
    });
    enter_glib_loop();
}

fn emit_refresh(lua: &Lua) {
    if let Err(err) = signal::global_emit_signal(lua, ("refresh".to_owned(), Value::Nil)) {
        error!("Internal error while emitting 'refresh' signal: {}", err);
    }
}

/// Main loop of the Lua thread:
///
/// * Initialise the Lua state
/// * Run a GMainLoop
pub fn enter_glib_loop() {
    MAIN_LOOP.with(|main_loop| main_loop.borrow().run());
}

pub fn terminate() {
    MAIN_LOOP.with(|main_loop| main_loop.borrow().quit())
}

/// Checks that the first configuration file used is syntactically correct.
///
/// If the inner io::Result is not `Ok(())` we could not read any config files.
pub fn syntax_check(cmdline_path: Option<&str>) -> Result<(), io::Result<rlua::Error>> {
    let (path, mut file) = get_config(cmdline_path).map_err(Err)?;
    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents).map_err(Err)?;
    LUA.with(|lua| {
        let lua = lua.borrow();
        let res = lua
            .load(file_contents.as_str(), path.to_str())
            .map(|_| ())
            .map_err(|err| Ok(err));
        res
    })
}

/// Register all the Rust functions for the lua libraries
pub fn register_libraries(lua: &Lua, lib_paths: &[&str]) -> rlua::Result<()> {
    trace!("Setting up Lua libraries");
    // TODO Is this awesome init code necessary?
    let init_code = include_str!("../../../lib/lua/init.lua");
    lua.exec::<()>(init_code, Some("init.lua"))?;
    let globals = lua.globals();
    globals.set("type", lua.create_function(type_override)?)?;
    init_libs(&lua, lib_paths).expect("Could not initialize awesome compatibility modules");
    Ok(())
}

fn init_libs(lua: &Lua, lib_paths: &[&str]) -> rlua::Result<()> {
    use crate::objects::*;
    use crate::*;
    setup_awesome_path(lua, lib_paths)?;
    setup_global_signals(lua)?;
    setup_xcb_connection(lua)?;
    button::init(lua)?;
    awesome::init(lua)?;
    key::init(lua)?;
    client::init(lua)?;
    screen::init(lua)?;
    keygrabber::init(lua)?;
    root::init(lua)?;
    mouse::init(lua)?;
    tag::init(lua)?;
    drawin::init(lua)?;
    drawable::init(lua)?;
    mousegrabber::init(lua)?;
    dbus::lua_init(lua)?;
    Ok(())
}

/// This function behaves just like Lua's built-in type() function, but also
/// recognises classes and returns special names for them.
fn type_override(_lua: &Lua, arg: Value) -> rlua::Result<String> {
    // Lua's type() returns the result of lua_typename(), but rlua does not make
    // that available to us, so write our own.
    Ok(match arg {
        Value::Error(e) => return Err(e),
        Value::Nil => "nil",
        Value::Boolean(_) => "boolean",
        Value::LightUserData(_) => "userdata",
        Value::Integer(_) => "number",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Function(_) => "function",
        Value::Thread(_) => "thread",
        Value::Table(_) => "table",
        Value::UserData(o) => {
            // Handle our own objects specially: Get the object's class from its user
            // value's metatable's __class entry. Then get the class name
            // from the class's user value's metatable's name entry.
            return o
                .get_user_value::<Table>()
                .ok()
                .and_then(|table| table.get_metatable())
                .and_then(|meta| meta.raw_get::<_, AnyUserData>("__class").ok())
                .and_then(|class| class.get_user_value::<Table>().ok())
                .map(|table| table.raw_get("name"))
                .unwrap_or_else(|| Ok("userdata".into()));
        }
    }
    .into())
}
