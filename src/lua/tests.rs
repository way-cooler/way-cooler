//! Lua thread test.
//! Spawns a lua thread!

use std::time::Duration;
use std::thread;
use std::collections::BTreeMap;

use hlua::any::AnyLuaValue::*;
use rustc_serialize::json::{Json, ToJson, Object};

use super::*;

#[test]
fn big_lua_test() {
    println!("Testing lua thread...");
    super::init();
    thread::sleep(Duration::from_secs(2));
    test_variable();

    send(LuaQuery::Terminate).unwrap();
    thread::sleep(Duration::from_millis(500u64));
    assert!(!thread_running())
}

fn test_variable() {
    let response = send(LuaQuery::Execute("hello = 'hello world!'".to_string()))
        .unwrap().recv().unwrap();
    assert_eq!(response, LuaResponse::Pong);

    let assertion = send(LuaQuery::Execute("assert(hello == 'hello world!')"
                                        .to_string())).unwrap().recv().unwrap();
    assert_eq!(assertion, LuaResponse::Pong);
}

fn test_bad_code() {
    
}


#[test]
fn convert_string() {
    let input = "Hello World".to_string();
    let json_input = Json::String(input.clone());
    assert_eq!(json_to_lua(json_input), LuaString(input));
}
#[test]
fn convert_numbers() {
    let lua_number = LuaNumber(42f64);
    assert_eq!(json_to_lua(Json::F64(42f64)), lua_number);
    assert_eq!(json_to_lua(Json::I64(42i64)), lua_number);
    assert_eq!(json_to_lua(Json::U64(42u64)), lua_number);
}

#[test]
fn convert_bool() {
    assert_eq!(json_to_lua(Json::Boolean(true)), LuaBoolean(true));
    assert_eq!(json_to_lua(Json::Boolean(false)), LuaBoolean(false));
}

#[test]
fn convert_array() {
    let json = Json::Array(
        vec![Json::Boolean(true), Json::F64(11f64),
             Json::String("Foo".to_string())]
    );
    let lua = LuaArray(vec![
        (LuaNumber(1f64), LuaBoolean(true)),
        (LuaNumber(2f64), LuaNumber(11f64)),
        (LuaNumber(3f64), LuaString("Foo".to_string()))
    ]);
    assert_eq!(json_to_lua(json), lua);
}

#[test]
fn convert_object() {
    // { name="foo", point: {x=0, y=1}, valid=false }
    // The fields are emitted in alphabetical order thanks to Json's
    // internal BTreeMap representation.

    let mut json_tree: Object = BTreeMap::new();
    let mut point_tree: Object = BTreeMap::new();
    point_tree.insert("x".to_string(), Json::U64(0u64));
    point_tree.insert("y".to_string(), Json::U64(1u64));
    json_tree.insert("point".to_string(), Json::Object(point_tree));
    json_tree.insert("name".to_string(), Json::String("foo".to_string()));
    json_tree.insert("valid".to_string(), Json::Boolean(false));

    let lua = LuaArray(vec![
        (LuaString("name".to_string()), LuaString("foo".to_string())),
        (LuaString("point".to_string()), LuaArray(vec![
            (LuaString("x".to_string()), LuaNumber(0f64)),
            (LuaString("y".to_string()), LuaNumber(1f64))
        ])),
        (LuaString("valid".to_string()), LuaBoolean(false)),
    ]);

    assert_eq!(json_to_lua(Json::Object(json_tree)), lua);
}
