//! Metamethods for accesssing the registry values!

use super::super::{json_to_lua, lua_to_json};
use super::super::super::registry;
use registry::*;

use hlua::any::AnyLuaValue;

use rustc_serialize::json::{Json, ToJson};
use std::ops::Deref;

pub fn index(table: AnyLuaValue, lua_key: AnyLuaValue) -> AnyLuaValue {
    if let AnyLuaValue::LuaString(key) = lua_key {
        let maybe_json = registry::get_json(&key);
        if let Some(json_pair) = maybe_json {
            let (access, json) = json_pair;
            if access.contains(registry::LUA_ACCESS) {
                return json_to_lua(json.deref().to_json());
            }
        }
    }
    AnyLuaValue::LuaNil
}

// Prevent lua from changing the registry?
pub fn new_index(table: AnyLuaValue, lua_key: AnyLuaValue, val: AnyLuaValue)
                 -> Result<(), &'static str> {
    if let AnyLuaValue::LuaString(key) = lua_key {
        if let Ok(json_val) = lua_to_json(val) {
            let ref mut reg = registry::write_lock();
            if let Some(mut reg_val) = (*reg).get_mut(&key) {
                reg_val.set_json(json_val);
                return Ok(());
            }
        }
    }
    Err("Invalid key!")
}

pub fn to_string(table: AnyLuaValue) -> &'static str {
    "A table used to share data with way-cooler"
}
