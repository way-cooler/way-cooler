//! TODO Fill in

use std::fmt::{self, Display, Formatter};
use std::default::Default;
use rlua::{self, Table, Lua, UserData, ToLua, Value, UserDataMethods, MetaMethod, AnyUserData};

const INDEX_MISS_FUNCTION: &'static str = "__index_miss_function";
const NEWINDEX_MISS_FUNCTION: &'static str = "__newindex_miss_function";

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

impl UserData for MouseState {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        methods.add_meta_function(MetaMethod::Index, index);
    }
}

pub fn init(lua: &Lua) -> rlua::Result<()> {
    let mouse_table = lua.create_table()?;
    let meta_table = lua.create_table()?;
    let mouse = lua.create_userdata(MouseState::default())?;
    method_setup(lua, &mouse_table)?;
    let globals = lua.globals();
    mouse_table.set_metatable(Some(meta_table));
    mouse.set_user_value(mouse_table)?;
    globals.set("mouse", mouse)
}

fn method_setup(lua: &Lua, mouse_table: &Table) -> rlua::Result<()> {
    mouse_table.set("coords", lua.create_function(coords)?)?;
    mouse_table.set("set_index_miss_handler", lua.create_function(set_index_miss)?)?;
    mouse_table.set("set_newindex_miss_handler", lua.create_function(set_newindex_miss)?)?;
    Ok(())
}


fn coords<'lua>(lua: &'lua Lua, (coords, _ignore_enter): (rlua::Value<'lua>, rlua::Value<'lua>))
                -> rlua::Result<Table<'lua>> {
    match coords {
        rlua::Value::Table(coords) => {
            let (x, y): (i32, i32) = (coords.get("x")?, coords.get("y")?);
            // TODO The ignore_enter is supposed to not send a send event to the client
            // That's not possible, at least until wlroots is complete.
            //
            // FIXME Actually do this
            //input::pointer::set_position_v2(x, y);
            Ok(coords)
        },
        _ => {
            // get the coords
            let coords = lua.create_table()?;
            // TODO FIXME Do this for real
            let (x, y) = (0, 0);
            //let (x, y) = input::pointer::get_position_v2();
            coords.set("x", x as i32)?;
            coords.set("y", y as i32)?;
            // TODO It expects a table of what buttons were pressed.
            coords.set("buttons", lua.create_table()?)?;
            Ok(coords)
        }
    }
}

fn set_index_miss(lua: &Lua, func: rlua::Function) -> rlua::Result<()> {
    let button = lua.globals().get::<_, AnyUserData>("button")?;
    let table = button.get_user_value::<Table>()?;
    table.set(INDEX_MISS_FUNCTION, func)
}

fn set_newindex_miss(lua: &Lua, func: rlua::Function) -> rlua::Result<()> {
    let button = lua.globals().get::<_, AnyUserData>("button")?;
    let table = button.get_user_value::<Table>()?;
    table.set(NEWINDEX_MISS_FUNCTION, func)
}

fn index<'lua>(lua: &'lua Lua,
               (mouse, index): (AnyUserData<'lua>, Value<'lua>))
               -> rlua::Result<Value<'lua>> {
    use super::screen::SCREENS_HANDLE;
    let obj_table = mouse.get_user_value::<Table>()?;
    match index {
        Value::String(ref string) => {
            let string = string.to_str()?;
            if string != "screen" {
                return obj_table.get(string)
                // TODO call miss index handler if it exists
            }
            // TODO Might need a more robust way to get the current output...
            // E.g they look at where the cursor is, I don't think we need to do that.

            // TODO FIXME Actually do this with wlroots
            //let index = WlcOutput::list().iter()
            //    .position(|&output| output == WlcOutput::focused())
            //    // NOTE Best to just lie because no one handles nil screens properly
            //    .unwrap_or(0);
            //let screens = lua.named_registry_value::<Vec<AnyUserData>>(SCREENS_HANDLE)?;
            //if index < screens.len() {
            //    return screens[index].clone().to_lua(lua)
            //}
            // TODO Return screen even in bad case, see how awesome does it for maximal compatibility
        },
        _ => {}
    }
    return obj_table.get(index)
}
