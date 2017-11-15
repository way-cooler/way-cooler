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
use super::signal;

#[derive(Clone, Debug)]
pub struct AwesomeState {
    preferred_icon_size: u32
}

impl Default for AwesomeState {
    fn default() -> Self {
        AwesomeState {
            preferred_icon_size: 0
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
    state_setup(lua, &awesome_table)?;
    meta_setup(lua, &awesome_table)?;
    method_setup(lua, &awesome_table)?;
    property_setup(lua, &awesome_table)?;
    let globals = lua.globals();
    globals.set("awesome", awesome_table)
}

fn state_setup(lua: &Lua, awesome_table: &Table) -> rlua::Result<()> {
    awesome_table.set("__data", AwesomeState::default().to_lua(lua)?)
}

fn meta_setup(lua: &Lua, awesome_table: &Table) -> rlua::Result<()> {
    let meta_table = awesome_table.get_metatable().unwrap_or_else(|| {
        let table = lua.create_table();
        awesome_table.set_metatable(Some(table.clone()));
        table
    });
    meta_table.set("__tostring", lua.create_function(|_, val: Table| {
        Ok(format!("{}", val.get::<_, AwesomeState>("__data")?))
    }))
}

fn method_setup<'lua>(lua: &'lua Lua, awesome_table: &Table<'lua>) -> rlua::Result<()> {
    // TODO Fill in rest
    awesome_table.set("connect_signal", lua.create_function(signal::global_connect_signal))?;
    awesome_table.set("disconnect_signal", lua.create_function(signal::global_disconnect_signal))?;
    awesome_table.set("emit_signal", lua.create_function(signal::global_emit_signal))?;
    awesome_table.set("xrdb_get_value", lua.create_function(xrdb_get_value))?;
    awesome_table.set("xkb_set_layout_group", lua.create_function(xkb_set_layout_group))?;
    awesome_table.set("xkb_get_layout_group", lua.create_function(xkb_get_layout_group))?;
    awesome_table.set("set_preferred_icon_size", lua.create_function(set_preferred_icon_size))?;
    awesome_table.set("register_xproperty", lua.create_function(register_xproperty))?;
    awesome_table.set("xkb_get_group_names", lua.create_function(xkb_get_group_names))?;
    awesome_table.set("set_xproperty", lua.create_function(set_xproperty))?;
    awesome_table.set("get_xproperty", lua.create_function(get_xproperty))?;
    awesome_table.set("restart", lua.create_function(restart))?;
    awesome_table.set("load_image", lua.create_function(load_image))?;
    awesome_table.set("sync", lua.create_function(sync))?;
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

/// Registers a new X property
/// This actually does nothing, since this is Wayland.
fn register_xproperty<'lua>(_: &'lua Lua, _: Value<'lua>) -> rlua::Result<()> {
    warn!("register_xproperty not supported");
    Ok(())
}

/// Get layout short names
fn xkb_get_group_names<'lua>(_: &'lua Lua, _: ()) -> rlua::Result<String> {
    use std::process::{Stdio, Command};
    // TODO FIXME wow this is _really_ _really_ dumb.
    // like really dumb
    // but not sure how else to do this to communicate to xwayland...
    warn!("xkb_get_group_names not supported");
    warn!("Doing a bad hack now");
    warn!("Fix me before release!");
    // This should be a function, not a whole freaking command!
    Ok(Command::new("/usr/bin/setxkbmap").arg("-v").arg("-query")
        .stdout(Stdio::piped())
        .spawn()
        .map(|child| {
            child.wait_with_output().ok().and_then(|output| {
                let stdout = String::from_utf8(output.stdout)
                    .expect("Output was not utf8");
                for line in stdout.lines() {
                    if line.contains("symbols:") {
                        let last = line.split(" ").last().unwrap();
                        return Some(last.trim().into())
                    }
                }
                None
            })
        }).unwrap_or_else(|err| {
            warn!("err: {:#?}", err);
            Some("pc+us+inet(evdev)".into())
        }).unwrap())
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

fn set_preferred_icon_size(lua: &Lua, val: u32) -> rlua::Result<()> {
    let mut awesome_state: AwesomeState = lua.globals().get::<_, Table>("awesome")?.get("__data")?;
    awesome_state.preferred_icon_size = val;
    lua.globals().get::<_, Table>("awesome")?.set("__data", awesome_state.to_lua(lua)?)

}

fn quit(_: &Lua, _: ()) -> rlua::Result<()> {
    ::rustwlc::terminate();
    Ok(())
}

/// No need to sync in Wayland
fn sync(_: &Lua, _: ()) -> rlua::Result<()> { Ok(()) }


fn set_xproperty(_: &Lua, _: Value) -> rlua::Result<()> {
    warn!("set_xproperty not supported");
    Ok(())
}

fn get_xproperty(_: &Lua, _: Value) -> rlua::Result<()> {
    warn!("get_xproperty not supported");
    Ok(())
}

fn xkb_set_layout_group(_: &Lua, _group: i32) -> rlua::Result<()> {
    warn!("xkb_set_layout_group not supported; Wait until wlroots");
    Ok(())
}

fn xkb_get_layout_group(_: &Lua, _: ()) -> rlua::Result<i32> {
    warn!("xkb_get_layout_group not supported; Wait until wlroots");
    warn!("Returning dummy data so I can move forward");
    warn!("Replace me before release!");
    Ok(0)
}

fn xrdb_get_value(lua: &Lua, (_resource_class, resource_name): (String, String))
                  -> rlua::Result<Value> {
    if &resource_name == "Xft.dpi" {
        return Ok("1".to_lua(lua)?)
    }
    warn!("xrdb_get_value not supported");
    Ok(Value::Nil)
}
