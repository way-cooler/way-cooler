//! Utilities for handling layout operations via dbus.
//!
//! Contains mostly basic parsing methods that return `DBusResult`.

use uuid::Uuid;
use dbus::tree::MethodErr;

use super::{DBusResult};

use layout::{Direction};

/// Parses a uuid from a string, returning `MethodErr::invalid_arg`
/// if the uuid is invalid.
pub fn parse_uuid(arg: &'static str, text: &str) -> DBusResult<Option<Uuid>> {
    if text == "" {
        Ok(None)
    } else {
        match Uuid::parse_str(text) {
            Ok(uuid) => Ok(Some(uuid)),
            Err(reason) => Err(MethodErr::invalid_arg(
                &format!("{}: {} is not a valid UUID: {:?}", arg, text, reason)))
        }
    }
}

/// Parses a `Direction` from a string, returning `MethodErr::invalid_arg`
/// if the string is invalid.
pub fn parse_direction(arg: &'static str, text: &str) -> DBusResult<Direction> {
    match &*text.to_lowercase() {
        "up" => Ok(Direction::Up),
        "down" => Ok(Direction::Down),
        "left" => Ok(Direction::Left),
        "right" => Ok(Direction::Right),
        other => Err(MethodErr::invalid_arg(
            &format!("{}: {} is not a valid direction. \
                     May be one of 'up', 'down', 'left', 'right'.", arg, text)))
    }
}

