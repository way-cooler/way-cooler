//! Utility methods and constructors for Lua classes

use std::default::Default;
use rlua::{self, Lua, ToLua, Table, UserData, Value, Function};
use super::object::{self, Object};
use super::property::Property;

pub type Allocator = fn(&Lua) -> rlua::Result<Object>;
pub type Collector = fn(Object);
pub type PropF<'lua> = rlua::Function<'lua>;
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
    checker: Option<Checker>,
    instances: u32,
    /// The global Lua key that corresponds to this class' function.
    /// Stored this way so that it's not tied to Lua lifetime.
    index_miss_handler_global: Option<String>,
    /// The global Lua key that corresponds to this class' function.
    /// Stored this way so that it's not tied to Lua lifetime.
    newindex_miss_handler_global: Option<String>,
}

pub struct ClassBuilder<'lua>{
    lua: &'lua Lua,
    class: Class<'lua>
}

impl <'lua> ClassBuilder<'lua> {
    pub fn method(self, name: String, meth: rlua::Function)
                  -> rlua::Result<Self> {
        let meta = self.class.table.get_metatable()
            .expect("Class had no meta table!");
        meta.set(name, meth)?;
        Ok(self)
    }

    pub fn property(self, prop: Property<'lua>) -> rlua::Result<Self> {
        let properties = self.class.table.get::<_, Table>("properties")?;
        let length = properties.len().unwrap_or(0) + 1;
        // TODO make sure no duplicate names...
        properties.set(length, prop)?;
        Ok(self)
    }

    pub fn save_class(mut self, name: &str)
                      -> rlua::Result<Self> {
        self.lua.globals().set(name, self.class.table)?;
        self.class.table = self.lua.globals().get(name)?;
        Ok(self)
    }

    pub fn build(self) -> rlua::Result<Class<'lua>> {
        Ok(self.class)
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
            checker: Option::default(),
            instances: 0,
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
               checker: Option<Checker>)
               -> rlua::Result<ClassBuilder<'lua>> {
        let mut class = ClassState::default();
        class.allocator = allocator;
        class.collector = collector;
        class.checker = checker;
        let table = lua.create_table();
        table.set("data", class)?;
        table.set("properties", Vec::<Property>::new().to_lua(lua)?)?;
        let meta = lua.create_table();
        meta.set("signals", lua.create_table())?;
        meta.set("set_index_miss_handler",
                 lua.create_function(set_index_miss_handler).bind(table.clone())?)?;
        meta.set("set_newindex_miss_handler",
                 lua.create_function(set_newindex_miss_handler).bind(table.clone())?)?;
        // TODO Is this the correct indexing function? Hm.
        meta.set("__index", lua.create_function(object::default_index))?;
        // TODO __tostring
        table.set_metatable(Some(meta));
        Ok(ClassBuilder{
            lua: lua,
            class: Class { table }
        })
    }

    pub fn properties(&self) -> rlua::Result<Table<'lua>> {
        self.table.get("properties")
    }
}

fn set_index_miss_handler<'lua>(lua: &'lua Lua, (obj, func): (Table, Function))
                                 -> rlua::Result<()> {
    let meta = obj.get_metatable()
        .expect("Object had no metatable");
    meta.set("__index_miss_handler", func)?;
    Ok(())
}
fn set_newindex_miss_handler<'lua>(lua: &'lua Lua, (obj, func): (Table, Function))
                                    -> rlua::Result<()> {
    let meta = obj.get_metatable()
        .expect("Object had no metatable");
    meta.set("__newindex_miss_handler", func)?;
    Ok(())
}

pub fn button_class(lua: &Lua) -> rlua::Result<Class> {
    let table = lua.globals().get::<_, Table>("button")
        .expect("Button class was not set! Did you call button::init?");
    // TODO Assert is correct table
    Ok(Class { table })
}
