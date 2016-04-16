//! Metamethods for accesssing the registry values!

use super::super::super::registry;
use registry::RegistryAccess;
use super::super::super::lua::json_to_lua;

use hlua::any::AnyLuaValue;

use rustc_serialize::json::{Json, ToJson};
use std::ops::Deref;

pub fn index(table: AnyLuaValue, lua_key: AnyLuaValue)
             -> Result<AnyLuaValue, &'static str> {
    if let AnyLuaValue::LuaString(key) = lua_key {
        let maybe_json = registry::get_json(&key);
        if let Some(json_pair) = maybe_json {
            let (access, json) = json_pair;
            if access == RegistryAccess::Private {
                // TODO In lua_async: return nil
                Err("No value found for that key!")
            }
            else {
                Ok(json_to_lua(json.deref().to_json()))
            }
        }
        else {
            // Should be nil, nil doesn't seem to be included
            Err("No value found for that key!")
        }
    }
    else {
        Err("The registry can only be indexed with keys!")
    }
}

// Prevent lua from changing the registry?
pub fn new_index(table: AnyLuaValue, key: AnyLuaValue, val: AnyLuaValue)
                 -> Result<(), &'static str> {
    Err("You can't set the registry!")
}
