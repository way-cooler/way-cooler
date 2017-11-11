//! Signals are a list of methods associated with a name that will be called
//! when the signal is triggered.
//!
//! Signals are stored with the object in its metatable,
//! the methods defined here are just to make it easier to use.

use rlua::{self, Lua, Table, ToLuaMulti, Value, Function};
use super::{GLOBAL_SIGNALS, Object};

/// Connects functions to a signal. Creates a new entry in the table if it
/// doesn't exist.
pub fn connect_signal(lua: &Lua, obj: Object, name: String, funcs: &[Function])
                      -> rlua::Result<()>{
    let signals = obj.signals()?;
    if let Ok(Value::Table(table)) = signals.get::<_, Value>(name.as_str()) {
        let mut length = table.len()? + 1;
        for func in funcs {
            table.set(length, func.clone())?;
            length += 1;
        }
        Ok(())
    } else {
        let table = lua.create_table();
        for (index, func) in funcs.into_iter().enumerate() {
            table.set(index + 1, func.clone())?;
        }
        signals.set(name, table)
    }
}

pub fn disconnect_signal(_: &Lua, obj: Object, name: String)
                         -> rlua::Result<()> {
    let signals = obj.signals()?;
    signals.set(name, Value::Nil)
}

/// Evaluate the functions associated with a signal.
pub fn emit_signal<'lua, A>(_: &'lua Lua,
                            obj: Object<'lua>,
                            name: String,
                            args: A)
                            -> rlua::Result<()>
    where A: ToLuaMulti<'lua> + Clone
    {
    let signals = obj.signals()?;
    trace!("Checking signal {}", name);
    let obj_table = obj.table();
    if let Ok(Value::Table(table)) = signals.get::<_, Value>(name) {
        for entry in table.pairs::<Value, Function>() {
            if let Ok((_, func)) = entry {
                trace!("Found func for signal");
                func.bind(obj_table.clone())?
                    .call(args.clone())?
            }
        }
    }
    Ok(())
}


/// Connect the function to the named signal in the global signal list.
pub fn global_connect_signal<'lua>(lua: &'lua Lua, (name, func): (String, rlua::Function<'lua>))
                        -> rlua::Result<()> {
    let global_signals = lua.globals().get::<_, Table>(GLOBAL_SIGNALS)?;
    let fake_object = lua.create_table();
    fake_object.set("signals", global_signals)?;
    connect_signal(lua, fake_object.into(), name, &[func])
}

/// Disconnect the function from the named signal in the global signal list.
pub fn global_disconnect_signal<'lua>(lua: &'lua Lua, name: String) -> rlua::Result<()> {
    let global_signals = lua.globals().get::<_, Table>(GLOBAL_SIGNALS)?;
    let fake_object = lua.create_table();
    fake_object.set("signals", global_signals)?;
    disconnect_signal(lua, fake_object.into(), name)
}

/// Emit the signal with the given name from the global signal list.
pub fn global_emit_signal<'lua>(lua: &'lua Lua, (name, args): (String, Value))
                     -> rlua::Result<()> {
    let global_signals = lua.globals().get::<_, Table>(GLOBAL_SIGNALS)?;
    let fake_object = lua.create_table();
    fake_object.set("signals", global_signals)?;
    emit_signal(lua, fake_object.into(), name, args)
}
