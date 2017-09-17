use rlua::{self, Lua, Value, ToLua};
use super::class::PropF;

pub struct Property<'lua> {
    name: String,
    cb_new: Option<PropF<'lua>>,
    cb_index: Option<PropF<'lua>>,
    cb_newindex: Option<PropF<'lua>>
}

impl <'lua> ToLua<'lua> for Property<'lua> {
    fn to_lua(self, lua: &'lua Lua) -> rlua::Result<Value<'lua>> {
        let table = lua.create_table();
        table.set("name", self.name)?;
        table.set("cb_new", self.cb_new
                  .map(Value::Function)
                  .unwrap_or(Value::Nil))?;
        table.set("cb_index", self.cb_index
                  .map(Value::Function)
                  .unwrap_or(Value::Nil))?;
        table.set("cb_newindex", self.cb_newindex
                  .map(Value::Function)
                  .unwrap_or(Value::Nil))?;
        Ok(Value::Table(table))
    }
}

impl <'lua> Property <'lua> {
    pub fn new(name: String,
               cb_new: Option<PropF<'lua>>,
               cb_index: Option<PropF<'lua>>,
               cb_newindex: Option<PropF<'lua>>) -> Self {
        Property { name, cb_new, cb_index, cb_newindex }
    }
}
