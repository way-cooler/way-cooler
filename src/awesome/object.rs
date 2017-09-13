//! Utility methods and constructors for Lua objects

use std::fmt::Display;
use std::convert::From;
use rlua::{self, Lua, Table, MetaMethod, UserData, AnyUserData, Value,
           UserDataMethods, FromLua, ToLua};
use super::signal::{connect_signal, emit_signal};

/// All Lua objects can be cast to this.
pub struct Object<'lua> {
    table: Table<'lua>
}

/// Trait implemented by all objects that represent OO lua objects.
///
/// This trait allows casting an object gotten back from the Lua runtime
/// into a concrete object so that Rust can do things with it.
///
/// You can't do anything to the object until it has been converted into a
/// canonical form using this trait.
pub trait Objectable<'lua, T, S: UserData> {
    fn cast(obj: Object<'lua>) -> rlua::Result<T> {
        let data = obj.table.get::<_, AnyUserData>("data")?;
        if data.is::<S>() {
            Ok(Self::_wrap(obj.table))
        } else {
            use rlua::Error::RuntimeError;
            Err(RuntimeError("Could not cast object to concrete type".into()))
        }
    }

    /// Given the internal object table, constructs a concrete object out
    /// of it.
    ///
    /// This is only used internally, which is why it's prefixed with a "_"
    /// Please do not use it outside of object.rs.
    fn _wrap(table: Table<'lua>) -> T;

    /// Gets the internal table for the concrete object.
    /// Used internally by cast, though there's nothing wrong with it being
    /// used outside of internal object use.
    fn get_table(self) -> Table<'lua>;
}

impl <'lua> ToLua<'lua> for Object<'lua> {
    fn to_lua(self, lua: &'lua Lua) -> rlua::Result<Value<'lua>> {
        Ok(Value::Table(self.table))
    }
}

impl <'lua> Object<'lua> {
    /// Coerces a concrete object into a generic object.
    ///
    /// This requires a check to ensure data integrity, and it's often useless.
    /// Please don't use this method unless you need to.
    pub fn to_object<T, S: UserData, O: Objectable<'lua, T, S>>(obj: O) -> Self {
        let table = obj.get_table();
        let has_data = table.contains_key("data")
            .expect("Could not get data field of object table");
        assert_eq!(has_data, true);
        Object { table }
    }

    pub fn signals(&self) -> rlua::Table {
        self.table.get::<_, Table>("signals")
            .expect("Object table did not have signals defined!")
    }

    // TODO make this return a builder so it's easier to modify the meta table
    // without having to resort to to_object.
    //
    // That would mean we can reduce usage of to_object, which is costy / not panic safe.
    pub fn new<T>(lua: &'lua Lua) -> rlua::Result<Self>
        where T: UserData + Default + Display + Clone
    {
        let object = T::default();
        let object_table = lua.create_table();
        object_table.set("data", object)?;
        let meta = lua.create_table();
        meta.set("signals", lua.create_table())?;
        meta.set("__index", lua.create_function(default_index))?;
        meta.set("__tostring", lua.create_function(|_, object_table: Table| {
            Ok(format!("{}", object_table.get::<_, T>("data")?))
        }))?;
        object_table.set_metatable(Some(meta));
        Ok(Object { table: object_table })
    }
}

/// Default indexing of an Awesome object.
///
/// Automatically looks up contents in meta table, so instead of overriding this
/// it's easier to just add the required data in the meta table.
pub fn default_index<'lua>(lua: &'lua Lua, (obj_table, index): (Table<'lua>, Value<'lua>))
                       -> rlua::Result<Value<'lua>> {
    // Look up in metatable first
    if let Some(meta) = obj_table.get_metatable() {
        if let Ok(val) = meta.raw_get::<_, Value>(index.clone()) {
            match val {
                Value::Nil => {},
                val => return Ok(val)
            }
        }
    }
    // TODO Handle non string indexing?
    // double check C code
    let index = String::from_lua(index, lua)?;
    // TODO FIXME handle special "valid" property
    match index.as_str() {
        "connect_signal" => {
            let func = lua.create_function(
                |lua, (obj_table, signal, func): (Table, String, rlua::Function)| {
                    connect_signal(lua, Object { table: obj_table }, signal, func)
            });
            func.bind(obj_table).map(rlua::Value::Function)
        },
        "emit_signal" => {
            let func = lua.create_function(
                |lua, (obj_table, signal, args): (Table, String, rlua::Value)| {
                    // TODO FIXME this seems wrong to always pass the object table in,
                    // but maybe that's always how object signal emitting should work?
                    // Look this up, double check!
                    emit_signal(lua, &Object { table: obj_table.clone() }, signal, obj_table)
                });
            func.bind(obj_table).map(rlua::Value::Function)
        }
        index => {
            let err_msg = format!("Could not find index \"{:#?}\"", index);
            warn!("{}", err_msg);
            Err(rlua::Error::RuntimeError(err_msg))
        }
    }
}
