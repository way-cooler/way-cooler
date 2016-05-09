//! Metamethods for accesssing the registry values!

use super::super::super::registry;

use hlua::any::AnyLuaValue;
use hlua::any::AnyLuaValue::*;

use std::ops::Deref;

pub fn index(_table: AnyLuaValue, lua_key: AnyLuaValue) -> AnyLuaValue {
    if let LuaString(key) = lua_key {
        if let Some(lua_pair) = registry::get_lua(&key) {
            let (access, lua_arc) = lua_pair;
            if access.contains(registry::LUA_READ) {
                return lua_arc.deref().clone();
            }
        }
    }
    AnyLuaValue::LuaNil
}

// Prevent lua from changing the registry?
pub fn new_index(_table: AnyLuaValue, lua_key: AnyLuaValue, val: AnyLuaValue)
                 -> Result<(), &'static str> {
    if let LuaString(key) = lua_key {
        let mut reg = registry::write_lock();
        if let Some(mut reg_val) = (*reg).get_mut(&key) {
            reg_val.set_lua(val);
            return Ok(());
        }
        // Putting an else here would mean allowing Lua code to create new keys
    }
    Err("Invalid key!")
}

/// Method called on Lua code like `print(registry)`
pub fn to_string(_table: AnyLuaValue) -> &'static str {
    "A table used to share data with way-cooler"
}
