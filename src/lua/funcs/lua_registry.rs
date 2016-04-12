//! Metamethods for accesssing the registry values!

use super::super::super::registry;

use hlua::{LuaTable};
use hlua::any::AnyLuaValue;

use rustc_serialize::json::{Json, ToJson};
use std::ops::Deref;
pub fn index(table: AnyLuaValue, lua_key: AnyLuaValue) -> Result<AnyLuaValue, &'static str> {
    if let AnyLuaValue::LuaString(key) = lua_key {
        let maybe_json = registry::get_json(&key);
        if let Some(json) = maybe_json {
            Ok(convert_json(json.deref().to_json()))
        }
        else {
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

fn convert_json(json: Json) -> AnyLuaValue {
    match json {
        Json::String(val) => AnyLuaValue::LuaString(val),
        Json::Boolean(val) => AnyLuaValue::LuaBoolean(val),
        Json::F64(val) => AnyLuaValue::LuaNumber(val),
        Json::I64(val) => AnyLuaValue::LuaNumber((val as i32) as f64),
        Json::U64(val) => AnyLuaValue::LuaNumber((val as u32) as f64),
        Json::Null => AnyLuaValue::LuaString("nil".to_string()),
        Json::Array(mut vals) => {
            let mut count = 0f64;
            // Gotta love that 1-based indexing. Start at zero but increment for
            // the first one. It works here at least.
            AnyLuaValue::LuaArray(vals.into_iter().map(|v| {
                count += 1.0;
                (AnyLuaValue::LuaNumber(count), convert_json(v))
            }).collect())
        },
        Json::Object(mut vals) => {
            AnyLuaValue::LuaArray(vals.into_iter().map(|(key, val)| {
                (AnyLuaValue::LuaString(key), convert_json(val))
            }).collect())
        }
    }
}
