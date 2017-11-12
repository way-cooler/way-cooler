//! TODO Fill in

use std::fmt::{self, Display, Formatter};
use std::default::Default;
use std::rc::Rc;
use rlua::{self, Table, Lua, UserData, ToLua, Value};
use super::object::{Object, Objectable};
use super::class::{self, Class, ClassBuilder};

#[derive(Clone, Debug)]
pub struct DrawableState {
    // TODO Fill in
    dummy: i32
}

pub struct Drawable<'lua>(Table<'lua>);

impl Default for DrawableState {
    fn default() -> Self {
        DrawableState {
            dummy: 0
        }
    }
}

impl <'lua> Drawable<'lua> {
    pub fn new(lua: &Lua) -> rlua::Result<Object> {
        let class = class::class_setup(lua, "drawable")?;
        Ok(Drawable::allocate(lua, class)?.build())
    }
}

impl Display for DrawableState {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Drawable: {:p}", self)
    }
}

impl <'lua> ToLua<'lua> for Drawable<'lua> {
    fn to_lua(self, lua: &'lua Lua) -> rlua::Result<Value<'lua>> {
        self.0.to_lua(lua)
    }
}

impl UserData for DrawableState {}

pub fn init(lua: &Lua) -> rlua::Result<Class> {
    method_setup(lua, Class::builder(lua, Some(Rc::new(Drawable::new)), None, None)?)?
        .save_class("drawable")?
        .build()
}

fn method_setup<'lua>(lua: &'lua Lua, builder: ClassBuilder<'lua>) -> rlua::Result<ClassBuilder<'lua>> {
    // TODO Do properly
    use super::dummy;
    builder.method("connect_signal".into(), lua.create_function(dummy))?
           .method("__call".into(), lua.create_function(dummy_create))
}

impl_objectable!(Drawable, DrawableState);

fn dummy_create<'lua>(lua: &'lua Lua, _: rlua::Value) -> rlua::Result<Table<'lua>> {
    Ok(lua.create_table())
}
