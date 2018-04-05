//! TODO Fill in

use super::class::{self, Class, ClassBuilder};
use super::object::{self, Object, Objectable};
use rlua::{self, Lua, Table, ToLua, UserData, UserDataMethods, Value};
use std::default::Default;
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug)]
pub struct KeyState {
    // TODO Fill in
    dummy: i32
}

pub struct Key<'lua>(Object<'lua>);

impl Default for KeyState {
    fn default() -> Self {
        KeyState { dummy: 0 }
    }
}

impl<'lua> Key<'lua> {
    fn new(lua: &'lua Lua, args: Table) -> rlua::Result<Object<'lua>> {
        // TODO FIXME
        let class = class::class_setup(lua, "key")?;
        Ok(Key::allocate(lua, class)?.handle_constructor_argument(args)?
                                     .build())
    }
}

impl Display for KeyState {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Key: {:p}", self)
    }
}

impl<'lua> ToLua<'lua> for Key<'lua> {
    fn to_lua(self, lua: &'lua Lua) -> rlua::Result<Value<'lua>> {
        self.0.to_lua(lua)
    }
}

impl UserData for KeyState {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        object::default_add_methods(methods);
    }
}

pub fn init(lua: &Lua) -> rlua::Result<Class> {
    property_setup(lua, method_setup(lua, Class::builder(lua, "key", None)?)?)?.save_class("key")?
                                                                               .build()
}

fn method_setup<'lua>(lua: &'lua Lua,
                      builder: ClassBuilder<'lua>)
                      -> rlua::Result<ClassBuilder<'lua>> {
    // TODO Do properly
    builder.method("__call".into(),
                   lua.create_function(|lua, args: Table| Key::new(lua, args))?)
}

fn property_setup<'lua>(lua: &'lua Lua,
                        builder: ClassBuilder<'lua>)
                        -> rlua::Result<ClassBuilder<'lua>> {
    // TODO Do properly
    builder.dummy_property("version".into(), "0".to_lua(lua)?)?
           .dummy_property("themes_path".into(),
                           "/usr/share/awesome/themes".to_lua(lua)?)?
           .dummy_property("conffile".into(), "".to_lua(lua)?)
}

impl_objectable!(Key, KeyState);
