//! A button that was pressed on a mouse by a user.
//!
//! This is mostly used to define bindings.

use std::default::Default;

use {
    rlua::{self, Table, ToLua as _, UserData, UserDataMethods, Value},
    xcb::ffi::xproto::xcb_button_t,
    xkbcommon::xkb::Keysym
};

use crate::common::{
    class::{self, Class},
    object::{self, Object},
    property::Property
};

#[derive(Clone, Debug)]
pub struct ButtonState {
    button: xcb_button_t,
    modifiers: Vec<Keysym>
}

pub type Button<'lua> = Object<'lua, ButtonState>;

impl Default for ButtonState {
    fn default() -> Self {
        ButtonState {
            button: xcb_button_t::default(),
            modifiers: Vec::new()
        }
    }
}

impl UserData for ButtonState {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        object::default_add_methods(methods);
    }
}

impl<'lua> Button<'lua> {
    fn new(
        lua: rlua::Context<'lua>,
        args: Table<'lua>
    ) -> rlua::Result<Button<'lua>> {
        let class = class::class_setup(lua, "button")?;
        Ok(Button::allocate(lua, class)?
            .handle_constructor_argument(args)?
            .build())
    }
}

pub fn init(lua: rlua::Context) -> rlua::Result<Class<ButtonState>> {
    Class::<ButtonState>::builder(lua, "button", None)?
        .method(
            "__call".into(),
            lua.create_function(|lua, args: Table| Button::new(lua, args))?
        )?
        .property(Property::new(
            "button".into(),
            Some(lua.create_function(set_button)?),
            Some(lua.create_function(get_button)?),
            Some(lua.create_function(set_button)?)
        ))?
        .property(Property::new(
            "modifiers".into(),
            Some(lua.create_function(set_modifiers)?),
            Some(lua.create_function(get_modifiers)?),
            Some(lua.create_function(set_modifiers)?)
        ))?
        .save_class("button")?
        .build()
}

fn set_button<'lua>(
    lua: rlua::Context<'lua>,
    (mut button, val): (Button<'lua>, Value<'lua>)
) -> rlua::Result<Value<'lua>> {
    let val = match val {
        Value::Number(num) => num as _,
        Value::Integer(num) => num as _,
        _ => xcb_button_t::default()
    };

    let mut state = button.state_mut()?;
    state.button = val;
    drop(state);

    Object::emit_signal(lua, &button, "property::button", val)?;

    Ok(Value::Nil)
}

fn get_button<'lua>(
    _: rlua::Context<'lua>,
    button: Button<'lua>
) -> rlua::Result<Value<'lua>> {
    Ok(Value::Integer(button.state()?.button as i64))
}

fn set_modifiers<'lua>(
    lua: rlua::Context<'lua>,
    (mut button, modifiers): (Button<'lua>, Table<'lua>)
) -> rlua::Result<()> {
    let modifier = crate::lua::mods_to_rust(modifiers.clone())?;

    let mut state = button.state_mut()?;
    state.modifiers = modifier;
    drop(state);

    Object::emit_signal(lua, &button, "property::modifiers", modifiers)?;

    Ok(())
}

fn get_modifiers<'lua>(
    lua: rlua::Context<'lua>,
    button: Button<'lua>
) -> rlua::Result<Value<'lua>> {
    crate::lua::mods_to_lua(lua, &button.state()?.modifiers)?.to_lua(lua)
}
