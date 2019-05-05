//! Signals are a list of methods associated with a name that will be called
//! when the signal is triggered.
//!
//! Signals are stored with the object in its metatable,
//! the methods defined here are just to make it easier to use.

use rlua::{self, Function, Table, ToLuaMulti, Value};

use crate::GLOBAL_SIGNALS;

/// Connects functions to a signal. Creates a new entry in the table if it
/// doesn't exist.
pub fn connect_signals<'lua>(
    lua: rlua::Context<'lua>,
    signals: Table<'lua>,
    name: &str,
    funcs: &[Function<'lua>]
) -> rlua::Result<()> {
    if let Ok(Value::Table(table)) = signals.get::<_, Value>(name) {
        let mut length = table.len()? + 1;
        for func in funcs {
            table.set(length, func.clone())?;
            length += 1;
        }
        Ok(())
    } else {
        let table = lua.create_table()?;
        for (index, func) in funcs.into_iter().enumerate() {
            table.set(index + 1, func.clone())?;
        }
        signals.set(name, table)
    }
}

pub fn disconnect_signals(_: rlua::Context, signals: Table, name: &str) -> rlua::Result<()> {
    signals.set(name, Value::Nil)
}

/// Evaluate the functions associated with a signal.
pub fn emit_signals<'lua, A>(
    _: rlua::Context<'lua>,
    signals: Table<'lua>,
    name: &str,
    args: A
) -> rlua::Result<()>
where
    A: ToLuaMulti<'lua> + Clone
{
    if let Ok(Value::Table(table)) = signals.get::<_, Value>(name.clone()) {
        for entry in table.pairs::<Value, Function>() {
            if let Ok((_, func)) = entry {
                match func.call(args.clone()) {
                    Ok(()) => {},
                    Err(e) => {
                        error!("Error while emitting signal {}: {}", name, e);
                    }
                };
            }
        }
    }
    Ok(())
}

/// Connect the function to the named signal in the global signal list.
pub fn global_connect_signal<'lua>(
    lua: rlua::Context<'lua>,
    (name, func): (String, Function<'lua>)
) -> rlua::Result<()> {
    let global_signals = lua.named_registry_value::<str, Table>(GLOBAL_SIGNALS)?;
    connect_signals(lua, global_signals, &name, &[func])
}

/// Disconnect the function from the named signal in the global signal list.
pub fn global_disconnect_signal<'lua>(lua: rlua::Context<'lua>, name: String) -> rlua::Result<()> {
    let global_signals = lua.named_registry_value::<str, Table>(GLOBAL_SIGNALS)?;
    disconnect_signals(lua, global_signals, &name)
}

/// Emit the signal with the given name from the global signal list.
pub fn global_emit_signal<'lua>(
    lua: rlua::Context<'lua>,
    (name, args): (String, Value<'lua>)
) -> rlua::Result<()> {
    let global_signals = lua.named_registry_value::<str, Table>(GLOBAL_SIGNALS)?;
    emit_signals(lua, global_signals, &name, args)
}
