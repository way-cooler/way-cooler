//! A button that was pressed on a keyboard by a user.
//!
//! This is mostly used to define keybindings.

use rlua::{self, Table, ToLua, UserData, UserDataMethods, Value};
use xkbcommon::xkb::{self, Keysym};

use crate::common::{
    class::{self, Class, ClassBuilder},
    object::{self, Object},
    property::Property
};
use crate::lua::mods_to_num;

#[derive(Clone, Debug, Default)]
pub struct KeyState {
    modifiers: u32,
    keysym: Keysym,
    keycode: xkb::Keycode
}

pub type Key<'lua> = Object<'lua, KeyState>;

impl<'lua> Key<'lua> {
    fn new(
        lua: rlua::Context<'lua>,
        args: Table<'lua>
    ) -> rlua::Result<Key<'lua>> {
        // TODO FIXME
        let class = class::class_setup(lua, "key")?;
        Ok(Key::allocate(lua, class)?
            .handle_constructor_argument(args)?
            .build())
    }

    #[allow(dead_code)]
    pub fn keycode(&self) -> rlua::Result<xkb::Keycode> {
        let state = self.state()?;
        Ok(state.keycode)
    }
}

impl UserData for KeyState {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        object::default_add_methods(methods);
    }
}

pub fn init(lua: rlua::Context) -> rlua::Result<Class<KeyState>> {
    property_setup(lua, method_setup(lua, Class::builder(lua, "key", None)?)?)?
        .save_class("key")?
        .build()
}

fn method_setup<'lua>(
    lua: rlua::Context<'lua>,
    builder: ClassBuilder<'lua, KeyState>
) -> rlua::Result<ClassBuilder<'lua, KeyState>> {
    // TODO Do properly
    builder.method(
        "__call".into(),
        lua.create_function(|lua, args: Table| Key::new(lua, args))?
    )
}

fn property_setup<'lua>(
    lua: rlua::Context<'lua>,
    builder: ClassBuilder<'lua, KeyState>
) -> rlua::Result<ClassBuilder<'lua, KeyState>> {
    // TODO Do properly
    builder
        .property(Property::new(
            "key".into(),
            Some(lua.create_function(set_key)?),
            Some(lua.create_function(get_key)?),
            Some(lua.create_function(set_key)?)
        ))?
        .property(Property::new(
            "keysym".into(),
            None,
            Some(lua.create_function(get_keysym)?),
            None
        ))?
        .property(Property::new(
            "modifiers".into(),
            Some(lua.create_function(set_modifiers)?),
            Some(lua.create_function(get_modifiers)?),
            Some(lua.create_function(set_modifiers)?)
        ))
}

fn get_modifiers<'lua>(
    _: rlua::Context<'lua>,
    key: Key<'lua>
) -> rlua::Result<u32> {
    Ok(key.state()?.modifiers)
}

fn set_modifiers<'lua>(
    _: rlua::Context<'lua>,
    (mut key, mods): (Key<'lua>, Table<'lua>)
) -> rlua::Result<()> {
    let mut state = key.state_mut()?;
    state.modifiers = mods_to_num(mods)?.bits();
    Ok(())
}

fn get_keysym<'lua>(
    lua: rlua::Context<'lua>,
    key: Key<'lua>
) -> rlua::Result<Value<'lua>> {
    // TODO Shouldn't this be able to fail?
    let keysym = key.state()?.keysym;
    xkb::keysym_get_name(keysym).to_lua(lua)
}

fn get_key<'lua>(
    lua: rlua::Context<'lua>,
    key: Key<'lua>
) -> rlua::Result<Value<'lua>> {
    key.state()?.keysym.to_lua(lua)
}

fn set_key<'lua>(
    _: rlua::Context<'lua>,
    (mut key, key_name): (Key<'lua>, String)
) -> rlua::Result<Value<'lua>> {
    let mut state = key.state_mut()?;

    if key_name.starts_with('#') && key_name.len() >= 2 {
        let number = key_name[1..].parse::<xkb::Keycode>().map_err(|err| {
            rlua::Error::RuntimeError(format!("Parse error: {:?}", err))
        })?;
        // the - 8 is because of xcb conventions, where "#10" is the keysim for 1,
        // and the keycode of 1 is 0x02 (obviously)
        state.keycode = number - 8;
    } else {
        let keysym = xkb::keysym_from_name(key_name.as_str(), 0);
        state.keysym = keysym;
    }

    Ok(rlua::Value::Nil)
}
