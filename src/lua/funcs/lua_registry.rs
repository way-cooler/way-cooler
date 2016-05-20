//! Metamethods for accesssing the registry values!

use std::ops::Deref;
use std::sync::Arc;

use hlua::any::AnyLuaValue;
use hlua::any::AnyLuaValue::LuaString;

use registry;
use registry::{RegistryField, AccessFlags};
use convert::ToTable;
use convert::json::lua_to_json;

pub fn index(_table: AnyLuaValue, lua_key: AnyLuaValue) -> AnyLuaValue {
    if let LuaString(key) = lua_key {
        if let Ok((access, json_arc)) = registry::get_json(&key) {
            if access.contains(AccessFlags::READ()) {
                return json_arc.deref().clone().to_table();
            }
        }
    }
    AnyLuaValue::LuaNil
}

// Prevent lua from changing the registry?
pub fn new_index(_table: AnyLuaValue, lua_key: AnyLuaValue, val: AnyLuaValue)
                 -> Result<(), &'static str> {
    if let LuaString(key) = lua_key {
        let json = try!(lua_to_json(val).map_err(
            |_| "Unable to convert value to JSON!"));
        let mut reg = registry::write_lock();
        let flags: AccessFlags;
        if let Some(reg_field) = reg.get(&key) {
            if let Some((access, _old_arc)) = reg_field.clone().as_object() {
                if !access.contains(AccessFlags::WRITE()) {
                    return Err("Unable to modify that key!");
                }
                flags = access;
            }
            else {
                return Err("Cannot modify a command!");
            }
        }
        else {
            return Err("Cannot create a new key! Use config.set instead.");
        }
        let new_val = RegistryField::Object {flags: flags, data: Arc::new(json) };
        reg.insert(key, new_val);
        return Err("That value does not yet exist!");
        // Putting an else here would mean allowing Lua code to create new keys
    }
    return Err("Invalid key!");
}

/// Method called on Lua code like `print(registry)`
pub fn to_string(_table: AnyLuaValue) -> &'static str {
    "A table used to share data with way-cooler"
}
