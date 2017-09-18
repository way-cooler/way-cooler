//! Signals are a list of methods associated with a name that will be called
//! when the signal is triggered.
//!
//! Signals are stored with the object in its metatable,
//! the methods defined here are just to make it easier to use.

use rlua::{self, Lua, ToLuaMulti, Table, Value, Function};
use super::Object;

// TODO FIXME Make this take a list of functions, it won't be that much harder.
// Don't bother checking for duplicates.

/// Connects functions to a signal. Creates a new entry in the table if it
/// doesn't exist.
pub fn connect_signal(lua: &Lua, obj: &Object, name: String, func: Function)
                      -> rlua::Result<()>{
    let signals = obj.signals();
    if let Ok(table) = signals.get::<_, Table>(name.as_str()) {
        let length = table.len()?;
        table.set(length + 1, func)
    } else {
        let table = lua.create_table();
        table.set(1, func)?;
        signals.set(name, table)
    }
}

pub fn disconnect_signal(lua: &Lua, obj: &Object, name: String)
                         -> rlua::Result<()> {
    let signals = obj.signals();
    signals.set(name, Value::Nil)
}

/// Evaluate the functions associated with a signal.
pub fn emit_signal<'lua, A>(lua: &'lua Lua,
                            obj: &'lua Object,
                            name: String,
                            obj_table: Table<'lua>,
                            args: A)
                            -> rlua::Result<()>
    where A: ToLuaMulti<'lua> + Clone
    {
    let signals = obj.signals();
    trace!("Checking signal {}", name);
    // TODO fix this weird hack where I have to store the obj table in
    // lua so that I don't move it in each iteration of the loop...
    lua.globals().set("__temp", obj_table)?;
    if let Ok(table) = signals.get::<_, Table>(name) {
        for entry in table.pairs::<Value, Function>() {
            if let Ok((_, func)) = entry {
                trace!("Found func for signal");
                func.bind(lua.globals().get::<_, Table<'lua>>("__temp"))?
                    .call(args.clone())?
            }
        }
    }
    lua.globals().set("__temp", Value::Nil)?;
    Ok(())
}
