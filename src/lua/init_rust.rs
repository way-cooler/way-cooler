//! Rust code which is called from lua in the init file
#![deny(unused_code)]

use std::sync::Arc;
use std::ops::Deref;

use hlua::{self, Lua, LuaTable};
use hlua::any::AnyLuaValue;
use hlua::any::AnyLuaValue::*;
use rustc_serialize::json::Json;

use registry::{self, RegistryField, RegistryError, AccessFlags};
use commands;
use keys::{self, KeyPress, KeyEvent};
use convert::json::{json_to_lua, lua_to_json};
use convert::serialize::ToTable;

type OkayResult = Result<(), &'static str>;
type ValueResult = Result<AnyLuaValue, &'static str>;

/// Register all the Rust functions for the lua libraries
pub fn register_libraries(lua: &mut Lua) {
    {
        let mut rust_table: LuaTable<_> = lua.empty_array("__rust");
        rust_table.set("init_workspaces", hlua::function1(init_workspaces));
        rust_table.set("register_lua_key", hlua::function2(register_lua_key));
        rust_table.set("register_command_key", hlua::function3(register_command_key));
    }
    {
        let mut ipc_table: LuaTable<_> = lua.empty_array("way_cooler");
        ipc_table.set("run", hlua::function1(ipc_run));
        ipc_table.set("get", hlua::function1(ipc_get));
        ipc_table.set("set", hlua::function2(ipc_set));
        let mut meta_ipc = ipc_table.get_or_create_metatable();
        meta_ipc.set("__metatable", "Turtles all the way down");
        meta_ipc.set("__index", hlua::function2(index));
        meta_ipc.set("__newindex", hlua::function3(new_index));
    }
    {
        let config_table: LuaTable<_> = lua.empty_array("config");
        let mut meta_config = config_table.get_or_create_metatable();
        meta_config.set("__metatable", "Turtles all the way down");
    }
}


/// Run a command
fn ipc_run(command: String) -> Result<(), &'static str> {
    match commands::get(&command) {
        Some(com) => {
            com();
            Ok(())
        },
        None => Err("Command does not exist")
    }
}

/// IPC 'get' handler
fn ipc_get(key: String) -> Result<AnyLuaValue, &'static str> {
    match registry::get_data(&key) {
        Ok(regdata) => {
            let (flags, arc_data) = regdata.resolve();

            Ok(json_to_lua(arc_data.deref().clone()))
        },
        Err(err) => match err {
            RegistryError::InvalidOperation =>
                Err("Cannot get that key, use set or assign"),
            RegistryError::KeyNotFound =>
                Err("Key not found")
        }
    }
}

/// ipc 'set' handler
fn ipc_set(key: String, value: AnyLuaValue) -> Result<(), &'static str> {
    let json = try!(lua_to_json(value).map_err(
        |_| "Unable to convert value to JSON!"));
    match registry::set_json(key, json.clone()) {
        Ok(data) => {
            data.call(json);
            Ok(())
        }
        Err(RegistryError::InvalidOperation) =>
            Err("That value cannot be set!"),
        Err(RegistryError::KeyNotFound) =>
            Err("That key could not be found!")
    }
}

fn new_index(_table: AnyLuaValue, lua_key: AnyLuaValue, val: AnyLuaValue) -> OkayResult {
    if let LuaString(key) = lua_key {
        ipc_set(key, val)
    }
    else {
        Err("Invalid key, String expected")
    }
}

fn index(_table: AnyLuaValue, lua_key: AnyLuaValue) ->  ValueResult {
    if let LuaString(key) = lua_key {
        ipc_get(key)
    }
    else {
        Err("Invalid key, string expected")
    }
}

fn init_workspaces(options: AnyLuaValue) -> OkayResult {
    Err("Not yet implemented")
}

/// Registers a command keybinding.
fn register_command_key(mods: String, command: String, _repeat: bool) -> OkayResult {
    if let Some(press) = keypress_from_string(mods) {
        if let Some(command) = commands::get(&command) {
            keys::register(vec![(press, KeyEvent::Command(command))]);
            Ok(())
        }
        else {
            Err("Command not found")
        }
    }
    else {
        Err("Invalid keypress")
    }
}

/// Rust half of registering a Lua key: store the KeyPress in the keys table
/// and send Lua back the index for __key_map.
fn register_lua_key(mods: String, repeat: bool) -> Result<String, String> {
    if let Some(press) = keypress_from_string(mods) {
        keys::register(vec![(press.clone(), KeyEvent::Lua)]);
        Ok(press.get_lua_index_string())
    }
    else {
        Err("Invalid keypress".to_string())
    }
}

/// Parses a keypress from a string
fn keypress_from_string(mods: String) -> Option<KeyPress> {
    let parts: Vec<&str> = mods.split(',').collect();
    if let Some((ref key, mods)) = parts.split_last() {
        KeyPress::from_key_names(mods, &key).ok()
    }
    else {
        None
    }
}
