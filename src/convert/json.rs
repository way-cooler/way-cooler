//! Conversion methods for JSON values.

use rustwlc::{Geometry, Point, Size};
use std::collections::BTreeMap;

use rlua;
use rustc_serialize::json::{Json, ToJson};

// TODO Fix
/// Converts a Json map into an AnyLuaValue
/*pub fn json_to_lua(json: Json) -> AnyLuaValue {
    match json {
        Json::String(val)  => LuaString(val),
        Json::Boolean(val) => LuaBoolean(val),
        Json::F64(val)     => LuaNumber(val),
        Json::I64(val)     => LuaNumber((val as i32) as f64),
        Json::U64(val)     => LuaNumber((val as u32) as f64),
        Json::Null         => LuaNil,
        Json::Array(vals)  => {
            let mut lua_arr = Vec::with_capacity(vals.len());
            for (ix, val) in vals.into_iter().enumerate() {
                lua_arr.push((LuaNumber(ix as f64 + 1.0),
                              json_to_lua(val)));
            }
            LuaArray(lua_arr)
        },
        Json::Object(vals) => {
            let mut lua_table = Vec::with_capacity(vals.len());
            for (key, val) in vals.into_iter() {
                lua_table.push((LuaString(key),
                                json_to_lua(val)));
            }
            LuaArray(lua_table)
        }
    }
}*/

/// Converts an `rlua::Value` to a `Json`.
pub fn lua_to_json<'lua>(lua_value: rlua::Value<'lua>)
                         -> Result<Json, rlua::Value<'lua>> {
    use rlua::Value::*;
    match lua_value {
        Nil => Ok(Json::Null),
        String(val) => Ok(Json::String(val.to_str().unwrap().into())),
        Number(val) => Ok(Json::F64(val)),
        Integer(val) => Ok(Json::I64(val as _)),
        Boolean(val) => Ok(Json::Boolean(val)),
        Table(arr) => lua_table_to_json(arr),
        _ => Err(lua_value)
    }
}

/// Convert an AnyLuaValue to a Json array using numerical indicies.
///
/// # Result
/// This function returns an Err if the Lua object has a non-String key.
pub fn lua_table_to_json<'lua>(table: rlua::Table<'lua>)
                               -> Result<Json, rlua::Value<'lua>> {
    use rlua::Value;
    use rlua::Error;
    // Check if every key is a number
    let mut counter = 0.0; // Account for first index?

    for entry in table.clone().pairs::<Value, Value>() {
        let (key, _ )= entry.unwrap();
        match key {
            Value::Number(num) => {
                counter += num;
            }
            Value::String(_) => {
                break;
            }
            // Non-string keys are not allowed
            _ => {
                return Err(Value::Error(Error::FromLuaConversionError {
                    from: "Lua table",
                    to: "JSON object",
                    message: Some(format!("Could not convert {:#?}", table))
                }))
            }
        }
    }

    // Gauss' trick
    let len = table.len().unwrap();
    let desired_sum = (len * (len + 1)) / 2;
    if counter != desired_sum as f64 {
        return lua_object_to_json(table)
    }

    let mut json_arr: Vec<Json> = Vec::with_capacity(len as _);

    for entry in table.pairs::<Value, Value>() {
        let (_, val) = entry.unwrap();
        let lua_val = lua_to_json(val)?;
        json_arr.push(lua_val);
    }
    Ok(Json::Array(json_arr))
}

/// Converts an AnyLuaValue object to a Json object.
///
/// Will return an Err if the Lua object uses non-String keys.
pub fn lua_object_to_json<'lua>(table: rlua::Table<'lua>)
                          -> Result<Json, rlua::Value<'lua>> {
    use rlua::Value;
    let mut json_obj: BTreeMap<String, Json> = BTreeMap::new();

    for entry in table.clone().pairs::<Value, Value>() {
        let (key, val) = entry.unwrap();
        match key {
            Value::String(text) => {
                let text = text.to_str().unwrap().into();
                json_obj.insert(text, lua_to_json(val.clone())?);
            },
            Value::Number(ix) => {
                json_obj.insert(ix.to_string(), lua_to_json(val.clone())?);
            }
            _ => {
                return Err(Value::Table(table))
            }
        }
    }
    Ok(Json::Object(json_obj))
}


pub fn size_to_json(size: Size) -> Json {
    let mut map = BTreeMap::new();
    map.insert("w".into(), size.w.to_json());
    map.insert("h".into(), size.h.to_json());
    map.to_json()
}

pub fn point_to_json(point: Point) -> Json {
    let mut map = BTreeMap::new();
    map.insert("x".into(), point.x.to_json());
    map.insert("y".into(), point.y.to_json());
    map.to_json()
}

pub fn geometry_to_json(geometry: Geometry) -> Json {
    let mut map = BTreeMap::new();
    let origin = point_to_json(geometry.origin);
    let size = size_to_json(geometry.size);
    map.insert("origin".into(), origin);
    map.insert("size".into(), size);
    map.to_json()
}
