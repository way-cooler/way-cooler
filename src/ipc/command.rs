//! Command part of IPC

use std::io::prelude::*;
use std::collections::BTreeMap;

use rustc_serialize::json::{encode, Json, ToJson, ParserError};

use super::channel::{self, ReceiveError};

use registry;
use registry::AccessFlags;

macro_rules! expect_key {
    ($in_json:expr; $name:expr, $typ:ident) => {
        match $in_json.remove($name) {
            Some(key) => match key {
                Json::$typ(var) => var,
                _ => return Err(channel::error_expecting_key($name,
                          stringify!($typ)))
            },
            None => return Err(channel::error_expecting_key($name,
            stringify!($typ)))
        }
    }
}

/// Run a thread reading and replying to queries
pub fn thread<S: Read + Write>(mut stream: S) {
    loop {
        match channel::read_packet(&mut stream) {
            Ok(packet) => {
                trace!("Read packet: {}", encode(&packet)
                    .unwrap_or("<packet that was read already??>".to_string()));
                let reply = reply(packet).unwrap_or_else(|e| e);
                trace!("Writing reply: {}", encode(&reply)
                       .unwrap_or("<a reply which is not writable>".to_string()));
                channel::write_packet(&mut stream, &reply)
                    .expect("response: Unable to reply!");
            },
            Err(read_err) => match read_err {
                ReceiveError::IO(ioerr) => {
                    warn!("Unable to read reply! Closing connection.");
                    debug!("Got IOError: {:?}", ioerr);
                    return;
                },
                ReceiveError::InvalidJson(parse_err) => match parse_err {
                    ParserError::SyntaxError(code, start, end) => {
                        let reply = Json::Object(json_object!{
                            "type" => "error",
                            "reason" => "invalid json",
                            "code" => (format!("{:?}", code)),
                            "start" => (start as u64),
                            "end" => (end as u64)
                        });
                        channel::write_packet(&mut stream, &reply)
                            .expect("invalid syntax: Unaable to reply!");
                    }
                    _ => unreachable!()
                }
            }
        }
    }
}

/// Generates the response needed to a given command
/// If the request is invalid it returns an Err.
/// If the request is valid but fails it returns an Ok.
pub fn reply(json: Json) -> Result<Json, Json> {
    let mut object: BTreeMap<String, Json>;
    if let Json::Object(obj) = json {
        object = obj;
    }
    else {
        return Err(channel::error_json(
            "invalid format - object required".to_string()))
    }

    let request_type = expect_key!(object; "type", String);

    // Converts the string to a str in the most Rustic way possible
    match &*request_type {
        // Registry
        "get" => {
            use std::ops::Deref;
            use registry::RegistryError::*;

            let key = expect_key!(object; "key", String);

            match registry::get_data(&key) {
                Ok(data) => {
                    let (flags, arc) = data.resolve();
                    let reply = channel::success_json_with(json_object! {
                        "value" => (arc.deref().clone()),
                        "flags" => flags
                    });
                    // I'd return a Cow<Json> because write_packet needs an
                    // &Json, but I dunno how to move the fields over in a
                    // borrow.
                    return Ok(reply);
                },
                Err(err) => match err {
                    KeyNotFound =>
                        return Err(channel::error_json("key not found".to_string())),
                    InvalidOperation|WrongKeyType =>
                        return Err(channel::error_json("invalid operation".to_string())),
                    DecoderError(err) => {
                        error!("Got a decoder error from the registry! {:?}", err);
                        return Err(channel::error_json("key not found".to_string()))
                    }
                }
            }
        },
        "set" => {
            use registry::RegistryError::*;

            let key = expect_key!(&mut object; "key", String);
            let value: Json;
            match object.remove("value") {
                Some(val) => value = val,
                None => return Err(channel::error_expecting_key("value", "any"))
            }

            // TODO FIXME properly implement flags
            let mut applied_flags = AccessFlags::all();
            if let Some(json_flags) = object.get("flags") {
                // Parse custom flags
                let flag_list = try!(json_flags.as_array()
                                     .ok_or(channel::error_expecting_key(
                                         "flags", "string array")));
                let mut flags = AccessFlags::empty();
                for flag in flag_list {
                    let flag_str = try!(flag.as_string()
                                        .ok_or(channel::error_expecting_key(
                                            "flags", "string array")));
                    match flag_str {
                        "read"|"r" => flags.insert(AccessFlags::READ()),
                        "write"|"w" => flags.insert(AccessFlags::WRITE()),
                        _ => return Err(channel::error_json(
                            "Flags can either be 'read' or 'write'".to_string()))
                    }
                }
                applied_flags = flags;
            }
            try!(registry::set_json(key, applied_flags, value)
                 .map_err(|e| match e {
                     InvalidOperation =>
                         channel::error_json("invalid operation 'set'".to_string()),
                     WrongKeyType =>
                         channel::error_json("invalid operation; use 'run'".to_string()),
                     _ => unimplemented!() // TODO clean up registry err enum(s)
                 }));
            Ok(channel::success_json())
        },
        "exists" => {
            let key = expect_key!(&mut object; "key", String);

            let reg_key = registry::contains_key(&key);

            // TODO registry::key_info(key)
            Ok(channel::success_json_with(json_object!{
                "key" => reg_key
            }))
        },

        // Commands
        "run" => {
            use registry::RegistryError::*;
            let key = expect_key!(&mut object; "key", String);

            let command = try!(registry::get_command(&key).map_err(|e| match e {
                KeyNotFound => channel::error_json("field not found".to_string()),
                WrongKeyType =>
                    channel::error_json("invalid operation; use 'get/'set'".to_string()),
                // may not be relevant
                InvalidOperation =>
                    channel::error_json("invalid operation; use 'set'".to_string()),
                _ => unimplemented!()
            }));

            command();

            Ok(channel::success_json())
        },

        // Meta/API commands
        "version" => {
            Ok(channel::value_json(Json::U64(super::VERSION)))
        },

        "commands" => {
            Ok(channel::value_json(json!([
                "get", "set", "exists", "run",
                "version", "commands", "ping"
                    ])))
        },

        "ping" => {
            Ok(channel::success_json())
        },

        _ => Err(channel::error_json("invalid request; see 'commands'".to_string()))
    }
}
