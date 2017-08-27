//! Rust code which is called from lua in the init file
#![deny(dead_code)]

use rustc_serialize::json::ToJson;
use uuid::Uuid;
use super::{send, LuaQuery, running};
use rlua;
use rlua::prelude::LuaResult;
use ::convert::json::json_to_lua;

use registry::{self};
use commands;
use keys::{self, KeyPress, KeyEvent};

use super::thread::{update_registry_value};

/// We've `include!`d the code which initializes from the Lua side.

/// Register all the Rust functions for the lua libraries
pub fn register_libraries(lua: &mut rlua::Lua) -> LuaResult<()> {
    trace!("Registering Rust libraries...");
    {
        let rust_table = lua.create_table();
        rust_table.set("init_workspaces",
                       lua.create_function(init_workspaces))?;
        rust_table.set("register_lua_key",
                       lua.create_function(register_lua_key))?;
        rust_table.set("unregister_lua_key",
                       lua.create_function(unregister_lua_key))?;
        rust_table.set("register_command_key",
                       lua.create_function(register_command_key))?;
        rust_table.set("register_mouse_modifier",
                       lua.create_function(register_mouse_modifier))?;
        rust_table.set("keypress_index",
                       lua.create_function(keypress_index))?;
        rust_table.set("ipc_run",
                       lua.create_function(ipc_run))?;
        rust_table.set("ipc_get",
                       lua.create_function(ipc_get))?;
        rust_table.set("ipc_set",
                       lua.create_function(ipc_set))?;
        let globals = lua.globals();
        globals.set("__rust", rust_table)?;
    }
    trace!("Executing Lua init...");
    let init_code = include_str!("../../lib/lua/lua_init.lua");
    let util_code = include_str!("../../lib/lua/utils.lua");
    lua.exec::<()>(util_code, Some("utils.lua"))?;
    lua.exec::<()>(init_code, Some("lua_init.lua"))?;
    trace!("Lua register_libraries complete");
    Ok(())
}

/// Run a command
fn ipc_run(_lua: &rlua::Lua, command: String) -> Result<(), rlua::Error> {
    use rlua::Error;
    commands::get(&command).map(|com| com())
        .ok_or(Error::RuntimeError("Command does not exist".into()))
}

/// IPC 'get' handler
fn ipc_get<'lua>(lua: &'lua rlua::Lua, (category, key): (String, String))
                 -> Result<rlua::Value<'lua>, rlua::Error> {
    use rlua::Error;
    let lock = registry::clients_read();
    let client = lock.client(Uuid::nil()).unwrap();
    let handle = registry::ReadHandle::new(&client);
    handle.read(category)
        .map_err(|_| Error::RuntimeError("Could not locate that category".into()))
        .and_then(|category| category.get(&key)
                  .ok_or(Error::RuntimeError("Could not locate that key in the category".into()))
                  .and_then(|value| {
                      let value = value.to_json();
                      json_to_lua(lua, value)
                  }))
}

/// ipc 'set' handler
fn ipc_set(_lua: &rlua::Lua, category: String) -> Result<(), rlua::Error> {
    update_registry_value(category);
    if running() {
        send(LuaQuery::UpdateRegistryFromCache)
            .expect("Could not send message to Lua thread to update registry");
    }
    Ok(())
}

fn init_workspaces(_: &rlua::Lua, _: rlua::Value) -> Result<(), rlua::Error> {
    warn!("Attempting to call `init_workspaces`, this is not implemented");
    Ok(())
}

/// Registers a modifier to be used in conjunction with mouse commands
fn register_mouse_modifier(_lua: &rlua::Lua, modifier: String)
                           -> Result<(), rlua::Error> {
    use rlua::Error;
    let modifier = keys::keymod_from_names(&[modifier.as_str()])
        .map_err(|txt| Error::RuntimeError(txt))?;
    keys::register_mouse_modifier(modifier);
    Ok(())
}

/// Registers a command keybinding.
fn register_command_key(_lua: &rlua::Lua,
                        (mods, command, _repeat, passthrough):
                        (String, String, bool, bool))
                        -> Result<(), rlua::Error> {
    use rlua::Error;
    if let Ok(press) = keypress_from_string(&mods) {
        commands::get(&command)
            .ok_or(Error::RuntimeError(
                format!("Command {} for keybinding {} not found", command, press)))
            .map(|command|
                keys::register(press, KeyEvent::Command(command), passthrough))?;
        Ok(())
    }
    else {
        Err(Error::RuntimeError(format!("Invalid keypress {}, {}", mods, command)))
    }
}

/// Rust half of registering a Lua key: store the KeyPress in the keys table
/// and send Lua back the index for __key_map.
fn register_lua_key(_lua: &rlua::Lua, (mods, _repeat, passthrough):
                          (String, bool, bool))
                          -> Result<String, rlua::Error> {
    use rlua::Error;
    keypress_from_string(&mods)
        .map(|press| {
            keys::register(press.clone(), KeyEvent::Lua, passthrough);
            press.get_lua_index_string()
        }).map_err(|_| Error::RuntimeError(format!("Invalid keys '{}'", mods)))
}

/// Rust half of unregistering a Lua key. This pops it from the key table, if
/// it exists.
///
/// If a key wasn't registered, a proper error string is raised.
fn unregister_lua_key(_lua: &rlua::Lua, mods: String)
                            -> Result<String, rlua::Error> {
    use rlua::Error;
    keypress_from_string(&mods).and_then(|press| {
        if let Some(action) = keys::unregister(&press) {
            trace!("Removed keybinding \"{}\" for {:?}", press, action);
            Ok(press.get_lua_index_string())
        } else {
            let error_str = format!("Could not remove keybinding \"{}\": \
                                     Not registered!",
                                    press);
            warn!("Could not remove keybinding \"{}\": Not registered!",
                  press);
            Err(error_str)
        }
    }).map_err(|err|
               Error::RuntimeError(format!("Invalid keys '{}': {:#?}", mods, err)))
}

/// Parses a keypress from a string
fn keypress_from_string(mods: &str) -> Result<KeyPress, String> {
    let parts: Vec<&str> = mods.split(',').collect();
    if let Some((ref key, mods)) = parts.split_last() {
        KeyPress::from_key_names(mods, &key)
    }
    else {
        Err(format!("Invalid key '{}'", mods))
    }
}

fn keypress_index(_lua: &rlua::Lua, press: String) -> Result<String, rlua::Error> {
    use rlua::Error;
    keypress_from_string(&press)
        .map(|key| key.get_lua_index_string())
        .map_err(|err_msg| Error::RuntimeError(err_msg))
}
