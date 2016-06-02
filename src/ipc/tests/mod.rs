//! Tests for IPC

use rustc_serialize::json::Json;

// Unit tests for methods in channel.rs
mod channel;
// Unit tests for individual packets in command.rs
mod commands;

// Integration tests for running the command socket
mod command_loop;

/// Parse text into Json
#[macro_export]
macro_rules! json {
    ($text:expr) => {
        match Json::from_str($text) {
            Ok(json) => json,
            Err(err) =>
                panic!(format!("json! failed with input {} - {:?}", $text, err))
        }
    }
}

#[inline]
pub fn json_eq(json: Json, text: &str) -> bool {
    json == json!(text)
}
