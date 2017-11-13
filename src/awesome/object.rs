//! Utility methods and constructors for Lua objects

use std::fmt::Display;
use std::convert::From;
use rlua::{self, Lua, Table, UserData, AnyUserData, Value,
           FromLua, ToLua, Function};
use super::signal;
use super::class::Class;
use super::property::Property;

/// All Lua objects can be cast to this.
#[derive(Clone, Debug)]
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
    #[allow(dead_code)]
    pub fn add_to_meta(self, new_meta: Table<'lua>) -> rlua::Result<Self> {
        let meta = self.object.table.get_metatable()
            .expect("Object had no meta table");
        for entry in new_meta.pairs::<rlua::Value, rlua::Value>() {
            let (key, value) = entry?;
            meta.set(key, value)?;
        }
        self.object.table.set_metatable(Some(meta));
        Ok(self)
    }

    #[allow(dead_code)]
    pub fn add_to_signals(self, name: String, func: rlua::Function)
                          -> rlua::Result<Self> {
        signal::connect_signal(self.lua, self.object.clone(), name, &[func])?;
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
        let data = obj.table.get::<_, Table>("data")?.get::<_, AnyUserData>("data")?;
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

    fn state(&self) -> rlua::Result<S> {
        self.get_table().get::<_, Table>("data")?.get::<_, S>("data")
    }

    fn set_state(&self, data: S) -> rlua::Result<()> {
        self.get_table().get::<_, Table>("data")?.set("data", data)
    }

    /// Gets the internal table for the concrete object.
    /// Used internally by cast, though there's nothing wrong with it being
    /// used outside of internal object use.
    fn get_table(&self) -> Table<'lua>;

    fn allocate(lua: &'lua Lua, class: Class) -> rlua::Result<ObjectBuilder<'lua>>
    {
        let object = S::default();
        // TODO FIXME
        // we put a wrapper table here,
        // but in awesome they use lua_setuservalue.
        // I don't think this is equivilent...it just forces things to "work"
        let wrapper_table = lua.create_table();
        wrapper_table.set("data", object)?;
        let object_table = lua.create_table();
        object_table.set("data", wrapper_table)?;
        let meta = lua.create_table();
        meta.set("__class", class)?;
        meta.set("signals", lua.create_table())?;
        meta.set("connect_signal",
                 lua.create_function(connect_signal))?;
        meta.set("disconnect_signal",
                 lua.create_function(disconnect_signal))?;
        meta.set("emit_signal", lua.create_function(emit_signal))?;
        meta.set("__index", lua.create_function(default_index))?;
        meta.set("__newindex", lua.create_function(default_newindex))?;
        meta.set("__tostring", lua.create_function(|_, object_table: Table| {
            Ok(format!("{}", object_table.get::<_, Table>("data")?.get::<_, S>("data")?))
        }))?;
        object_table.set_metatable(Some(meta));
        let object = Object { table: object_table };
        Ok(ObjectBuilder { object, lua })
    }
}

impl <'lua> ToLua<'lua> for Object<'lua> {
    fn to_lua(self, _: &'lua Lua) -> rlua::Result<Value<'lua>> {
        Ok(Value::Table(self.table))
    }
}

impl <'lua> Object<'lua> {
    /// Coerces a concrete object into a generic object.
    ///
    /// This requires a check to ensure data integrity, and it's often useless.
    /// Please don't use this method unless you need to.
    #[allow(dead_code)]
    #[deprecated]
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

    pub fn signals(&self) -> rlua::Result<rlua::Table<'lua>> {
        self.table.get::<_, Table>("signals")
    }

    pub fn table(self) -> rlua::Table<'lua> {
        self.table
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
    let meta = obj_table.get_metatable()
        .expect("Object had no metatable");
    if meta.get::<_, Table>("__class").is_ok() {
        if let Ok(val) = meta.raw_get::<_, Value>(index.clone()) {
            match val {
                Value::Nil => {},
                val => return Ok(val)
            }
        }
    }
    let index = String::from_lua(index, lua)?;
    match index.as_str() {
        "valid" => {
            Ok(Value::Boolean(
                if let Ok(class) = meta.get::<_, Table>("__class") {
                    let class: Class = class.into();
                    class.checker()?
                        .map(|checker| checker(obj_table.into()))
                        .unwrap_or(true)
                } else {
                    false
                }))
        },
        "data" => {
            obj_table.get("data")
        },
        index => {
            // Try see if there is a property of the class with the name
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
                match class.get::<_, Function>("__index_miss_handler") {
                    Ok(function) => {
                        return function.bind(obj_table)?.call(index)
                    },
                    Err(_) => {}
                }
            }
            // TODO property miss handler if index doesn't exst
            Ok(rlua::Value::Nil)
        }
    }
}

/// Default new indexing (assignment) of an Awesome object.
///
/// Automatically looks up contents in meta table, so instead of overriding this
/// it's easier to just add the required data in the meta table.
pub fn default_newindex<'lua>(_: &'lua Lua,
                              (obj_table, index, val):
                              (Table<'lua>, String, Value<'lua>))
                              -> rlua::Result<Value<'lua>> {
    // Look up in metatable first
    if let Some(meta) = obj_table.get_metatable() {
        if let Ok(val) = meta.raw_get::<_, Value>(index.clone()) {
            match val {
                Value::Nil => {},
                val => return Ok(val)
            }
        }
        let class = meta.get::<_, Table>("__class")?;
        let props = class.get::<_, Vec<Property>>("properties")?;
        for prop in props {
            if prop.name.as_str() == index {
                // Property exists and has a newindex callback
                if let Some(newindex) = prop.cb_newindex {
                    return newindex.bind(obj_table)?.call(val)
                }
            }
        }
        match class.get::<_, Function>("__newindex_miss_handler") {
            Ok(function) => {
                return function.bind(obj_table)?.call(index)
            },
            Err(_) => {}
        }
        // TODO property miss handler if index doesn't exst
    }
    Ok(Value::Nil)
}

fn connect_signal(lua: &Lua, (obj_table, signal, func): (Table, String, Function))
                  -> rlua::Result<()> {
    signal::connect_signal(lua, obj_table.into(), signal, &[func])
}

fn disconnect_signal(lua: &Lua, (obj_table, signal): (Table, String))
                     -> rlua::Result<()> {
    signal::disconnect_signal(lua, obj_table.into(), signal)
}

fn emit_signal(lua: &Lua, (obj_table, signal, args): (Table, String, Value))
               -> rlua::Result<()> {
    signal::emit_signal(lua, obj_table.into(), signal, args)
}
