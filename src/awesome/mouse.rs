//! TODO Fill in

use std::fmt::{self, Display, Formatter};
use std::default::Default;
use rlua::{self, Table, Lua, UserData, ToLua};
use rustwlc::{Point, input};

#[derive(Clone, Debug)]
pub struct MouseState {
    // TODO Fill in
    dummy: i32
}

impl Default for MouseState {
    fn default() -> Self {
        MouseState {
            dummy: 0
        }
    }
}

impl Display for MouseState {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Mouse: {:p}", self)
    }
}

impl UserData for MouseState {}

pub fn init(lua: &Lua) -> rlua::Result<()> {
    let mouse_table = lua.create_table();
    state_setup(lua, &mouse_table)?;
    meta_setup(lua, &mouse_table)?;
    method_setup(lua, &mouse_table)?;
    let globals = lua.globals();
    globals.set("mouse", mouse_table)
}

fn state_setup(lua: &Lua, mouse_table: &Table) -> rlua::Result<()> {
    mouse_table.set("__data", MouseState::default().to_lua(lua)?)
}

fn meta_setup(lua: &Lua, mouse_table: &Table) -> rlua::Result<()> {
    let meta_table = lua.create_table();
    meta_table.set("__tostring", lua.create_function(|_, val: Table| {
        Ok(format!("{}", val.get::<_, MouseState>("__data")?))
    }))?;
    mouse_table.set_metatable(Some(meta_table));
    Ok(())
}

fn method_setup(lua: &Lua, mouse_table: &Table) -> rlua::Result<()> {
    mouse_table.set("coords", lua.create_function(coords))?;
    Ok(())
}


fn coords<'lua>(lua: &'lua Lua, (coords, _ignore_enter): (rlua::Value<'lua>, rlua::Value<'lua>))
                -> rlua::Result<Table<'lua>> {
    match coords {
        rlua::Value::Table(coords) => {
            let (x, y) = (coords.get("x")?, coords.get("y")?);
            // TODO The ignore_enter is supposed to not send a send event to the client
            // That's not possible, at least until wlroots is complete.
            input::pointer::set_position(Point { x, y });
            Ok(coords)
        },
        _ => {
            // get the coords
            let coords = lua.create_table();
            let Point { x, y } = input::pointer::get_position();
            coords.set("x", x)?;
            coords.set("y", y)?;
            // TODO It expects a table of what buttons were pressed.
            coords.set("buttons", lua.create_table())?;
            Ok(coords)
        }
    }
}
