//! Metamethods for accesssing the registry values!

use super::super::super::convert::{ToTable, FromTable};
use super::super::super::registry;
use registry::*;

use hlua::any::AnyLuaValue;
use hlua::any::AnyLuaValue::*;

use std::ops::Deref;

pub fn index(table: AnyLuaValue, lua_key: AnyLuaValue) -> AnyLuaValue {
    if let LuaString(key) = lua_key {
        let maybe_lua = registry::get_lua(&key);
        if let Some(lua_pair) = maybe_lua {
            let (access, lua_arc) = lua_pair;
            if access.contains(registry::LUA_ACCESS) {
                return lua_arc.deref().clone();
            }
        }
    }
    AnyLuaValue::LuaNil
}

// Prevent lua from changing the registry?
pub fn new_index(table: AnyLuaValue, lua_key: AnyLuaValue, val: AnyLuaValue)
                 -> Result<(), &'static str> {
    if let LuaString(key) = lua_key {
        let ref mut reg = registry::write_lock();
        if let Some(mut reg_val) = (*reg).get_mut(&key) {
            reg_val.set_lua(val);
            return Ok(());
        }
    }
    Err("Invalid key!")
}

pub fn to_string(table: AnyLuaValue) -> &'static str {
    "A table used to share data with way-cooler"
}
