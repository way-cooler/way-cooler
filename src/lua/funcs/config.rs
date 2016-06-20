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
    
}

pub fn register_key_command(mods: AnyLuaValue, key: String, command: String)
                            -> Result<(), String> {
    
}

pub fn register_key_function(mods: AnyLuaValue, key: String, id: String)
                            -> Result<(), String> {
    
}
