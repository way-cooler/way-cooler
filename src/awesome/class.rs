//! Utility methods and constructors for Lua classes

use std::sync::Arc;
use std::default::Default;
use std::borrow::BorrowMut;
use rlua::{self, Lua, ToLua, Table, MetaMethod, UserData, UserDataMethods,
           Value, AnyUserData};
use super::object::{self, Object};
use super::property::Property;

pub type Allocator = fn(&Lua) -> rlua::Result<Object>;
pub type Collector = fn(Object);
pub type PropF = fn(&Lua, Object) -> i32;
pub type Checker = fn(&Object) -> bool;

#[derive(Debug)]
pub struct Class<'lua> {
    table: Table<'lua>
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

#[derive(Debug)]
pub struct ClassBuilder<'lua>(Class<'lua>);

impl <'lua> ClassBuilder<'lua> {
    pub fn method(self, lua: &Lua, name: String, meth: rlua::Function)
                  -> rlua::Result<Self> {
        let meta = self.0.table.get_metatable()
            .expect("Class had no meta table!");
        meta.set(name, meth)?;
        Ok(self)
    }

    pub fn property(self, prop: Property) -> rlua::Result<Self> {
        let class = self.0.table.get::<_, AnyUserData>("data")?;
        class.borrow_mut::<ClassState>()?.properties.push(prop);
        Ok(self)
    }

    pub fn build(self) -> rlua::Result<Class<'lua>> {
        Ok(self.0)
    }
}

impl <'lua> ToLua<'lua> for Class<'lua> {
    fn to_lua(self, lua: &'lua Lua) -> rlua::Result<Value<'lua>> {
        self.table.to_lua(lua)
    }
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
    pub fn new(lua: &'lua Lua,
               allocator: Option<Allocator>,
               collector: Option<Collector>,
               checker: Option<Checker>,
               index_miss_property: Option<PropF>,
               newindex_miss_property: Option<PropF>)
               -> rlua::Result<ClassBuilder> {
        let mut class = ClassState::default();
        class.allocator = allocator.into();
        class.collector = collector.into();
        class.index_miss_property = index_miss_property.into();
        class.newindex_miss_property = newindex_miss_property.into();
        class.checker = checker.into();

        // TODO Do same thing as in option, e.g throw this sucker in a table
        // set the meta table to be the correct thing.
        // the meta table is going to be the thing that changes, because different clasess have different methods
        // so do the same thing as in the object I guess

        let table = lua.create_table();
        table.set("data", class)?;
        let meta = lua.create_table();
        meta.set("signals", lua.create_table())?;
        meta.set("__index", lua.create_function(object::default_index))?;
        table.set_metatable(Some(meta));
        Ok(ClassBuilder(Class { table }))
    }
}


// TODO Implement
// TODO return rlua::Value in result, however that will cause lifetime issues...
pub fn index_miss_property(lua: &Lua, obj: Object) -> i32 {unimplemented!()}
pub fn newindex_miss_property(lua: &Lua, obj: Object) -> i32 {unimplemented!()}
