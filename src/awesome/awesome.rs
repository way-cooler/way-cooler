//! TODO Fill in

use nix::{self, libc};
use render;
use gdk_pixbuf::Pixbuf;
use glib::translate::ToGlibPtr;
use std::fmt::{self, Display, Formatter};
use std::process::{Command, Stdio};
use std::thread;
use std::default::Default;
use rlua::{self, Table, Lua, UserData, ToLua, Value};
use super::{GLOBAL_SIGNALS, signal};

// TODO This isn't used yet, but it will be eventually.
// It'll all be "global" values though, so we'll probably
// just store it in Lua (hence the UserData trait)
#[derive(Clone, Debug)]
pub struct AwesomeState {
    // TODO Fill in
    dummy: i32
}

impl Default for AwesomeState {
    fn default() -> Self {
        AwesomeState {
            dummy: 0
        }
    }
}

impl Display for AwesomeState {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Awesome: {:p}", self)
    }
}

impl UserData for AwesomeState {}

pub fn init(lua: &Lua) -> rlua::Result<()> {
    let awesome_table = lua.create_table();
    method_setup(lua, &awesome_table)?;
    property_setup(lua, &awesome_table)?;
    let globals = lua.globals();
    globals.set("awesome", awesome_table)
}

fn method_setup<'lua>(lua: &'lua Lua, awesome_table: &Table<'lua>) -> rlua::Result<()> {
    // TODO Fill in rest
    awesome_table.set("connect_signal", lua.create_function(connect_signal))?;
    awesome_table.set("disconnect_signal", lua.create_function(disconnect_signal))?;
    awesome_table.set("emit_signal", lua.create_function(emit_signal))?;
    awesome_table.set("register_xproperty", lua.create_function(register_xproperty))?;
    awesome_table.set("xkb_get_group_names", lua.create_function(xkb_get_group_names))?;
    awesome_table.set("restart", lua.create_function(restart))?;
    awesome_table.set("load_image", lua.create_function(load_image))?;
    awesome_table.set("exec", lua.create_function(exec))?;
    awesome_table.set("kill", lua.create_function(kill))?;
    awesome_table.set("quit", lua.create_function(quit))
}

fn property_setup<'lua>(lua: &'lua Lua, awesome_table: &Table<'lua>) -> rlua::Result<()> {
    // TODO Do properly
    awesome_table.set("version", "0".to_lua(lua)?)?;
    awesome_table.set("themes_path", "/usr/share/awesome/themes".to_lua(lua)?)?;
    awesome_table.set("conffile", "".to_lua(lua)?)
}

fn connect_signal<'lua>(lua: &'lua Lua, (name, func): (String, rlua::Function<'lua>))
                        -> rlua::Result<()> {
    let global_signals = lua.globals().get::<_, Table>(GLOBAL_SIGNALS)?;
    let fake_object = lua.create_table();
    fake_object.set("signals", global_signals)?;
    signal::connect_signal(lua, fake_object.into(), name, &[func])
}

fn disconnect_signal<'lua>(lua: &'lua Lua, name: String) -> rlua::Result<()> {
    let global_signals = lua.globals().get::<_, Table>(GLOBAL_SIGNALS)?;
    let fake_object = lua.create_table();
    fake_object.set("signals", global_signals)?;
    signal::disconnect_signal(lua, fake_object.into(), name)
}

fn emit_signal<'lua>(lua: &'lua Lua, (name, args): (String, Value))
                     -> rlua::Result<()> {
    let global_signals = lua.globals().get::<_, Table>(GLOBAL_SIGNALS)?;
    let fake_object = lua.create_table();
    fake_object.set("signals", global_signals)?;
    signal::emit_signal(lua, fake_object.into(), name, args)
}

/// Registers a new X property
/// This actually does nothing, since this is Wayland.
fn register_xproperty<'lua>(_: &'lua Lua, _: Value<'lua>) -> rlua::Result<()> {
    warn!("register_xproperty not supported");
    Ok(())
}

/// Get layout short names
fn xkb_get_group_names<'lua>(_: &'lua Lua, _: ()) -> rlua::Result<()> {
    warn!("xkb_get_group_names not supported");
    Ok(())
}

/// Restart Awesome by restarting the Lua thread
fn restart<'lua>(_: &'lua Lua, _: ()) -> rlua::Result<()> {
    use lua::{self, LuaQuery};
    if let Err(err) = lua::send(LuaQuery::Restart) {
        warn!("Could not restart Lua thread {:#?}", err);
    }
    Ok(())
}

/// Load an image from the given path
/// Returns either a cairo surface as light user data, nil and an error message
fn load_image<'lua>(lua: &'lua Lua, file_path: String) -> rlua::Result<Value<'lua>> {
    let pixbuf = Pixbuf::new_from_file(file_path.as_str())
        .map_err(|err| rlua::Error::RuntimeError(format!("{}", err)))?;
    let surface = render::load_surface_from_pixbuf(pixbuf);
    // UGH, I wanted to do to_glib_full, but that isn't defined apparently
    // So now I have to ignore the lifetime completely and just forget about the surface.
    let surface_ptr = surface.to_glib_none().0;
    ::std::mem::forget(surface);
    rlua::LightUserData(surface_ptr as _).to_lua(lua)
}

fn exec(_: &Lua, command: String) -> rlua::Result<()> {
    trace!("exec: \"{}\"", command);
    thread::Builder::new().name(command.clone()).spawn(|| {
        Command::new(command)
            .stdout(Stdio::null())
            .spawn()
            .expect("Could not spawn command")
    }).expect("Unable to spawn thread");
    Ok(())
}

/// Kills a PID with the given signal
///
/// Returns false if it could not send the signal to that process
fn kill(_: &Lua, (pid, sig): (libc::pid_t, libc::c_int)) -> rlua::Result<bool> {
    Ok(nix::sys::signal::kill(pid, sig).is_ok())
}

fn quit(_: &Lua, _: ()) -> rlua::Result<()> {
    ::rustwlc::terminate();
    Ok(())
}
