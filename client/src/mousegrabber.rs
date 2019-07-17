//! AwesomeWM Mousegrabber interface

use rlua::{self, Function, Value};

use crate::{
    wayland_obj::{grab_mouse, release_mouse},
    LUA
};

pub const MOUSEGRABBER_TABLE: &str = "mousegrabber";
const MOUSEGRABBER_CALLBACK: &str = "__callback";

/// Init the methods defined on this interface
pub fn init(lua: rlua::Context) -> rlua::Result<()> {
    let mousegrabber_table = lua.create_table()?;
    mousegrabber_table.set("run", lua.create_function(run)?)?;
    mousegrabber_table.set("stop", lua.create_function(stop)?)?;
    mousegrabber_table.set("isrunning", lua.create_function(isrunning)?)?;

    let globals = lua.globals();
    globals.set(MOUSEGRABBER_TABLE, mousegrabber_table)
}

pub fn mousegrabber_handle(
    x: i32,
    y: i32,
    button: Option<(u32, u32)>
) -> rlua::Result<()> {
    LUA.with(|lua| {
        let lua = lua.borrow();
        let button_events = button
            .map(|(button, button_state)| {
                crate::lua::mouse_events_to_lua(&*lua, button, button_state)
            })
            .unwrap_or_else(|| Ok(vec![false; 5]))?;
        lua.context(|lua_ctx| call_mousegrabber(lua_ctx, (x, y, button_events)))
    })
}

fn call_mousegrabber(
    lua: rlua::Context,
    (x, y, button_events): (i32, i32, Vec<bool>)
) -> rlua::Result<()> {
    let lua_callback = match lua
        .named_registry_value::<str, Function>(MOUSEGRABBER_CALLBACK)
    {
        Ok(function) => function,
        _ => return Ok(())
    };
    let res_table = lua.create_table()?;
    res_table.set("x", x)?;
    res_table.set("y", y)?;
    res_table.set("buttons", button_events)?;
    match lua_callback.call(res_table)? {
        Value::Boolean(true) => Ok(()),
        _ => stop(lua, ())
    }
}

fn run<'lua>(
    lua: rlua::Context<'lua>,
    (function, cursor): (Function<'lua>, String)
) -> rlua::Result<()> {
    match lua.named_registry_value::<str, Value>(MOUSEGRABBER_CALLBACK)? {
        Value::Function(_) => Err(rlua::Error::RuntimeError(
            "mousegrabber callback already set!".into()
        )),
        _ => {
            lua.set_named_registry_value(MOUSEGRABBER_CALLBACK, function)?;
            grab_mouse(cursor);
            Ok(())
        }
    }
}

fn stop(lua: rlua::Context, _: ()) -> rlua::Result<()> {
    release_mouse();
    lua.set_named_registry_value(MOUSEGRABBER_CALLBACK, Value::Nil)
}

fn isrunning(lua: rlua::Context, _: ()) -> rlua::Result<bool> {
    match lua.named_registry_value::<str, Value>(MOUSEGRABBER_TABLE)? {
        Value::Function(_) => Ok(true),
        _ => Ok(false)
    }
}
