//! TODO Fill in

use std::fmt::{self, Display, Formatter};
use std::default::Default;
use std::rc::Rc;
use rlua::{self, Table, Lua, UserData, ToLua, Value};
use super::object::{Object, Objectable};
use super::class::{self, Class, ClassBuilder};

#[derive(Clone, Debug)]
pub struct AwesomeState {
    // TODO Fill in
    dummy: i32
}

pub struct Awesome<'lua>(Table<'lua>);

impl Default for AwesomeState {
    fn default() -> Self {
        AwesomeState {
            dummy: 0
        }
    }
}

impl <'lua> Awesome<'lua> {
    fn new(lua: &Lua) -> rlua::Result<Object> {
        // TODO FIXME
        let class = class::button_class(lua)?;
        Ok(Awesome::allocate(lua, class)?.build())
    }
}

impl Display for AwesomeState {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Awesome: {:p}", self)
    }
}

impl <'lua> ToLua<'lua> for Awesome<'lua> {
    fn to_lua(self, lua: &'lua Lua) -> rlua::Result<Value<'lua>> {
        self.0.to_lua(lua)
    }
}

impl UserData for AwesomeState {}

pub fn init(lua: &Lua) -> rlua::Result<Class> {
    property_setup(lua, method_setup(lua, Class::builder(lua, Some(Rc::new(Awesome::new)), None, None)?)?)?
        .save_class("awesome")?
        .build()
}

fn method_setup<'lua>(lua: &'lua Lua, builder: ClassBuilder<'lua>) -> rlua::Result<ClassBuilder<'lua>> {
    // TODO Do properly
    builder.method("connect_signal".into(), lua.create_function(dummy))?
           .method("register_xproperty".into(), lua.create_function(dummy))?
           .method("restart".into(), lua.create_function(dummy))?
           .method("quit".into(), lua.create_function(dummy))
}

fn property_setup<'lua>(lua: &'lua Lua, builder: ClassBuilder<'lua>) -> rlua::Result<ClassBuilder<'lua>> {
    // TODO Do properly
    builder.dummy_property("version".into(), "0".to_lua(lua)?)?
           .dummy_property("themes_path".into(), "/usr/share/awesome/themes".to_lua(lua)?)?
           .dummy_property("conffile".into(), "".to_lua(lua)?)
}

impl_objectable!(Awesome, AwesomeState);

fn dummy<'lua>(_: &'lua Lua, _: rlua::Value) -> rlua::Result<()> { Ok(()) }
