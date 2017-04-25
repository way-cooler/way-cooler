//! Conversion methods for JSON values.

use rustwlc::{Geometry, Point, Size};
use std::collections::BTreeMap;

use hlua::any::AnyLuaValue;
use hlua::any::AnyLuaValue::*;
use rustc_serialize::json::{Json, ToJson};

/// Converts a Json map into an AnyLuaValue
pub fn json_to_lua(json: Json) -> AnyLuaValue {
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
}

/// Converts an `AnyLuaValue` to a `Json`.
///
/// For an already-matched `LuaArray`, use `lua_array_to_json`.
///
/// For a `LuaArray` that should be mapped to a `JsonObject`,
/// use `lua_object_to_json`.
pub fn lua_to_json(lua: AnyLuaValue) -> Result<Json, AnyLuaValue> {
    match lua {
        LuaNil => Ok(Json::Null),
        LuaString(val) => Ok(Json::String(val)),
        LuaNumber(val) => Ok(Json::F64(val)),
        LuaBoolean(val) => Ok(Json::Boolean(val)),
        LuaArray(arr) => lua_array_to_json(arr),
        LuaOther => Err(lua)
    }
}

/// Convert an AnyLuaValue to a Json array using numerical indicies.
///
/// # Result
/// This function returns an Err if the Lua object has a non-String key.
pub fn lua_array_to_json(arr: Vec<(AnyLuaValue, AnyLuaValue)>)
                         -> Result<Json, AnyLuaValue> {
    // Check if every key is a number
    let mut counter = 0.0; // Account for first index?

    let mut return_early = false;
    for &(ref key, ref _val) in &arr {
        match *key {
            LuaNumber(num) => {
                counter += num;
            }
            LuaString(_) => {
                break;
            }
            // Non-string keys are not allowed
            _ => {
                return_early = true;
                break;
            }
        }
    }
    if return_early {
        return Err(AnyLuaValue::LuaArray(arr));
    }

    // Gauss' trick
    let desired_sum = ((arr.len()) * (arr.len() + 1)) / 2;
    if counter != desired_sum as f64 {
        return lua_object_to_json(arr)
    }

    let mut json_arr: Vec<Json> = Vec::with_capacity(arr.len());

    for (_key, val) in arr.into_iter() {
        let lua_val = try!(lua_to_json(val));
        json_arr.push(lua_val);
    }
    Ok(Json::Array(json_arr))
}

/// Converts an AnyLuaValue object to a Json object.
///
/// Will return an Err if the Lua object uses non-String keys.
pub fn lua_object_to_json(obj: Vec<(AnyLuaValue, AnyLuaValue)>)
                          -> Result<Json, AnyLuaValue> {
    let mut json_obj: BTreeMap<String, Json> = BTreeMap::new();

    let mut error = false;
    for &(ref key, ref val) in obj.iter() {
        match *key {
            LuaString(ref text) => {
                json_obj.insert(text.clone(), try!(lua_to_json(val.clone())));
            },
            LuaNumber(ref ix) => {
                json_obj.insert(ix.to_string(), try!(lua_to_json(val.clone())));
            }
            _ => {
                error = true;
                break
            }
        }
    }
    if error {
        return Err(AnyLuaValue::LuaArray(obj))
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
