//! Contains code to handle an IPC channel which is issuing commands.

use std::io::Error as IOError;
use std::io::prelude::*;

use std::mem::transmute;

use std::collections::BTreeMap;

use rustc_serialize::json::{Json, encode, ParserError, EncoderError};


/// Errors which arise from sending a message
#[derive(Debug)]
pub enum SendError {
    /// IO Error
    IO(IOError),
    /// JSON encoding error - less likely
    Encoding(EncoderError)
}

/// Reasons a client message might be erroneous
#[derive(Debug)]
pub enum ReceiveError {
    /// There were IO issues
    IO(IOError),
    /// Json was invalid
    InvalidJson(ParserError),
}

pub type SendResult = Result<(), SendError>;
pub type ReceiveResult = Result<Json, ReceiveError>;

/// Converts a u32 to its byte representation
#[inline]
pub fn u32_to_bytes(input: u32) -> [u8; 4] {
    unsafe { transmute(input.to_be()) }
}

/// Parses a set of bytes into a u32
#[inline]
pub fn u32_from_bytes(bytes: [u8; 4]) -> u32 {
    u32::from_be(unsafe { transmute(bytes) })
}

/// Receives a packet from the given stream.
pub fn read_packet(stream: &mut Read) -> ReceiveResult {
    let mut buffer = [0u8; 4];
    try!(stream.read_exact(&mut buffer).map_err(ReceiveError::IO));
    let len = u32_from_bytes(buffer);
    trace!("Listening for packet of length {}", len);
    return Json::from_reader(&mut stream.take(len as u64))
        .map_err(ReceiveError::InvalidJson);
}

/// Writes a packet to the given stream
pub fn write_packet(stream: &mut Write, packet: &Json) -> SendResult {
    let json_string = try!(encode(packet).map_err(SendError::Encoding));
    trace!("Writing packet of length {}: {}", json_string.len(), json_string);
    if json_string.len() > ::std::u32::MAX as usize {
        panic!("Attempted to send reply too big for the channel!");
    }
    let len = json_string.len() as u32;
    let len_bytes = u32_to_bytes(len);
    try!(stream.write_all(&len_bytes).map_err(SendError::IO));
    stream.write_all(json_string.as_bytes()).map_err(SendError::IO)
}

pub fn error_json(reason: String) -> Json {
    let mut json = BTreeMap::new();
    json.insert("type".to_string(), Json::String("error".to_string()));
    json.insert("reason".to_string(), Json::String(reason));
    Json::Object(json)
}

/// Send an error message with additional fields
pub fn error_json_with(reason: String,
                       others: BTreeMap<String, Json>) -> Json {
    if let Json::Object(mut json) = error_json(reason) {
        for (key, value) in others.into_iter() {
            json.insert(key, value);
        }
        return Json::Object(json)
    }
    unreachable!()
}

pub fn error_expecting_key(key: &'static str, type_: &'static str) -> Json {
    let mut others = BTreeMap::new();
    others.insert("missing".to_string(), Json::String(key.to_string()));
    others.insert("expected".to_string(), Json::String(type_.to_string()));
    error_json_with("message field not found".to_string(), others)
}

/// A Json representing a success packet
pub fn success_json() -> Json {
    let mut responses = BTreeMap::new();
    responses.insert("type".to_string(), Json::String("success".to_string()));
    Json::Object(responses)
}

/// Send an error message with additional fields
pub fn success_json_with(others: BTreeMap<String, Json>) -> Json {
    if let Json::Object(mut json) = success_json() {
        for (key, value) in others.into_iter() {
            json.insert(key, value);
        }
        return Json::Object(json)
    }
    unreachable!()
}

// A Json representing a value packet
pub fn value_json(value: Json) -> Json {
    if let Json::Object(mut responses) = success_json() {
        responses.insert("value".to_string(), value);
        return Json::Object(responses);
    }
    unreachable!()
}
