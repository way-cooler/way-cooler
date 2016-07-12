//! Rust functions handled by Lua config.
//! This code is injected via config.set_rust

use commands;
use keys::{self, KeyPress, KeyEvent};
use hlua::Function;

pub struct WorkspaceOptions {
    pub name: Option<String>,
    pub mode: Option<String>
}

pub struct Keybind {
    mods: Vec<String>,
    key: String,
    command: String,
    repeat: bool
}

pub fn init_workspaces(count: i32, options: Vec<(i32, WorkspaceOptions)>)
                       -> Result<(), String> {
    // TODO The tree doesn't accept workspace names yet.
    Err("This function is not yet available!".to_string())
}

/// Registers a command keybinding.
pub fn register_command_key(mods: String, command: String, _repeat: bool)
                            -> Result<(), String> {
    if let Some(press) = keypress_from_string(mods) {
        if let Some(command) = commands::get(&command) {
            keys::register(vec![(press, KeyEvent::Command(command))]);
            Ok(())
        }
        else {
            Err("Command not found".to_string())
        }
    }
    else {
        Err("Invalid keypress".to_string())
    }
}

/// Rust half of registering a Lua key: store the KeyPress in the keys table
/// and send Lua back the index for __key_map.
pub fn register_lua_key(mods: String, repeat: bool) -> Result<String, String> {
    if let Some(press) = keypress_from_string(mods) {
        keys::register(vec![(press.clone(), KeyEvent::Lua)]);
        Ok(press.get_lua_index_string())
    }
    else {
        Err("Invalid keypress".to_string())
    }
}

/// Parses a keypress from a string
pub fn keypress_from_string(mods: String) -> Option<KeyPress> {
    let parts: Vec<&str> = mods.split(',').collect();
    if let Some((ref key, mods)) = parts.split_last() {
        KeyPress::from_key_names(mods, &key).ok()
    }
    else {
        None
    }
}
