//! Conversion methods for JSON values.

use std::collections::BTreeMap;
use wlroots::{Origin, Size};

use rlua;
use rustc_serialize::json::{Json, ToJson};

/// Converts a Json map into a Lua value
pub fn json_to_lua<'lua>(lua: &'lua rlua::Lua,
                         json: Json)
                         -> Result<rlua::Value<'lua>, rlua::Error> {
    use rlua::Value;
    match json {
        Json::String(val) => Ok(Value::String(lua.create_string(&val)?)),
        Json::Boolean(val) => Ok(Value::Boolean(val)),
        Json::F64(val) => Ok(Value::Number(val)),
        Json::I64(val) => Ok(Value::Number(val as f64)),
        Json::U64(val) => Ok(Value::Number(val as f64)),
        Json::Null => Ok(Value::Nil),
        Json::Array(vals) => {
            let mut new_vals = Vec::with_capacity(vals.len());
            for (idx, val) in vals.into_iter().enumerate() {
                new_vals.push((idx, json_to_lua(lua, val)?));
            }
            lua.create_table_from(new_vals).map(|table| Value::Table(table))
        }
        Json::Object(vals) => {
            let mut new_vals = BTreeMap::new();
            for (k, v) in vals {
                new_vals.insert(k, json_to_lua(lua, v)?);
            }
            lua.create_table_from(new_vals).map(|table| Value::Table(table))
        }
    }
}

/// Converts an `rlua::Value` to a `Json`.
pub fn lua_to_json<'lua>(lua_value: rlua::Value<'lua>) -> Result<Json, rlua::Error> {
    use rlua::Error;
    use rlua::Value::*;
    match lua_value {
        Nil => Ok(Json::Null),
        String(val) => {
            val.to_str().map_err(|err| Error::RuntimeError(format!("Bad string: {:#?}", err)))
               .map(|string| Json::String(string.into()))
        }
        Number(val) => Ok(Json::F64(val)),
        Integer(val) => Ok(Json::I64(val as _)),
        Boolean(val) => Ok(Json::Boolean(val)),
        Table(arr) => lua_table_to_json(arr),
        _ => Err(Error::RuntimeError(format!("Did not expect {:#?}", lua_value)))
    }
}

/// Convert an AnyLuaValue to a Json array using numerical indices.
///
/// # Result
/// This function returns an Err if the Lua object has a non-String key.
fn lua_table_to_json<'lua>(table: rlua::Table<'lua>) -> Result<Json, rlua::Error> {
    use rlua::Error;
    use rlua::Value;
    // Check if every key is a number
    let mut counter = 0.0; // Account for first index?

    for entry in table.clone().pairs::<Value, Value>() {
        let (key, _) = entry?;
        match key {
            Value::Number(num) => {
                counter += num;
            }
            Value::String(_) => break,
            // Non-string keys are not allowed
            _ => {
                return Err(Error::FromLuaConversionError { from: "Lua table",
                                                           to: "JSON object",
                                                           message: Some(format!("Could not \
                                                                                  convert {:\
                                                                                  #?}",
                                                                                 table)) })
            }
        }
    }

    let len = table.len()?;
    // Gauss' trick
    let desired_sum = (len * (len + 1)) / 2;
    if counter != desired_sum as f64 {
        return lua_object_to_json(table)
    }

    let mut json_arr: Vec<Json> = Vec::with_capacity(len as _);

    for entry in table.pairs::<Value, Value>() {
        let (_, val) = entry?;
        let lua_val = lua_to_json(val)?;
        json_arr.push(lua_val);
    }
    Ok(Json::Array(json_arr))
}

/// Converts an AnyLuaValue object to a Json object.
///
/// Will return an Err if the Lua object uses non-String keys.
fn lua_object_to_json<'lua>(table: rlua::Table<'lua>) -> Result<Json, rlua::Error> {
    use rlua::{Error, Value};
    let mut json_obj: BTreeMap<String, Json> = BTreeMap::new();

    for entry in table.clone().pairs::<Value, Value>() {
        let (key, val) = entry?;
        match key {
            Value::String(text) => {
                let text = text.to_str()?.into();
                json_obj.insert(text, lua_to_json(val)?);
            }
            Value::Number(ix) => {
                json_obj.insert(ix.to_string(), lua_to_json(val)?);
            }
            val => {
                return Err(Error::RuntimeError(format!("Did not expect {:#?} as key \
                                                        in {:#?}",
                                                       val, table)))
            }
        }
    }
    Ok(Json::Object(json_obj))
}
