//! Contains code to handle an IPC channel which is issuing commands.

use std::io::prelude::*;

use rustc_serialize::Decodable;
use rustc_serialize::json::{Json, ToJson, Decoder, EncoderError};

use unix_socket::UnixStream;

/// Reasons a client message might be erroneous
#[derive(Debug, Clone)]
enum ResponseError {
    /// Connection was closed
    ConnectionClosed,
    /// Json was invalid
    InvalidJson(EncoderError)
}

/// Receives a packet from the given stream.
fn receive_packet(stream: &mut UnixStream) -> Result<Json, ResponseError> {
    Err(ResponseError::ConnectionClosed)
}

pub fn handle_client(mut stream: UnixStream) {
    println!("Starting connection.");

    // Listen for starting connection
}

fn command(mut stream: UnixStream) {
    
}

fn event(mut stream: UnixStream) {
    
}
