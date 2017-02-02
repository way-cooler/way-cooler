//! Tests for the registry

use std::sync::Arc;
use std::collections::HashMap;

use rustc_serialize::json::{Json, ToJson};

use registry::{self, RegMap};

/// Gets the initial HashMap used by the registry
pub fn registry_map() -> RegMap {
    let mut map = HashMap::new();

    let values: Vec<(&str, Json)> = vec![
        ("bool",  BOOL.to_json()),
        ("u64",   U64.to_json()),
        ("i64",   I64.to_json()),
        ("f64",   F64.to_json()),
        ("text",  TEXT.to_json()),
        ("point", POINT.to_json()),
        ("null",  Json::Null),
        ("u64s",  Json::Array(u64s())),
    ];

    for (name, json) in values.into_iter() {
        assert!(map.insert(name.to_string(), Arc::new(json)).is_none(),
                "Duplicate element inserted!");
    }

    map
}

// Constants for use with registry-accessing tests

pub const BOOL: bool = true;
pub const U64: u64 = 42;
pub const I64: i64 = -1;
pub const F64: f64 = 21.5;

pub const TEXT: &'static str = "Hello way-cooler";
pub const POINT: Point = Point { x: 12, y: 12 };

/// [0, 1, 2]
pub fn u64s() -> Vec<Json> {
    vec![Json::U64(0), Json::U64(1), Json::U64(2)]
}

json_convertible! {
    /// Has an x and y field
    #[derive(Debug, Clone, Eq, PartialEq)]
    struct Point {
        x: i32,
        y: i32
    }
}

unsafe impl Sync for Point {}

#[test]
fn add_keys() {
}

#[test]
fn contains_keys() {
    let keys = [
        "bool", "u64", "i64", "f64", "null", "text", "point", "u64s"];
    for key in keys.into_iter() {
        assert!(registry::get_data(key).is_ok(),
                "Could not find key {}", key);
    }
}

#[test]
fn objects_and_keys_equal() {
    let values = vec![
        ("bool",  BOOL.to_json()),
        ("u64",   U64.to_json()),
        ("i64",   I64.to_json()),
        ("f64",   F64.to_json()),
        ("text",  TEXT.to_json()),
        ("point", POINT.to_json()),
        ("null",  Json::Null),
        ("u64s",  Json::Array(u64s())),
    ];

    for (name, json) in values.into_iter() {
        let found = registry::get_data(name)
            .expect(&format!("Unable to get key {}", name));
        assert_eq!(*found, json);
    }
}
