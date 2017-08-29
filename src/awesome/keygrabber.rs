//! AwesomeWM Keygrabber interface

use ::lua::LUA;
use rlua::{self, Lua, Table, Function, Value};
use rustwlc::*;
#[allow(deprecated)]
use rustwlc::xkb::Keysym;

pub const KEYGRABBER_TABLE: &str = "keygrabber";
const SECRET_CALLBACK: &str = "__callback";

/// Init the methods defined on this interface.
pub fn init(lua: &Lua) -> rlua::Result<()> {
    let keygrabber_table = lua.create_table();
    keygrabber_table.set("run", lua.create_function(run))?;
    keygrabber_table.set("stop", lua.create_function(stop))?;
    keygrabber_table.set("isrunning", lua.create_function(isrunning))?;
    let globals = lua.globals();
    globals.set(KEYGRABBER_TABLE, keygrabber_table)
}

#[allow(deprecated)]
/// Given the current input, handle calling the Lua defined callback if it is
/// defined with the input.
pub fn keygrabber_handle(mods: KeyboardModifiers, sym: Keysym, state: KeyState)
                         -> rlua::Result<()> {
    let lua = LUA.lock().expect("LUA was poisoned");
    let lua_state = if state == KeyState::Pressed {
        "press"
    } else {
        "release"
    }.into();
    let lua_sym = sym.get_name().expect("Key symbol did not have a defined name");
    let lua_mods = ::lua::mods_to_lua(&lua.0, mods.mods)
        .expect("Could not convert mods to lua representation");
    let res = call_keygrabber(&lua.0, (lua_mods, lua_sym, lua_state));
    match res {
        Ok(_) | Err(rlua::Error::FromLuaConversionError { .. }) => {Ok(())},
        err => {
            err
        }
    }

}

/// Call the Lua callback function for when a key is pressed.
fn call_keygrabber(lua: &Lua,
                   (mods, key, event): (Table, String, String))
                   -> rlua::Result<()> {
    let globals = lua.globals();
    let lua_callback = globals
        .get::<_, Table>(KEYGRABBER_TABLE).expect("keygrabber table not defined")
        .get::<_, Function>(SECRET_CALLBACK)?;
    lua_callback.call((mods, key, event))
}

fn run(lua: &Lua, function: rlua::Function) -> rlua::Result<()> {
    let keygrabber_table = lua.globals().get::<_, Table>(KEYGRABBER_TABLE)?;
    match keygrabber_table.get::<_, Value>(SECRET_CALLBACK)? {
        Value::Function(_) =>
            Err(rlua::Error::RuntimeError("keygrabber callback already set!"
                                          .into())),
        _ => keygrabber_table.set(SECRET_CALLBACK, function)
    }
}

fn stop(lua: &Lua, _: ()) -> rlua::Result<()> {
    let keygrabber_table = lua.globals().get::<_, Table>(KEYGRABBER_TABLE)?;
    keygrabber_table.set(SECRET_CALLBACK, Value::Nil)
}

fn isrunning(lua: &Lua, _: ()) -> rlua::Result<bool> {
    let keygrabber_table = lua.globals().get::<_, Table>(KEYGRABBER_TABLE)?;
    match keygrabber_table.get::<_, Value>(SECRET_CALLBACK)? {
        Value::Function(_) => Ok(true),
        _ => Ok(false)
    }
}
