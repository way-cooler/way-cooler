//! Contains code to handle an IPC channel which is issuing commands.

use std::io::Error as IOError;
use std::io::prelude::*;

use std::mem::transmute;
use std::mem::drop;

use std::collections::BTreeMap;

use rustc_serialize::Encodable;
use rustc_serialize::json::{Json, ToJson, Encoder,
                            encode, ParserError, EncoderError};

use unix_socket::UnixStream;

use registry;
use registry::RegistryError;

/// Reasons a client message might be erroneous
#[derive(Debug)]
enum ResponseError {
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
    let response_tree = BTreeMap::new();
    response_tree.insert("type".to_string(), Json::String("error".to_string()));
    response_tree.insert("reason".to_string(), Json::String(reason));

    write_packet(stream, &Json::Object(response_tree))
}

pub fn handle_command<S: Read + Write>(mut stream: S) {
    loop {
        let maybe_packet = read_packet(&mut stream);
        let response = command_response(maybe_packet);
        match response {
            Ok(json) => { write_packet(&mut stream, &json).expect("Unable to reply!"); },
            Err(action) => match action {
                QueryReply::SendError(err) => {
                    send_error(err.to_string()).expect("Unable to reply!");
                },
                QueryReply::MissingKey(key) => {
                    
                }
                QueryReply::DropConnection => { return; }
            }
        }
    }
}

/// A Json representing a success packet
pub fn success_json() -> Json {
    let response_tree = BTreeMap::new();
    response_tree.insert("type".to_string(), Json::String("success".to_string()));
    Json::Object(response_tree)
}

// A Json representing a value packet
pub fn value_json(value: Json) -> Json {
    let response_tree = success_json().as_object()
        .expect("success_json didn't return object");

    response_tree.insert("value".to_string(), value);
    Json::Object(response_tree)
}

/// Attempts to get a String key off of a Json
pub fn expect_key(source: &BTreeMap<String, Json>, name: &'static str)
                  -> Result<String, QueryReply> {
    let key = try!(source.get(name)
                   .ok_or(QueryReply::MissingKey(name)));
    let value = try!(key.as_string()
                     .ok_or(QueryReply::WrongKeyType(name, "string")));
    return Ok(value);
}

/// Reply messages
#[derive(Debug, Clone, PartialEq)]
enum QueryReply {
    SendError(String),
    MissingKey(&'static str),
    WrongKeyType(&'static str, &'static str),
    DropConnection
}

/// Generates the response needed to a given command
fn command_response(input: Result<Json, ResponseError>)
                    -> Result<Json, QueryReply> {
    use self::QueryReply::*;
    use registry::RegistryError::*;

    let json = try!(input.map_err(|e| match e {
        ResponseError::IO(err) => {
            warn!("IO error communicating with client: {}", err);
            DropConnection
        },
        ResponseError::ConnectionClosed => DropConnection,
        ResponseError::InvalidJson => SendError("invalid json"),
        ResponseError::UnableToFormat => unimplemented!()
    }));
    let object = try!(json.as_object().ok_or(SendError("invalid request")));
    let request_type = try!(object.get("type").ok_or(SendError("invalid request")));

    // Converts the string to a str in the most Rustic way possible
    match &**request_type {
        // Registry
        "get" => {
            let key = try!(expect_key());

            let data = try!(registry::get_data(key).map_err(|e| match e {
                KeyNotFound => SendError("key not found"),
                InvalidOperation => SendError("invalid operation"),
                WrongKeyType => SendError("invalid operation; use 'run'"),
                _ => unimplemented!()
            })).resolve();
            Ok(value_json(data))
        },
        "set" => {
            let key = try!(object.get("key")
                           .ok_or(MISSING_KEY).as_string().ok_or(WRONG_TYPE_KEY));
            let value = try!(object.get("value")
                             .ok_or(SendError("missing field 'value'")));
            // TODO FIXME properly implement flags
            let mut applied_flags = AccessFlags::all();
            if let Some(json_flags) = object.get("flags") {
                // Parse custom flags
                return SendError("flags is unimplemented!");
            }
            try!(registry::set_json(key, applied_flags, value)
                 .map_err(|e| match e {
                     InvalidOperation => SendError("invalid operation"),
                     WrongKeyType => SendError("invalid operation; use 'run'"),
                     _ => unimplemented!()
                 }));
            Ok(success_json())
        },
        "exists" => {
            let key = try!(object.get("key")
                           .ok_or(SendError("missing field 'key'")));

            let reg_key = registry::contains_key(key);

            Ok(value_json(Json::Boolean(reg_key)))
        },

        // Commands
        "run" => {
            let key = try!(obkect.get("key")
                           .ok_or(SendError("missing field 'key'")));

            let command = try!(registry::get_command(key).map_err(|e| match e {
                KeyNotFound => SendError("key not found"),
                WrongKeyType => SendError("invalid operation; use 'get'/'set'"),
                InvalidOperation => SendError("invalid operation"),
                _ => unimplemented!()
            }));

            command();

            Ok(success_json())
        },

        // Meta/API commands
        "version" => {
            Ok(value_json(Json::String(ipc::VERSION)))
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

pub fn handle_event<S: Read + Write>(mut stream: S) {
    
}
