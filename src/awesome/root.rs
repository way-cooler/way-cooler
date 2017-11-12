//! TODO Fill in

use std::fmt::{self, Display, Formatter};
use std::default::Default;
use std::rc::Rc;
use rlua::{self, Table, Lua, UserData, ToLua, Value};
use super::object::{Object, Objectable};
use super::class::{self, Class, ClassBuilder};

#[derive(Clone, Debug)]
pub struct RootState {
    // TODO Fill in
    dummy: i32
}

pub struct Root<'lua>(Table<'lua>);

impl Default for RootState {
    fn default() -> Self {
        RootState {
            dummy: 0
        }
    }
}

impl <'lua> Root<'lua> {
    fn new(lua: &Lua) -> rlua::Result<Object> {
        // TODO FIXME
        let class = class::class_setup(lua, "root")?;
        Ok(Root::allocate(lua, class)?.build())
    }
}

impl Display for RootState {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Root: {:p}", self)
    }
}

impl <'lua> ToLua<'lua> for Root<'lua> {
    fn to_lua(self, lua: &'lua Lua) -> rlua::Result<Value<'lua>> {
        self.0.to_lua(lua)
    }
}

impl UserData for RootState {}

pub fn init(lua: &Lua) -> rlua::Result<Class> {
    method_setup(lua, Class::builder(lua, Some(Rc::new(Root::new)), None, None)?)?
        .save_class("root")?
        .build()
}

fn method_setup<'lua>(lua: &'lua Lua, builder: ClassBuilder<'lua>) -> rlua::Result<ClassBuilder<'lua>> {
    // TODO Do properly
    use super::dummy;
    builder.method("connect_signal".into(), lua.create_function(dummy))?
           .method("buttons".into(), lua.create_function(dummy))?
           .method("keys".into(), lua.create_function(dummy))?
           .method("size".into(), lua.create_function(dummy_double))?
           .method("size_mm".into(), lua.create_function(dummy_double))?
           .method("cursor".into(), lua.create_function(dummy))
}

impl_objectable!(Root, RootState);

fn dummy_double<'lua>(_: &'lua Lua, _: rlua::Value) -> rlua::Result<(i32, i32)> { Ok((0, 0)) }
