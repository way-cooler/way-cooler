//! Utility methods and constructors for Lua classes

use std::sync::Arc;
use std::default::Default;
use rlua::{self, Lua, ToLua, Table, MetaMethod, UserData, UserDataMethods,
           Value};
use super::object::Object;
use super::property::Property;

pub type Allocator = fn(&Lua) -> Object;
pub type Collector = fn(Object);
pub type PropF = fn(&Lua, Object) -> i32;
pub type Checker = fn(&Object) -> bool;

pub struct Class<'lua> {
    table: Table<'lua>
}

impl <'lua> ToLua<'lua> for Class<'lua> {
    fn to_lua(self, lua: &'lua Lua) -> rlua::Result<Value<'lua>> {
        self.table.to_lua(lua)
    }
}

pub struct ClassState {
    name: String,
    // TODO Needed? If we store this like the object surely it's not needed...
    /// The global Lua key that corresponds to this class' signal list.
    /// Stored this way so that it's not tied to Lua lifetime.
    signals_global: Option<String>,
    // TODO Another storage as a key in lua...hmm
    parent: Option<String>,
    allocator: Option<Allocator>,
    collector: Option<Collector>,
    properties: Vec<Property>,
    index_miss_property: Option<PropF>,
    newindex_miss_property: Option<PropF>,
    checker: Option<Checker>,
    instances: u32,
    tostring: Option<PropF>,
    /// The global Lua key that corresponds to this class' function.
    /// Stored this way so that it's not tied to Lua lifetime.
    index_miss_handler_global: Option<String>,
    /// The global Lua key that corresponds to this class' function.
    /// Stored this way so that it's not tied to Lua lifetime.
    newindex_miss_handler_global: Option<String>,
}

impl Default for ClassState {
    fn default() -> Self {
        ClassState {
            name: String::default(),
            signals_global: Option::default(),
            parent: Option::default(),
            allocator: Option::default(),
            collector: Option::default(),
            properties: Vec::new(),
            index_miss_property: Option::default(),
            newindex_miss_property: Option::default(),
            checker: Option::default(),
            instances: 0,
            tostring: Option::default(),
            index_miss_handler_global: Option::default(),
            newindex_miss_handler_global: Option::default(),

        }
    }
}

impl UserData for ClassState {}

impl <'lua> Class<'lua> {
    pub fn new<ALLOC, COLLECT, CHECK, PROP>(lua: &'lua Lua,
                                            allocator: ALLOC,
                                            collector: COLLECT,
                                            index_miss_property: PROP,
                                            newindex_miss_property: PROP,
                                            checker: CHECK,
                                            tostring: PROP)
                                            -> rlua::Result<Self>
        where ALLOC: Into<Option<Allocator>>,
              COLLECT: Into<Option<Collector>>,
              CHECK: Into<Option<Checker>>,
              PROP: Into<Option<PropF>>,
    {
        let mut class = ClassState::default();
        class.allocator = allocator.into();
        class.collector = collector.into();
        class.index_miss_property = index_miss_property.into();
        class.newindex_miss_property = newindex_miss_property.into();
        class.checker = checker.into();
        // might not be needed
        class.tostring = tostring.into();

        // TODO Do same thing as in option, e.g throw this sucker in a table
        // set the meta table to be the correct thing.
        // the meta table is going to be the thing that changes, because different clasess have different methods
        // so do the same thing as in the object I guess

        let table = lua.create_table();
        table.set("data", class)?;
        Ok(Class { table })
    }
}
