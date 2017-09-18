//! Utility methods and constructors for Lua objects

use std::fmt::Display;
use std::convert::From;
use rlua::{self, Lua, Table, MetaMethod, UserData, AnyUserData, Value,
           UserDataMethods, FromLua, ToLua};
use super::signal::{disconnect_signal, connect_signal, emit_signal};
use super::class::Class;
use super::property::Property;

/// All Lua objects can be cast to this.
pub struct Object<'lua> {
    table: Table<'lua>
}

impl <'lua> From<Table<'lua>> for Object<'lua> {
    fn from(table: Table<'lua>) -> Self {
        Object { table }
    }
}

/// Construct a new object, used when using the default Objectable::new.
pub struct ObjectBuilder<'lua>{
    lua: &'lua Lua,
    object: Object<'lua>
}

impl <'lua> ObjectBuilder<'lua> {
    pub fn add_to_meta(self, new_meta: Table<'lua>) -> rlua::Result<Self> {
        let meta = self.object.table.get_metatable()
            .expect("Object had no meta table");
        for entry in new_meta.pairs::<rlua::Value, rlua::Value>() {
            let (key, value) = entry?;
            meta.set(key, value)?;
        }
        Ok(self)
    }

    pub fn add_to_signals(self, name: String, func: rlua::Function)
                          -> rlua::Result<Self> {
        connect_signal(self.lua, &self.object, name, func)?;
        Ok(self)
    }

    pub fn build(self) -> Object<'lua> {
        self.object
    }
}

/// Trait implemented by all objects that represent OO lua objects.
///
/// This trait allows casting an object gotten back from the Lua runtime
/// into a concrete object so that Rust can do things with it.
///
/// You can't do anything to the object until it has been converted into a
/// canonical form using this trait.
pub trait Objectable<'lua, T, S: UserData + Default + Display + Clone> {
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

    fn new(lua: &'lua Lua, class: Class) -> rlua::Result<ObjectBuilder<'lua>>
    {
        let object = S::default();
        let object_table = lua.create_table();
        object_table.set("data", object)?;
        let meta = lua.create_table();
        meta.set("__class", class)?;
        meta.set("signals", lua.create_table())?;
        meta.set("__index", lua.create_function(default_index))?;
        meta.set("__tostring", lua.create_function(|_, object_table: Table| {
            Ok(format!("{}", object_table.get::<_, S>("data")?))
        }))?;
        object_table.set_metatable(Some(meta));
        let object = Object { table: object_table };
        Ok(ObjectBuilder { object, lua })
    }
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
    pub fn to_object<T, S, O>(obj: O) -> Self
        where S: Default + Display + Clone + UserData,
              O: Objectable<'lua, T, S>
    {
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
}

/// Default indexing of an Awesome object.
///
/// Automatically looks up contents in meta table, so instead of overriding this
/// it's easier to just add the required data in the meta table.
pub fn default_index<'lua>(lua: &'lua Lua,
                           (obj_table, index): (Table<'lua>, Value<'lua>))
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
    let index = String::from_lua(index, lua)?;
    match index.as_str() {
        // TODO FIXME This should also be usable for invalid objects!
        // might need to abstract logic...?
        "valid" => {
            // this requires class being implemented
            unimplemented!();
            /*
            // TODO object table? Why not Objectable?
            Value::Boolean(class.checker.and_then(|checker| checker(obj_table))
                           // TODO Only true if this is really an object!
                           // Make it objectable to make it so!
                           .unwrap_or(true))
            */
        },
        "data" => {
            obj_table.get("data")
        },
        // TODO Move these into the meta table itself
        // (so only check valid property and data)
        "connect_signal" => {
            let func = lua.create_function(
                |lua, (obj_table, signal, func): (Table, String, rlua::Function)| {
                    connect_signal(lua, &Object { table: obj_table }, signal, func)
                });
            func.bind(obj_table).map(rlua::Value::Function)
        },
        "disconnect_signal" => {
            let func = lua.create_function(
                |lua, (obj_table, signal): (Table, String)| {
                    disconnect_signal(lua, &Object { table: obj_table }, signal)
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
            // Try see if there is a property of the class with the name
            if let Some(meta) = obj_table.get_metatable() {
                if let Ok(class) = meta.get::<_, Table>("__class") {
                    let props = class.get::<_, Vec<Property>>("properties")?;
                    for prop in props {
                        if prop.name.as_str() == index {
                            // Property exists and has an index callback
                            if let Some(index) = prop.cb_index {
                                return index.call(obj_table)
                            }
                        }
                    }
                }
            }
            Ok(rlua::Value::Nil)
        }
    }
}
