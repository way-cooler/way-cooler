//! TODO Fill in

use rlua::{self, AnyUserData, Lua, Table, ToLua, UserData, UserDataMethods, Value};
use std::fmt::{self, Display, Formatter};

use wlroots::{self, xkbcommon::xkb};

use super::class::{self, Class, ClassBuilder};
use super::lua::mods_to_num;
use super::object::{self, Object, Objectable};
use super::property::Property;

#[derive(Clone, Debug, Default)]
pub struct KeyState {
    modifiers: u32,
    keysym: wlroots::Key,
    keycode: xkb::Keycode
}

pub struct Key<'lua>(Object<'lua>);

impl<'lua> Key<'lua> {
    fn new(lua: &'lua Lua, args: Table) -> rlua::Result<Object<'lua>> {
        // TODO FIXME
        let class = class::class_setup(lua, "key")?;
        Ok(Key::allocate(lua, class)?.handle_constructor_argument(args)?
                                     .build())
    }

    pub fn set_modifiers(&mut self, modifiers: u32) -> rlua::Result<()> {
        let mut state = self.get_object_mut()?;
        state.modifiers = modifiers;
        Ok(())
    }

    pub fn modifiers(&self) -> rlua::Result<u32> {
        let state = self.state()?;
        Ok(state.modifiers)
    }

    pub fn set_keysym(&mut self, keysym: wlroots::Key) -> rlua::Result<()> {
        let mut state = self.get_object_mut()?;
        state.keysym = keysym;
        Ok(())
    }

    pub fn keysym(&self) -> rlua::Result<wlroots::Key> {
        let state = self.state()?;
        Ok(state.keysym)
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
    builder.property(Property::new("key".into(),
                                   Some(lua.create_function(set_key)?),
                                   Some(lua.create_function(get_key)?),
                                   Some(lua.create_function(set_key)?)))?
           .property(Property::new("keysym".into(),
                                   None,
                                   Some(lua.create_function(get_keysym)?),
                                   None))?
           .property(Property::new("modifiers".into(),
                                   Some(lua.create_function(set_modifiers)?),
                                   Some(lua.create_function(get_modifiers)?),
                                   Some(lua.create_function(set_modifiers)?)))
}

fn get_modifiers<'lua>(_: &'lua Lua, obj: AnyUserData<'lua>) -> rlua::Result<u32> {
    Key::cast(obj.into())?.modifiers()
}

fn set_modifiers<'lua>(_: &'lua Lua,
                       (obj, mods): (AnyUserData<'lua>, Table<'lua>))
                       -> rlua::Result<()> {
    let mut key = Key::cast(obj.into())?;
    key.set_modifiers(mods_to_num(mods)?.bits())
}

fn get_keysym<'lua>(lua: &'lua Lua, obj: AnyUserData<'lua>) -> rlua::Result<Value<'lua>> {
    let key = Key::cast(obj.into())?;
    // TODO Shouldn't this be able to fail?
    xkb::keysym_get_name(key.keysym()?).to_lua(lua)
}

fn get_key<'lua>(lua: &'lua Lua, obj: AnyUserData<'lua>) -> rlua::Result<Value<'lua>> {
    Key::cast(obj.into())?.keysym()?.to_lua(lua)
}

fn set_key<'lua>(_: &'lua Lua,
                 (obj, key): (AnyUserData<'lua>, String))
                 -> rlua::Result<Value<'lua>> {
    // TODO Deal with #number's correctly. This isn't 1-1 to how awesome does
    // parsing
    let keysym = xkb::keysym_from_name(key.as_str(), 0);
    let mut key = Key::cast(obj.clone().into())?;
    key.set_keysym(keysym)?;
    Ok(rlua::Value::Nil)
}

impl_objectable!(Key, KeyState);
