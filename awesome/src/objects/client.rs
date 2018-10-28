//! A client to the Wayland compositor. We control their position through tiling
//! and other properties based on what kind of shell they are.

use std::default::Default;
use std::fmt::{self, Display, Formatter};

use rlua::{self, Lua, Table, ToLua, UserData, Value};

use common::{class::{Class, ClassBuilder},
             object::Object};

#[derive(Clone, Debug)]
pub struct ClientState {
    // TODO Fill in
    dummy: i32
}

pub type Client<'lua> = Object<'lua, ClientState>;

impl Default for ClientState {
    fn default() -> Self {
        ClientState { dummy: 0 }
    }
}

/* This is currently unused.
 * TODO: Figure out if this will be needed later.

impl <'lua> Client<'lua> {
    fn new(lua: &Lua) -> rlua::Result<Client> {
        let class = class::class_setup(lua, "client")?;
        Ok(Client::allocate(lua, class)?.build())
    }
}
*/

impl Display for ClientState {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Client: {:p}", self)
    }
}

impl UserData for ClientState {}

pub fn init(lua: &Lua) -> rlua::Result<Class<ClientState>> {
    method_setup(lua, Class::builder(lua, "client", None)?)?.save_class("client")?
                                                            .build()
}

fn method_setup<'lua>(lua: &'lua Lua,
                      builder: ClassBuilder<'lua, ClientState>)
                      -> rlua::Result<ClassBuilder<'lua, ClientState>> {
    // TODO Do properly
    use super::dummy;
    builder.method("connect_signal".into(), lua.create_function(dummy)?)?
           .method("get".into(), lua.create_function(dummy_table)?)
}

fn dummy_table<'lua>(lua: &'lua Lua, _: rlua::Value) -> rlua::Result<Table<'lua>> {
    Ok(lua.create_table()?)
}
