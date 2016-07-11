//! Rust functions for lua libraries

use hlua;
use hlua::{Lua, LuaTable};

// Prevent functions declared here from not being registered
#[forbid(dead_code)]
#[forbid(unused_variables)]

mod input;
mod ipc;

const ERR_TABLE_404: &'static str = "Lua thread was not properly initialized!";

/// Register all the Rust functions for the lua libraries
pub fn register_libraries(lua: &mut Lua) {
    // Initialize 'way_cooler'
    init_ipc(lua);
}

#[inline] // It's called once
fn init_ipc(lua: &mut Lua) {
    let ipc_table: LuaTable<_> = lua.get("way_cooler").expect(ERR_TABLE_404);
    let mut meta_ipc = ipc_table.get_or_create_metatable();
    meta_ipc.set("__metatable", "Turtles all the way down");
    meta_ipc.set("__tostring", hlua::function1(ipc::to_string));
    meta_ipc.set("__index", hlua::function2(ipc::index));
    meta_ipc.set("__newindex", hlua::function3(ipc::new_index));
}
