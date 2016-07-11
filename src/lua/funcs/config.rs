//! Rust functions handled by Lua config.
//! This code is injected via config.set_rust

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
}

pub fn register_command_key(mods: String, command: String, repeat: bool)
                            -> Result<(), String> {
}

pub fn register_lua_key(mods: String, repeat: bool) -> Result<(), String> {
    
}

pub fn keymods_index(mods: String)
