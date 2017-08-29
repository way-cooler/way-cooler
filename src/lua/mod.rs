//! Lua functionality

use rlua::Lua;
use std::sync::Mutex;

#[cfg(test)]
mod tests;

mod types;
mod thread;
mod rust_interop;
mod init_path;

pub struct LuaWrapper(pub Lua);

unsafe impl Send for LuaWrapper{}


lazy_static! {
    pub static ref LUA: Mutex<LuaWrapper> = Mutex::new(LuaWrapper(Lua::new()));
}

pub use self::types::{LuaQuery, LuaResponse};
pub use self::thread::{init, on_compositor_ready, running, send, update_registry_value,
                       LuaSendError};
