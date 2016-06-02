//! Tests individual commands via `reply`

use std::collections::HashSet;

use rustc_serialize::json::Json;

use super::*;

use super::super::channel;

const PING: &'static str = r#"{ "type":"ping" }"#;
const COMMANDS: &'static str = r#"{ "type":"commands" }"#;
const VERSION: &'static str = r#"{ "type":"version" }"#;

const RUN_COMMAND: &'static str = r#"{ "type":"ping" }"#;
const RUN_PROP_GET: &'static str = r#"{ "type":"ping" }"#;
const RUN_OBJECT: &'static str = r#"{ "type":"ping" }"#;
const RUN_NO_KEY: &'static str = r#"{ "type":"ping" }"#;

const GET_OBJECT: &'static str = r#"{ "type":"ping" }"#;
const GET_PROP: &'static str = r#"{ "type":"ping" }"#;
const GET_READONLY: &'static str = r#"{ "type":"ping" }"#;
const GET_WRITEONLY: &'static str = r#"{ "type":"ping" }"#;
const GET_PROP_WRITE: &'static str = r#"{ "type":"ping" }"#;
const GET_NO_KEY: &'static str = r#"{ "type":"ping" }"#;
const GET_NO_PERMS: &'static str = "";

const SET_OBJECT: &'static str = r#"{ "type":"ping" }"#;
const SET_PROP: &'static str = r#"{ "type":"ping" }"#;
const SET_READONLY: &'static str = r#"{ "type":"ping" }"#;
const SET_WRITEONLY: &'static str = r#"{ "type":"ping" }"#;
const SET_PROP_READ: &'static str = r#"{ "type":"ping" }"#;
const SET_NEW_KEY: &'static str = r#"{ "type":"ping" }"#;
const SET_NO_PERMS: &'static str = "";

const EXISTS_OBJECT_RW: &'static str = r#"{}"#;
const EXISTS_PROP: &'static str = r#"{}"#;
const EXISTS_PROP_READ: &'static str = r#"{}"#;
const EXISTS_PROP_WRITE: &'static str = r#"{}"#;
const EXISTS_OBJECT_READ: &'static str = r#"{}"#;
const EXISTS_OBJECT_WRITE: &'static str = r#"{}"#;
const EXISTS_NO_KEY: &'static str =
    r#"{ "type": "exists", "key": "test.no_key"}"#;

const BAD_NO_REQUEST: &'static str = r#"{}"#;
const BAD_INVALID_REQUEST: &'static str = r#"{}"#;
const BAD_GET_NO_KEY: &'static str = r#"{}"#;
const BAD_SET_NO_KEY: &'static str = r#"{}"#;
const BAD_SET_NO_VALUE: &'static str = r#"{}"#;
const BAD_RUN_NO_COMMAND: &'static str = r#"{}"#;
const BAD_RUN_EXTRA_FIELDS: &'static str = r#"{}"#;
const BAD_RUN_U64_COMMAND: &'static str = r#"{}"#;
const BAD_GET_OBJECT_KEY: &'static str = r#"{}"#;
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
