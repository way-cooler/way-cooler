//! Utilities for handling layout operations via dbus.
//!
//! Contains mostly basic parsing methods that return `DBusResult`.

use uuid::Uuid;
use dbus::tree::MethodErr;

use super::{DBusResult};

use layout::{Direction, Layout, Tree, try_lock_tree};

use rustwlc::{ResizeEdge, RESIZE_TOP, RESIZE_BOTTOM,
              RESIZE_LEFT, RESIZE_RIGHT};

pub fn lock_tree_dbus() -> DBusResult<Tree> {
    match try_lock_tree() {
        Ok(tree) => Ok(tree),
        Err(err) => Err(MethodErr::failed(&format!("{:?}", err)))
    }
}

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
    match text.to_lowercase().as_str() {
        "up" => Ok(Direction::Up),
        "down" => Ok(Direction::Down),
        "left" => Ok(Direction::Left),
        "right" => Ok(Direction::Right),
        _ => Err(MethodErr::invalid_arg(
            &format!("{}: {} is not a valid direction. \
                     May be one of 'up', 'down', 'left', 'right'.", arg, text)))
    }
}

pub fn parse_axis(arg: &'static str, text: &str) -> DBusResult<Layout> {
    match text.to_lowercase().as_str() {
        "vertical" | "v" => Ok(Layout::Vertical),
        "horizontal" | "h" => Ok(Layout::Horizontal),
        _ => Err(MethodErr::invalid_arg(
            &format!("{}: {} is not a valid axis direction. \
                      May be either 'horizontal' or 'vertical'", arg, text)))
    }
}

pub fn parse_edge(dir: &str) -> DBusResult<ResizeEdge> {
    let result = Ok(match dir.to_lowercase().as_str() {
        "up" => RESIZE_TOP,
        "down" => RESIZE_BOTTOM,
        "left" => RESIZE_LEFT,
        "right" => RESIZE_RIGHT,
        _ => return Err(MethodErr::invalid_arg(
        &format!("{} is not a valid direction. \
                  May be one of 'up', 'down', 'left', 'right'.", dir)))
    });
    result
}

