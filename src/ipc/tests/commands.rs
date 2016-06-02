//! Tests individual commands via `reply`

use std::collections::HashSet;

use rustc_serialize::json::Json;

use super::*;

use super::super::channel;

const PING: &'static str = r#"{ "type":"ping" }"#;
const COMMANDS: &'static str = r#"{ "type":"commands" }"#;
const VERSION: &'static str = r#"{ "type":"ping" }"#;
const RUN_COMMAND: &'static str = r#"{ "type":"ping" }"#;
const RUN_PROP_GET: &'static str = r#"{ "type":"ping" }"#;
const RUN_OBJECT: &'static str = r#"{ "type":"ping" }"#;
const RUN_NO_KEY: &'static str = r#"{ "type":"ping" }"#;
const GET_OBJECT: &'static str = r#"{ "type":"ping" }"#;

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

    let mut obj = reply.as_object_mut().expect("reply not object");
    let mut value = obj.remove("value").expect("reply: no 'value'");
    let val_arr = value.as_array_mut().expect("reply: 'value' not array");
    let mut set = HashSet::new();
    for val in val_arr {
        set.insert(val.as_string().expect("reply: 'value' not string array"));
    }
    assert_eq!(set.len(), 7);
    let desired = &["get", "set", "exists", "run", "version", "commands", "ping"];
    for com in desired.iter() {
        assert!(set.contains(com), "Missing command {}", com);
    }
}
