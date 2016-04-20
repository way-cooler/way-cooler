//! Lua thread test.
//! Spawns a lua thread!

use std::time::Duration;
use std::thread;
use std::collections::BTreeMap;

use hlua::any::AnyLuaValue::*;
use hlua::LuaError;
use rustc_serialize::json::{Json, ToJson, Object};

use super::*;

fn wait_for_thread() {
    for i in 0..20 {
        if !thread_running() {
            thread::sleep_ms(200);
        }
        else { return; }
    }
    panic!("Thread didn't run!");
}

#[test]
fn activate_thread() {
    super::init();
}

#[test]
fn thread_exec_okay() {
    wait_for_thread();

    let hello_receiver = send(LuaQuery::Execute(
        "hello = 'hello world'".to_string())).unwrap();
    let hello_result = hello_receiver.recv().unwrap();
    assert!(hello_result.is_ok());
    assert_eq!(hello_result, LuaResponse::Pong);

    let assert_receiver = send(LuaQuery::Execute(
        "assert(hello == 'hello world')".to_string())).unwrap();
    let assert_result = assert_receiver.recv().unwrap();
    assert!(hello_result.is_ok());
    assert_eq!(assert_result, LuaResponse::Pong);
}

#[test]
fn thread_exec_err() {
    wait_for_thread();

    let assert_receiver = send(LuaQuery::Execute(
        "assert(true == false, 'Logic works')".to_string())).unwrap();
    let assert_result = assert_receiver.recv().unwrap();
    assert!(assert_result.is_err());
    assert!(!assert_result.is_ok());
    if let LuaResponse::Error(err) = assert_result {
        if let LuaError::ExecutionError(lua_err) = err {
            assert!(lua_err.contains("Logic works"));
        }
        else {
            panic!("thread_exec_err result was not an ExecutionError: {:?}",
                   err);
        }
    }
    else {
        panic!("thread_exec_err result was not an error: {:?}", assert_result);
    }
}

#[test]
fn thread_exec_file_ok() {
    wait_for_thread();

    let file_receiver = send(LuaQuery::ExecFile(
        "lib/test/lua-exec-file.lua".to_string())).unwrap();
    let file_result = file_receiver.recv().unwrap();
    if let LuaResponse::Error(err) = file_result {
        if let LuaError::ReadError(ioerr) = err {
            panic!("Lua thread was unable to execute file: {:?}", ioerr);
        }
        else {
            panic!("Unexpected error executing file: {:?}", err);
        }
    }
    assert!(file_result.is_ok());
    assert_eq!(file_result, LuaResponse::Pong);

    // Print the method from the file
    let test_receiver = send(LuaQuery::Execute(
        "foo = confirm_file()".to_string())).unwrap();
    let test_result = test_receiver.recv().unwrap();
    assert!(test_result.is_ok());
    assert!(test_result == LuaResponse::Pong);
}

#[test]
fn thread_exec_file_err() {
    wait_for_thread();

    let run_receiver = send(LuaQuery::ExecFile(
        "lib/test/lua-bad-assert.lua".to_string())).unwrap();
    let run_result = run_receiver.recv().unwrap();
    assert!(run_result.is_err());
    match run_result {
        LuaResponse::Error(err) => {
            match err {
                LuaError::ExecutionError(ex) => {
                    assert!(ex.contains("Dude, 1 is totally equal to 0"));
                },
                _ => panic!("Got wrong hlua error type: {:?}", err)
            }
        },
        _ => panic!("Got wrong LuaResponse type: {:?}", run_result)
    }

    let syntax_receiver = send(LuaQuery::ExecFile(
        "lib/test/lua-syntax-err.txt".to_string())).unwrap();
    let syntax_result = syntax_receiver.recv().unwrap();
    assert!(syntax_result.is_err());
    match syntax_result {
        LuaResponse::Error(err) => {
            match err {
                LuaError::SyntaxError(serr) => {
                    assert!(serr.contains("expected"));
                },
                _ => panic!("Got wrong hlua error type: {:?}", err)
            }
        }
        _ => panic!("Got wrong LuaResponse type: {:?}", syntax_result)
    }
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
