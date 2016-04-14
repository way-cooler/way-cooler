//! Tests for the registry

use std::collections::BTreeMap;
use super::*;
use rustc_serialize::{Decodable, Decoder};
use rustc_serialize::json;
use rustc_serialize::json::{Json, ToJson};

#[derive(Debug, Clone, Eq, PartialEq)]
struct Point {
    x: i32,
    y: i32
}

impl ToJson for Point {
    fn to_json(&self) -> Json {
        let mut map = BTreeMap::new();
        map.insert("x".to_string(), Json::I64(self.x as i64));
        map.insert("y".to_string(), Json::I64(self.y as i64));
        Json::Object(map)
    }
}

impl Decodable for Point {
    fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
        let mut x = 0;
        let mut y = 0;

        x = try!(d.read_struct_field("x", 1, |d| d.read_i32()));
        y = try!(d.read_struct_field("y", 1, |d| d.read_i32()));

        Ok(Point { x: x, y: y })
    }
}

#[test]
fn registry_tests() {
    let num = 1i32;
    let double = -392f64;
    let string = "Hello world".to_string();
    let numbers = vec![1, 2, 3, 4, 5];
    let point = Point { x: -11, y: 12 };

    set(String::from("test_num"), num);
    set(String::from("test_double"), double);
    set(String::from("test_string"), string.clone());
    set(String::from("test_numbers"), numbers.clone());
    set(String::from("test_point"), point.clone());

    assert!(contains_key(&String::from("test_num")));
    assert!(contains_key(&String::from("test_double")));
    assert!(contains_key(&String::from("test_string")));
    assert!(contains_key(&String::from("test_numbers")));
    assert!(contains_key(&String::from("test_point")));

    assert_eq!(get::<_, i32>(&String::from("test_num")).unwrap(), num);
    assert_eq!(get::<_, f64>(&String::from("test_double")).unwrap(), double);
    assert_eq!(get::<_, String>(&String::from("test_string")).unwrap(), string);
    assert_eq!(get::<_, Vec<i32>>(&String::from("test_numbers")).unwrap(),
               numbers);
    assert_eq!(get::<_, Point>(&String::from("test_point")).unwrap(), point);


}
