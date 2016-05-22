//! Contains code to handle an IPC channel which is issuing commands.

use std::io::Error as IOError;
use std::io::prelude::*;

use std::mem::transmute;
use std::mem::drop;

use std::collections::BTreeMap;

use rustc_serialize::Encodable;
use rustc_serialize::json::{Json, encode, ParserError, EncoderError};

use registry;
use registry::{RegistryError, AccessFlags};

/// Reasons a client message might be erroneous
#[derive(Debug)]
pub enum ResponseError {
    /// Connection was closed
    ConnectionClosed,
    /// There were IO issues
    IO(IOError),
    /// Json was invalid
    InvalidJson(ParserError),
    /// Couldn't format Json server-side
    UnableToFormat(EncoderError)
}

/// Receives a packet from the given stream.
pub fn read_packet(stream: &mut Read) -> Result<Json, ResponseError> {
    let mut buffer = [0u8; 4];
    try!(stream.read_exact(&mut buffer).map_err(ResponseError::IO));
    let len: u32 = u32::from_be(unsafe { transmute(buffer) }); drop(buffer);
    trace!("Listening for packet of length {}", len);
    return Json::from_reader(&mut stream.take(len as u64))
        .map_err(ResponseError::InvalidJson);
}

/// Writes a packet to the given stream
pub fn write_packet(stream: &mut Write, packet: &Json) -> Result<(), ResponseError> {
    let json_string = try!(encode(packet).map_err(ResponseError::UnableToFormat));
    trace!("Writing packet of length {}: {}", json_string.len(), json_string);
    if json_string.len() > ::std::u32::MAX as usize {
        panic!("Attempted to send reply too big for the channel!");
    }
    let len = (json_string.len() as u32).to_be();
    let len_bytes: [u8; 4] = unsafe { transmute(len) }; drop(len);
    stream.write_all(&len_bytes);
    stream.write_all(json_string.as_bytes()).map_err(ResponseError::IO)
}

/// Send an error message across the given stream
pub fn send_error(stream: &mut Write, reason: String) -> Result<(), ResponseError> {
    let mut responses = BTreeMap::new();
    responses.insert("type".to_string(), Json::String("error".to_string()));
    responses.insert("reason".to_string(), Json::String(reason));

    write_packet(stream, &Json::Object(responses))
}

/// Send an error message with additional fields
pub fn send_error_with(stream: &mut Write, reason: String,
                  mut responses: BTreeMap<String, Json>) -> Result<(), ResponseError> {
    responses.insert("type".to_string(), Json::String("error".to_string()));
    responses.insert("reason".to_string(), Json::String(reason));

    write_packet(stream, &Json::Object(responses))
}

pub fn handle_command<S: Read + Write>(mut stream: S) {
    loop {
        let maybe_packet = read_packet(&mut stream);
        let response = command_response(maybe_packet);
        match response {
            Ok(json) => {
                write_packet(&mut stream, &json)
                    .expect("Unable to reply!");
            },
            Err(action) => match action {
                QueryReply::SendError(err) => {
                    send_error(&mut stream, err.to_string())
                        .expect("Unable to reply!");
                },
                QueryReply::MissingKey(key) => {
                    let mut responses = BTreeMap::new();
                    responses.insert("key".to_string(),
                                     Json::String(key.to_string()));
                    send_error_with(&mut stream, "missing key".to_string(),
                                    responses)
                        .expect("Unable to reply!");
                },
                QueryReply::WrongKeyType(key, key_type) => {
                    let mut responses = BTreeMap::new();
                    responses.insert("key".to_string(),
                                     Json::String(key.to_string()));
                    responses.insert("expected".to_string(),
                                     Json::String(key_type.to_string()));
                    send_error_with(&mut stream, "invalid key type".to_string(),
                                    responses)
                        .expect("Unable to reply!");
                }
                QueryReply::DropConnection => { return; }
            }
        }
    }
}

/// A Json representing a success packet
pub fn success_json() -> Json {
    let mut responses = BTreeMap::new();
    responses.insert("type".to_string(), Json::String("success".to_string()));
    Json::Object(responses)
}

// A Json representing a value packet
fn value_json(value: Json) -> Json {
    if let Json::Object(mut responses) = success_json() {
        responses.insert("value".to_string(), value);
        return Json::Object(responses);
    }
    unimplemented!()
}

/// Attempts to get a String key off of a Json
fn expect_key(source: &BTreeMap<String, Json>, name: &'static str)
                  -> Result<String, QueryReply> {
    let key = try!(source.get(name)
                   .ok_or(QueryReply::MissingKey(name)));
    let value = try!(key.as_string()
                     .ok_or(QueryReply::WrongKeyType(name, "string")));
    return Ok(value.to_string());
}

/// Reply messages
#[derive(Debug, Clone, PartialEq)]
enum QueryReply {
    SendError(&'static str),
    MissingKey(&'static str),
    WrongKeyType(&'static str, &'static str),
    DropConnection
}

/// Generates the response needed to a given command
fn command_response(input: Result<Json, ResponseError>)
                    -> Result<Json, QueryReply> {
    use self::QueryReply::*;

    let json = try!(input.map_err(|e| match e {
        ResponseError::IO(err) => {
            warn!("IO error communicating with client: {}", err);
            DropConnection
        },
        ResponseError::ConnectionClosed => DropConnection,
        ResponseError::InvalidJson(_) => SendError("invalid json"),
        ResponseError::UnableToFormat(_) => unimplemented!()
    }));
    let mut object: BTreeMap<String, Json>;
    if let Json::Object(obj) = json {
        object = obj;
    }
    else {
        return Err(SendError("invalid json, table expected"));
    }
    let request_type = try!(expect_key(&object, "type"));

    // Converts the string to a str in the most Rustic way possible
    match &*request_type {
        // Registry
        "get" => {
            use std::ops::Deref;
            let key = try!(expect_key(&object, "key"));

            let (_flags, data) = try!(registry::get_data(&key).map_err(|e| match e {
                RegistryError::KeyNotFound =>
                    SendError("key not found"),
                RegistryError::InvalidOperation =>
                    SendError("invalid operation"),
                RegistryError::WrongKeyType =>
                    SendError("invalid operation; use 'run'"),
                _ => unimplemented!()
            })).resolve();

            Ok(value_json(data.deref().clone()))
        },
        "set" => {
            let key = try!(expect_key(&object, "key"));
            let value = try!({
                BTreeMap::remove(&mut object, "value").ok_or(MissingKey("value"))
            });

            // TODO FIXME properly implement flags
            let mut applied_flags = AccessFlags::all();
            if let Some(json_flags) = object.get("flags") {
                // Parse custom flags
                let flag_list = try!(json_flags.as_array()
                                     .ok_or(WrongKeyType("flags", "string array")));
                let mut flags = AccessFlags::empty();
                for flag in flag_list {
                    let flag_str = try!(flag.as_string()
                                        .ok_or(WrongKeyType("flags", "string array")));
                    match flag_str {
                        "read"|"r" => flags.insert(AccessFlags::READ()),
                        "write"|"w" => flags.insert(AccessFlags::WRITE()),
                        _ => return Err(SendError("Flags can either be 'read' or 'write'"))
                    }
                }
                applied_flags = flags;
            }
            try!(registry::set_json(key, applied_flags, value)
                 .map_err(|e| match e {
                     RegistryError::InvalidOperation =>
                         SendError("invalid operation"),
                     RegistryError::WrongKeyType =>
                         SendError("invalid operation; use 'run'"),
                     _ => unimplemented!()
                 }));

            Ok(success_json())
        },
        "exists" => {
            let key = try!(expect_key(&object, "key"));

            let reg_key = registry::contains_key(&key);

            Ok(value_json(Json::Boolean(reg_key)))
        },

        // Commands
        "run" => {
            let key = try!(expect_key(&object, "key"));

            let command = try!(registry::get_command(&key).map_err(|e| match e {
                RegistryError::KeyNotFound =>
                    SendError("key not found"),
                RegistryError::WrongKeyType =>
                    SendError("invalid operation; use 'get'/'set'"),
                RegistryError::InvalidOperation =>
                    SendError("invalid operation"),
                _ => unimplemented!()
            }));

            command();

            Ok(success_json())
        },

        // Meta/API commands
        "version" => {
            Ok(value_json(Json::U64(super::VERSION)))
        },
        "commands" => {
            Ok(value_json(Json::Array(vec![
                Json::String("get".to_string()),
                Json::String("set".to_string()),
                Json::String("exists".to_string()),
                Json::String("run".to_string()),
                Json::String("version".to_string()),
                Json::String("commands".to_string()) ])))
        },
        _ => Err(SendError("invalid request"))
    }
}

#[allow(dead_code)]
#[allow(unused_mut)]
pub fn handle_event<S: Read + Write>(mut stream: S) {
    
}
