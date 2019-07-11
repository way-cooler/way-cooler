//! A client to the Wayland compositor. We control their position through tiling
//! and other properties based on what kind of shell they are.

use std::{
    default::Default,
    hash::{Hash, Hasher}
};

use rlua::{self, Table, UserData};

use crate::common::{
    class::{self, Class, ClassBuilder},
    object::Object
};

#[derive(Clone, Debug, Hash)]
pub struct ClientState {
    // TODO Fill in
    pub dummy: i32
}

pub type Client<'lua> = Object<'lua, ClientState>;

impl Default for ClientState {
    fn default() -> Self {
        ClientState { dummy: 0 }
    }
}

impl<'lua> PartialEq for Client<'lua> {
    fn eq(&self, other: &Self) -> bool {
        &*self.state().unwrap() as *const _ ==
            &*other.state().unwrap() as *const _
    }
}

impl<'lua> Eq for Client<'lua> {}

impl<'lua> Hash for Client<'lua> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.state().unwrap().hash(state);
    }
}

// This is currently unused.
// TODO: Figure out if this will be needed later.
impl<'lua> Client<'lua> {
    pub fn new(
        lua: rlua::Context<'lua>,
        args: Table<'lua>
    ) -> rlua::Result<Client<'lua>> {
        let class = class::class_setup(lua, "client")?;
        Ok(Client::allocate(lua, class)?
            .handle_constructor_argument(args)?
            .build())
    }
}

impl UserData for ClientState {}

pub fn init(lua: rlua::Context) -> rlua::Result<Class<ClientState>> {
    method_setup(lua, Class::builder(lua, "client", None)?)?
        .save_class("client")?
        .build()
}

fn method_setup<'lua>(
    lua: rlua::Context<'lua>,
    builder: ClassBuilder<'lua, ClientState>
) -> rlua::Result<ClassBuilder<'lua, ClientState>> {
    // TODO Do properly
    use super::dummy;
    builder
        .method("connect_signal".into(), lua.create_function(dummy)?)?
        .method(
            "__call".into(),
            lua.create_function(|lua, args: Table| Client::new(lua, args))?
        )?
        .method("get".into(), lua.create_function(dummy_table)?)
}

fn dummy_table<'lua>(
    lua: rlua::Context<'lua>,
    _: rlua::Value
) -> rlua::Result<Table<'lua>> {
    Ok(lua.create_table()?)
}
