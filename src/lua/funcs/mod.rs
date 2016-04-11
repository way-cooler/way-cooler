//! Rust functions for lua libraries

use hlua;
use hlua::{Lua, LuaTable, Function};

// Prevent functions declared here from not being registered
#[forbid(dead_code)]
#[forbid(unused_variables)]

mod input;

/// Register all the Rust functions for the lua libraries
pub fn register_libraries(lua: &mut Lua) {
    // Yeah, need to access individual tables

    let mut wm: LuaTable<_> = lua.get("wm").unwrap();
    let mut pointer: LuaTable<_> = wm.get("pointer").unwrap();

    pointer.set("get_position", hlua::function0(input::pointer_get_position));
    pointer.set("set_position", hlua::function2(input::pointer_set_position));
}
