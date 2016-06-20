//! Tests individual commands via `reply`

use std::collections::HashSet;

use rustc_serialize::json::Json;

use super::*;
use super::super::channel;
// use commands::tests::wait_for_commands;
// use registry::tests::wait_for_registry;

const PING: &'static str = r#"{ "type":"ping" }"#;
const COMMANDS: &'static str = r#"{ "type":"commands" }"#;
const VERSION: &'static str = r#"{ "type":"version" }"#;

const RUN_COMMAND: &'static str =  r#"{ "type":"run", "key":"command" }"#;
const RUN_PANIC_COMMAND: &'static str=r#"{"type":"run","key":"panic_command"}"#;
const RUN_PROP_GET: &'static str = r#"{ "type":"run", "key":"get_prop" }"#;
const RUN_OBJECT: &'static str =   r#"{ "type":"run", "key":"point" }"#;
const RUN_NO_KEY: &'static str =   r#"{ "type":"run" }"#;

const GET_OBJECT: &'static str =     r#"{ "type":"get", "key":"point" }"#;
const GET_PROP: &'static str =       r#"{ "type":"get", "key":"prop_get" }"#;
const GET_READONLY: &'static str =   r#"{ "type":"get", "key":"readonly" }"#;
const GET_WRITEONLY: &'static str =  r#"{ "type":"get", "key":"writeonly" }"#;
const GET_PROP_WRITE: &'static str = r#"{ "type":"get", "key":"set_prop" }"#;
const GET_NO_KEY: &'static str = r#"{ "type":"get" }"#;
const GET_NO_PERMS: &'static str = r#"{ "type":"get",   "key":"noperms" }"#;

const SET_OBJECT: &'static str
    = r#"{ "type":"set", "key":"bool", "value":false }"#;
const SET_PROP: &'static str
    = r#"{ "type":"set", "key":"prop", "value":null }"#;
const SET_READONLY: &'static str
    = r#"{ "type":"set", "key":"readonly", "value":12 }"#;
const SET_WRITEONLY: &'static str
    = r#"{ "type":"set", "key":"writeonly", "value":11 }"#;
const SET_PROP_READ: &'static str
    = r#"{ "type":"set", "key":"prop_read", "value":"asdf" }"#;
const SET_NEW_KEY: &'static str
    = r#"{ "type":"set", "key":"new_key", "value":"value" }"#;
const SET_NO_PERMS: &'static str
    = r#"{ "type":"set", "key":"noperms", "value":"something" }"#;

const EXISTS_OBJECT_RW: &'static str = r#"{"type":"exists", "key":"null" }"#;
const EXISTS_PROP: &'static str = r#"{"type":"exists", "key":"prop" }"#;
const EXISTS_PROP_READ: &'static str = r#"{"type":"exists","key":"get_prop"}"#;
const EXISTS_PROP_WRITE: &'static str = r#"{"type":"exists","key":"set_prop"}"#;
const EXISTS_OBJECT_READ:&'static str = r#"{"type":"exists","key":"readonly"}"#;
const EXISTS_OBJECT_WRITE:&'static str=r#"{"type":"exists","key":"writeonly"}"#;
const EXISTS_NO_KEY: &'static str = r#"{ "type": "exists", "key": "nope"}"#;

const BAD_NO_REQUEST: &'static str = r#"{ "key":"foo", "value":"bar" }"#;
const BAD_INVALID_REQUEST: &'static str = r#"{ "type":"foo" }"#;
const BAD_GET_NO_KEY: &'static str = r#"{ "type":"get" }"#;
const BAD_SET_NO_KEY: &'static str = r#"{ "type":"set", "value":12 }"#;
const BAD_SET_NO_VALUE: &'static str = r#"{ "type":"set", "key":"f64" }"#;
const BAD_RUN_NO_COMMAND: &'static str = r#"{ "type":"run" }"#;
const BAD_RUN_EXTRA_FIELDS: &'static str =
    r#"{ "type":"run", "key":"command", "foo":"bar" }"#;
const BAD_RUN_U64_COMMAND: &'static str = r#"{ "type":"run", "key":12 }"#;
const BAD_GET_OBJECT_KEY: &'static str = r#"{ "type":"get", "key": 12 }"#;
const BAD_REQUEST_IS_A_NUMBER: &'static str = r#"23"#;

macro_rules! reply {
    ($json:expr) => {
        match super::super::command::reply(
            Json::from_str($json).expect("Couldn't parse json")) {
            Ok(json) => json,
            Err(json) => {
                panic!(format!("Bad reply(): got {}", json.to_string()))
            }
        }
    }
}

#[test]
fn ping() {
    let reply = reply!(PING);
    assert_eq!(reply, channel::success_json());
}

#[test]
fn commands() {
    let mut reply = reply!(COMMANDS);

    let mut obj = reply.as_object_mut()
        .expect("commands: reply not object");
    let mut value = obj.remove("value")
        .expect("commands: reply: no 'value'");
    let val_arr = value.as_array_mut()
        .expect("commands: reply: 'value' not array");
    let mut set = HashSet::new();
    for val in val_arr {
        set.insert(val.as_string()
                   .expect("commands: reply: 'value' not string array"));
    }
    assert_eq!(set.len(), 7);
    let desired = &["get", "set", "exists", "run", "version", "commands", "ping"];
    for com in desired.iter() {
        assert!(set.contains(com), "Missing command {}", com);
    }
}

#[test]
fn version() {
    let mut reply = reply!(VERSION);
    let version = reply.as_object_mut().expect("version: reply not object")
        .remove("value").expect("version: reply: no 'value'")
        .as_u64().expect("version: reply not u64");

    assert_eq!(version, super::super::VERSION);
}

//#[test]
fn run_command() {
    //wait_for_commands();
    let reply = reply!(RUN_COMMAND);
    assert_eq!(reply, channel::success_json());
}

// Note: If a command were to panic in way-cooler, the thread servicing that
// IPC connection would shut down. However, reply! is routing around that and
// everything is being run in each test thread. This is more of a general way
// of making sure commands are being run and we are aware of them.

//#[test]
//#[should_panic("panic_command panic")]
fn run_panic_command() {
    let reply = reply!(RUN_PANIC_COMMAND);
}
