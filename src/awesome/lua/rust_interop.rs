//! Rust code which is called from lua in the init file

use awesome::{self, convert::json_to_lua};
use rlua::{self, prelude::LuaResult};
use rustc_serialize::json::ToJson;
use uuid::Uuid;

use super::{send, LuaQuery};

/// We've `include!`d the code which initializes from the Lua side.

/// Register all the Rust functions for the lua libraries
pub fn register_libraries(lua: &rlua::Lua) -> LuaResult<()> {
    trace!("Setting up Lua libraries");
    // TODO Is this awesome init code necessary?
    let init_code = include_str!("../../../lib/lua/init.lua");
    lua.exec::<()>(init_code, Some("init.lua"))?;
    awesome::init(&lua).expect("Could not initialize awesome compatibility modules");
    Ok(())
}

/// This function behaves just like Lua's built-in type() function, but also recognises classes and
/// returns special names for them.
fn type_override(_lua: &rlua::Lua, arg: rlua::Value) -> Result<String, rlua::Error> {
    use rlua::Value;

    // Lua's type() returns the result of lua_typename(), but rlua does not make
    // that available to us, so write our own.
    Ok(match arg {
           Value::Error(e) => return Err(e),
           Value::Nil => "nil",
           Value::Boolean(_) => "boolean",
           Value::LightUserData(_) => "userdata",
           Value::Integer(_) => "number",
           Value::Number(_) => "number",
           Value::String(_) => "string",
           Value::Function(_) => "function",
           Value::Thread(_) => "thread",
           Value::Table(_) => "table",
           Value::UserData(o) => {
               // Handle our own objects specially: Get the object's class from its user
               // value's metatable's __class entry. Then get the class name
               // from the class's user value's metatable's name entry.
               return o.get_user_value::<rlua::Table>().ok()
                       .and_then(|table| table.get_metatable())
                       .and_then(|meta| meta.raw_get::<_, rlua::AnyUserData>("__class").ok())
                       .and_then(|class| class.get_user_value::<rlua::Table>().ok())
                       .map(|table| table.raw_get("name"))
                       .unwrap_or_else(|| Ok("userdata".into()))
           }
       }.into())
}
