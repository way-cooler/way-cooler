//! Signals are methods defined on the __index of a table in the Awesome
//! Lua API.

use rlua::{self, ToLuaMulti};

#[derive(Debug, Clone)]
pub struct Signal {
    pub name: String,
    pub funcs: Vec<rlua::Function<'static>>
}

unsafe impl Send for Signal {}

impl Signal {
    pub fn new(name: String, funcs: Vec<rlua::Function<'static>>) -> Self {
        Signal { name, funcs }
    }

    pub fn evaluate<'lua, A>(&self, args: A) -> rlua::Result<()>
        where A: ToLuaMulti<'lua> + Clone
    {
        for func in &self.funcs {
            func.call(args.clone())?
        }
        Ok(())
    }
}
