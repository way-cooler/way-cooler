//! Lua functionality

mod config;
pub mod rust_interop;
mod types;
mod utils;

use self::config::load_config;
pub use self::types::{LuaQuery, LuaResponse};
pub use self::utils::*;

use glib::MainLoop;
use rlua;

use std::cell::RefCell;
use std::cell::Cell;

use common::signal;

thread_local! {
    // NOTE The debug library does some powerful reflection that can do crazy things,
    // which is why it's unsafe to load.

    /// Global Lua state.
    pub static LUA: RefCell<rlua::Lua> = RefCell::new(unsafe { rlua::Lua::new_with_debug() });

    /// If set then we have restarted the Lua thread. We need to replace LUA when it's not borrowed.
    pub static NEXT_LUA: Cell<bool> = Cell::new(false);

    /// Main GLib loop
    static MAIN_LOOP: RefCell<MainLoop> = RefCell::new(MainLoop::new(None, false));
}

/// Sets up the Lua environment before running the compositor.
pub fn setup_lua() {
    LUA.with(|lua| {
        rust_interop::register_libraries(&*lua.borrow())
            .expect("Could not register lua libraries");
        info!("Initializing lua...");
        load_config(&mut *lua.borrow_mut());
    });
}

fn emit_refresh(lua: &rlua::Lua) {
    if let Err(err) = signal::global_emit_signal(lua, ("refresh".to_owned(), rlua::Value::Nil)) {
        error!("Internal error while emitting 'refresh' signal: {}", err);
    }
}

/// Main loop of the Lua thread:
///
/// * Initialise the Lua state
/// * Run a GMainLoop
pub fn enter_glib_loop() {
    MAIN_LOOP.with(|main_loop| main_loop.borrow().run());
}

pub fn terminate() {
    MAIN_LOOP.with(|main_loop| main_loop.borrow().quit())
}
