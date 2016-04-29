//! Lua thread test.
//! Spawns a lua thread!

use std::time::Duration;
use std::thread;
use std::collections::BTreeMap;

use hlua::any::AnyLuaValue;
use hlua::any::AnyLuaValue::*;
use hlua::lua_tables::LuaTable;
use hlua::{Lua, LuaError};

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
        "confirm_file()".to_string())).unwrap();
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
fn test_rust_exec() {
    wait_for_thread();
    let rust_receiver = send(LuaQuery::ExecRust(rust_lua_fn)).unwrap();
    let rust_result = rust_receiver.recv().unwrap();
    assert!(rust_result.is_ok());

    match rust_result {
        LuaResponse::Variable(Some(var)) => {
            match var {
                AnyLuaValue::LuaBoolean(b) => {
                    if !b {
                        panic!("Rust function failed!");
                    }
                },
                _ => panic!("Rust function returned wrong AnyLuaValue")
            }
        },
        _ => panic!("Got wrong LuaResponse from Rust function!")
    }
}

fn rust_lua_fn(lua: &mut Lua) -> AnyLuaValue {
    {
        let mut foo = lua.empty_array("foo");
        foo.set("bar", 12.0);
    }
    let mut maybe_foo = lua.get::<LuaTable<_>, _>("foo");
    assert!(maybe_foo.is_some());
    let mut foo = maybe_foo.unwrap();
    assert!(foo.get::<f64, _>("bar").is_some());
    AnyLuaValue::LuaBoolean(true)
}

