//! TODO Fill in

use std::fmt::{self, Display, Formatter};
use std::default::Default;
use std::rc::Rc;
use rlua::{self, Table, Lua, UserData, ToLua, Value};
use super::object::{Object, Objectable};
use super::class::{self, Class, ClassBuilder};

#[derive(Clone, Debug)]
pub struct ScreenState {
    // TODO Fill in
    dummy: i32
}

pub struct Screen<'lua>(Table<'lua>);

impl Default for ScreenState {
    fn default() -> Self {
        ScreenState {
            dummy: 0
        }
    }
}

impl <'lua> Screen<'lua> {
    fn new(lua: &Lua) -> rlua::Result<Object> {
        let class = class::class_setup(lua, "screen")?;
        Ok(Screen::allocate(lua, class)?.build())
    }
}

impl Display for ScreenState {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Screen: {:p}", self)
    }
}

impl <'lua> ToLua<'lua> for Screen<'lua> {
    fn to_lua(self, lua: &'lua Lua) -> rlua::Result<Value<'lua>> {
        self.0.to_lua(lua)
    }
}

impl UserData for ScreenState {}

pub fn init(lua: &Lua) -> rlua::Result<Class> {
    method_setup(lua, Class::builder(lua, Some(Rc::new(Screen::new)), None, None)?)?
        .save_class("screen")?
        .build()
}

fn method_setup<'lua>(lua: &'lua Lua, builder: ClassBuilder<'lua>) -> rlua::Result<ClassBuilder<'lua>> {
    // TODO Do properly
    use super::dummy;
    builder.method("connect_signal".into(), lua.create_function(dummy))?
           .method("__call".into(), lua.create_function(dummy))
}

impl_objectable!(Screen, ScreenState);
