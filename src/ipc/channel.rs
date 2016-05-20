//! Contains code to handle an IPC channel which is issuing commands.

use std::io::Error as IOError;
use std::io::prelude::*;

use std::mem::transmute;
use std::mem::drop;

use rustc_serialize::Encodable;
use rustc_serialize::json::{Json, ToJson, Encoder, encode, ParserError, EncoderError};

use unix_socket::UnixStream;

/// Reasons a client message might be erroneous
#[derive(Debug)]
enum ResponseError {
    /// Connection was closed
    ConnectionClosed,
    /// Some bytes dun goofed
    InvalidString,
    /// There were IO issues
    IO(IOError),
    /// Json was invalid
    InvalidJson(ParserError),
    /// Couldn't format Json server-side
    UnableToFormat(EncoderError)
}

/// Receives a packet from the given stream.
fn receive_packet(stream: &mut Read) -> Result<Json, ResponseError> {
    let mut buffer = [0u8; 4];
    try!(stream.read_exact(&mut buffer).map_err(ResponseError::IO));
    // This is what the byteorder crate does (needs testing)
    let len: u32 = u32::from_be(unsafe { transmute(buffer) }); drop(buffer);
    trace!("Listening for packet of length {}", len);
    return Json::from_reader(&mut stream.take(len as u64))
        .map_err(ResponseError::InvalidJson);
}

fn write_packet(stream: &mut Write, packet: &Json) -> Result<(), ResponseError> {
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

pub fn handle_client(mut stream: UnixStream) {
    println!("Starting connection.");

    // Listen for starting connection
}

fn command(mut stream: UnixStream) {
    
}

fn event(mut stream: UnixStream) {
    
}
