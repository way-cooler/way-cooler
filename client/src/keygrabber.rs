//! AwesomeWM Keygrabber interface

use rlua::{self, Function, Table, Value};
use xkbcommon::xkb::{keysym_get_name, Keysym};

use crate::common::signal;
use crate::LUA;

pub const KEYGRABBER_TABLE: &str = "keygrabber";
const KEYGRABBER_CALLBACK: &str = "__callback";

/// Init the methods defined on this interface.
pub fn init(lua: rlua::Context) -> rlua::Result<()> {
    let keygrabber_table = lua.create_table()?;
    let meta = lua.create_table()?;
    meta.set("__index", lua.create_function(index)?)?;
    meta.set("__newindex", lua.create_function(new_index)?)?;
    keygrabber_table.set("run", lua.create_function(run)?)?;
    keygrabber_table.set("stop", lua.create_function(stop)?)?;
    keygrabber_table.set("isrunning", lua.create_function(isrunning)?)?;
    keygrabber_table.set_metatable(Some(meta));
    let globals = lua.globals();
    globals.set(KEYGRABBER_TABLE, keygrabber_table)
}

/// Given the current input, handle calling the Lua defined callback if it is
/// defined with the input.
#[allow(dead_code)]
pub fn keygrabber_handle(mods: Vec<Keysym>, sym: Keysym, state: u32) -> rlua::Result<()> {
    LUA.with(|lua| {
        let lua = lua.borrow();
        // TODO Need key state proper type
        let lua_state = if state == 0 { "press" } else { "release" }.into();
        lua.context(|ctx| {
            let lua_sym = keysym_get_name(sym);
            let lua_mods = crate::lua::mods_to_lua(ctx, &mods)?;
            let res = call_keygrabber(ctx, (lua_mods, lua_sym, lua_state));
            match res {
                Ok(_) | Err(rlua::Error::FromLuaConversionError { .. }) => Ok(()),
                err => err
            }
        })
    })
}

/// Check is the Lua callback function is set
#[allow(dead_code)]
pub fn is_keygrabber_set(lua: rlua::Context) -> bool {
    lua.named_registry_value::<str, Function>(KEYGRABBER_CALLBACK)
        .is_ok()
}

/// Call the Lua callback function for when a key is pressed.
#[allow(dead_code)]
pub fn call_keygrabber<'lua>(
    lua: rlua::Context<'lua>,
    (mods, key, event): (Table<'lua>, String, String)
) -> rlua::Result<()> {
    let lua_callback = lua.named_registry_value::<str, Function>(KEYGRABBER_CALLBACK)?;
    lua_callback.call((mods, key, event))
}

fn run<'lua>(lua: rlua::Context<'lua>, function: Function<'lua>) -> rlua::Result<()> {
    match lua.named_registry_value::<str, Value>(KEYGRABBER_CALLBACK)? {
        Value::Function(_) => Err(rlua::Error::RuntimeError(
            "keygrabber callback already set!".into()
        )),
        _ => lua.set_named_registry_value(KEYGRABBER_CALLBACK, function)
    }
}

fn stop(lua: rlua::Context, _: ()) -> rlua::Result<()> {
    lua.set_named_registry_value(KEYGRABBER_CALLBACK, Value::Nil)
}

fn isrunning(lua: rlua::Context, _: ()) -> rlua::Result<bool> {
    match lua.named_registry_value::<str, Value>(KEYGRABBER_CALLBACK)? {
        Value::Function(_) => Ok(true),
        _ => Ok(false)
    }
}

fn index<'lua>(lua: rlua::Context<'lua>, args: Value<'lua>) -> rlua::Result<()> {
    signal::global_emit_signal(lua, ("debug::index::miss".into(), args))
}

fn new_index<'lua>(lua: rlua::Context<'lua>, args: Value<'lua>) -> rlua::Result<()> {
    signal::global_emit_signal(lua, ("debug::newindex::miss".into(), args))
}
