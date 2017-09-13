//! Signals are a list of methods associated with a name that will be called
//! when the signal is triggered.
//!
//! Signals are stored with the object in its metatable,
//! the methods defined here are just to make it easier to use.

use rlua::{self, ToLuaMulti, Table};

/// Connects functions to a signal. Creates a new entry in the table if it
/// doesn't exist.
pub fn connect_signal(table: Table, name: String, func: rlua::Function)
                      -> rlua::Result<()>{
    let signals = table.get::<_, Table>("signals")?;
    if let Ok(table) = signals.get::<_, Table>(name.as_str()) {
        let length = table.len()?;
        table.set(length + 1, func)
    } else {
        signals.set(name, func)
    }
}

/// Evaluate the functions associated with a signal.
pub fn emit_signal<'lua, A>(table: Table<'lua>, name: String, args: A)
                            -> rlua::Result<()>
    where A: ToLuaMulti<'lua> + Clone
    {
    let signals = table.get::<_, Table>("signals")?;
    if let Ok(table) = signals.get::<_, Table>(name) {
        for entry in table.pairs::<rlua::Value, rlua::Function>() {
            if let Ok((_, func)) = entry {
                func.call(args.clone())?
            }
        }
    }
    Ok(())
}
