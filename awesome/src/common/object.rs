//! Utility methods and constructors for Lua objects

use super::class::Class;
use super::property::Property;
use super::signal;
use rlua::{self, AnyUserData, FromLua, Function, Lua, MetaMethod, Table, ToLua, UserData,
           UserDataMethods, Value};
use std::convert::From;
use std::fmt::Display;
use std::cell;
use std::marker::PhantomData;

/// Define the traits for states
pub trait State: UserData + Default + Display + Send {}
impl<T> State for T where T: UserData + Default + Display + Send {}

/// All Lua objects can be cast to this.
#[derive(Debug)]
pub struct Object<'lua, S: State>{
    pub obj: AnyUserData<'lua>,
    state: PhantomData<S>,
}

impl<'lua, S: State> Clone for Object<'lua, S> {
    fn clone(&self) -> Self {
        Object { obj: self.obj.clone(), state: PhantomData }
    }
}

impl<'lua, S: State> From<AnyUserData<'lua>> for Object<'lua, S> {
    fn from(obj: AnyUserData<'lua>) -> Self {
        Object { obj, state: PhantomData }
    }
}

impl<'lua, S: State> Into<AnyUserData<'lua>> for Object<'lua, S> {
    fn into(self) -> AnyUserData<'lua> {
        self.obj
    }
}

/// Construct a new object, used when using the default Objectable::new.
pub struct ObjectBuilder<'lua, S: State> {
    lua: &'lua Lua,
    object: Object<'lua, S>
}

impl<'lua, S: State> ObjectBuilder<'lua, S> {
    pub fn add_to_meta(self, new_meta: Table<'lua>) -> rlua::Result<Self> {
        let meta = self.object
                       .table()?
                       .get_metatable()
                       .expect("Object had no meta table");
        for entry in new_meta.pairs::<rlua::Value, rlua::Value>() {
            let (key, value) = entry?;
            meta.set(key, value)?;
        }
        self.object.table()?.set_metatable(Some(meta));
        Ok(self)
    }

    #[allow(dead_code)]
    pub fn add_to_signals(self, name: String, func: rlua::Function) -> rlua::Result<Self> {
        signal::connect_signal(self.lua, self.object.clone(), name, &[func])?;
        Ok(self)
    }

    pub fn handle_constructor_argument(self, args: Table) -> rlua::Result<Self> {
        let meta = self.object
                       .table()?
                       .get_metatable()
                       .expect("Object had no meta table");
        let class = meta.get::<_, AnyUserData>("__class")?;
        let class_table = class.get_user_value::<Table>()?;
        let props = class_table.get::<_, Vec<Property>>("properties")?;

        // Handle all table entries that correspond to known properties,
        // silently ignore all other keys
        for pair in args.pairs() {
            let (key, value): (Value, Value) = pair?;
            if let rlua::Value::String(key) = key {
                if let Ok(key) = key.to_str() {
                    // Find the property
                    for prop in props.iter() {
                        if prop.name == key {
                            // Property exists and has a cb_new callback
                            if let Some(ref new) = prop.cb_new {
                                let _: () = new.bind(self.object.clone())?.call(value)?;
                                break
                            }
                        }
                    }
                }
            }
        }
        Ok(self)
    }

    pub fn build(self) -> Object<'lua, S> {
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
impl<'lua, S: State> Object<'lua, S> {
    pub fn cast(obj: AnyUserData<'lua>) -> rlua::Result<Self> {
        if obj.is::<S>()? {
            Ok(obj.into())
        } else {
            use rlua::Error::RuntimeError;
            Err(RuntimeError("Could not cast object to concrete type".into()))
        }
    }

    pub fn state(&self) -> rlua::Result<cell::Ref<S>> {
        Ok(self.obj.borrow::<S>()?)
    }

    /// Gets a mutable reference to the internal state for the concrete object.
    pub fn get_object_mut(&mut self) -> rlua::Result<cell::RefMut<S>> {
        Ok(self.obj.borrow_mut::<S>()?)
    }

    pub fn signals(&self) -> rlua::Result<rlua::Table<'lua>> {
        self.table()?.get::<_, Table>("signals")
    }

    pub fn table(&self) -> rlua::Result<Table<'lua>> {
        self.obj.get_user_value::<Table<'lua>>()
    }

    /// Lua objects in Way Cooler are just how they are in Awesome:
    /// * We expose the user data directly.
    /// * State for the object is stored using set_user_value in a "wrapper"
    /// table. * The wrapper table has a data field which hosts the data.
    /// * Class methods/attributes are on the meta table of the wrapper table.
    pub fn allocate(lua: &'lua Lua, class: Class<S>) -> rlua::Result<ObjectBuilder<'lua, S>> {
        let obj = lua.create_userdata(S::default())?;
        // TODO Increment the instance count
        let wrapper_table = lua.create_table()?;
        let data_table = lua.create_table()?;
        wrapper_table.set("data", data_table)?;
        let meta = lua.create_table()?;
        meta.set("__class", class)?;
        meta.set("properties", Vec::<Property>::new().to_lua(lua)?)?;
        meta.set("signals", lua.create_table()?)?;
        meta.set("connect_signal", lua.create_function(connect_signal::<S>)?)?;
        meta.set("disconnect_signal", lua.create_function(disconnect_signal::<S>)?)?;
        meta.set("emit_signal", lua.create_function(emit_signal::<S>)?)?;
        meta.set("__index", meta.clone())?;
        meta.set("__tostring",
                  lua.create_function(|_, data: AnyUserData| {
                                           Ok(format!("{}", data.borrow::<S>()?))
                                       })?)?;
        wrapper_table.set_metatable(Some(meta));
        obj.set_user_value(wrapper_table)?;
        // TODO Emit new signal event
        let object = Object { obj, state: PhantomData };
        Ok(ObjectBuilder { object, lua })
    }
}

impl<'lua, S: State> ToLua<'lua> for Object<'lua, S> {
    fn to_lua(self, _: &'lua Lua) -> rlua::Result<Value<'lua>> {
        Ok(Value::UserData(self.obj))
    }
}

impl<'lua, S: State> FromLua<'lua> for Object<'lua, S> {
    fn from_lua(val: Value<'lua>, _lua: &'lua Lua) -> rlua::Result<Self> {
        if let Value::UserData(obj) = val {
            Object::cast(obj)
        } else {
            Err(rlua::Error::RuntimeError("Invalid data supplied".into()))
        }
    }
}

/// Can be used for implementing UserData for Lua objects. This provides some
/// default metafunctions.
pub fn default_add_methods<S>(methods: &mut UserDataMethods<S>)
    where S: State
{
    methods.add_meta_function(MetaMethod::Index, default_index::<S>);
    methods.add_meta_function(MetaMethod::NewIndex, default_newindex::<S>);
    methods.add_meta_function(MetaMethod::ToString, default_tostring::<S>);
}

/// Default indexing of an Awesome object.
///
/// Automatically looks up contents in meta table, so instead of overriding this
/// it's easier to just add the required data in the meta table.
pub fn default_index<'lua, S: State>(lua: &'lua Lua,
                                    (obj, index): (Object<'lua, S>, Value<'lua>))
                                    -> rlua::Result<Value<'lua>> {
    // Look up in metatable first
    let obj_table = obj.table()?;
    let meta = obj_table.get_metatable().expect("Object had no metatable");
    if meta.get::<_, AnyUserData>("__class").is_ok() {
        if let Ok(val) = meta.raw_get::<_, Value>(index.clone()) {
            match val {
                Value::Nil => {}
                val => return Ok(val)
            }
        }
    }
    let index = match String::from_lua(index, lua) {
        Ok(s) => s,
        Err(_) => return Ok(Value::Nil)
    };
    match index.as_str() {
        "valid" => {
            Ok(Value::Boolean(if let Ok(class) = meta.get::<_, AnyUserData>("__class") {
                                  let class: Class<S> = class.into();
                                  class.checker()?.map(|checker| checker(obj)).unwrap_or(true)
                              } else {
                                  false
                              }))
        }
        "data" => obj_table.to_lua(lua),
        index => {
            // Try see if there is a property of the class with the name
            if let Ok(class) = meta.get::<_, AnyUserData>("__class") {
                let class_table = class.get_user_value::<Table>()?;
                let props = class_table.get::<_, Vec<Property>>("properties")?;
                for prop in props {
                    if prop.name.as_str() == index {
                        // Property exists and has an index callback
                        if let Some(index) = prop.cb_index {
                            return index.call(obj)
                        }
                    }
                }
                if let Some(meta) = class_table.get_metatable() {
                    match meta.get::<_, Function>("__index_miss_handler") {
                        Ok(function) => return function.bind(obj)?.call(index),
                        Err(_) => {}
                    }
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
pub fn default_newindex<'lua, S: State>(
                _: &'lua Lua,
                (obj, index, val): (Object<'lua, S>, String, Value<'lua>))
                -> rlua::Result<Value<'lua>> {
    // Look up in metatable first
    let obj_table = obj.table()?;
    if let Some(meta) = obj_table.get_metatable() {
        if let Ok(val) = meta.raw_get::<_, Value>(index.clone()) {
            match val {
                Value::Nil => {}
                val => return Ok(val)
            }
        }
        let class = meta.get::<_, AnyUserData>("__class")?;
        let class_table = class.get_user_value::<Table>()?;
        let props = class_table.get::<_, Vec<Property>>("properties")?;
        for prop in props {
            if prop.name.as_str() == index {
                // Property exists and has a newindex callback
                if let Some(newindex) = prop.cb_newindex {
                    return newindex.bind(obj.clone())?.call(val)
                }
            }
        }
        if let Some(meta) = class_table.get_metatable() {
            match meta.get::<_, Function>("__newindex_miss_handler") {
                Ok(function) => return function.bind(obj)?.call((index, val)),
                Err(_) => {}
            }
        }
        // TODO property miss handler if index doesn't exst
    }
    Ok(Value::Nil)
}

pub fn default_tostring<'lua, S>(_: &'lua Lua, obj: AnyUserData<'lua>) -> rlua::Result<String>
        where S: State {
    let obj: Object<'lua, S> = obj.into();
    let obj_table = obj.table()?;
    if let Some(meta) = obj_table.get_metatable() {
        let class = meta.get::<_, AnyUserData>("__class")?;
        let class_table = class.get_user_value::<Table>()?;
        let name = class_table.get::<_, String>("name")?;
        return Ok(format!("{}: {:p}", name, &obj.obj as *const _))
    }
    Err(rlua::Error::UserDataTypeMismatch)
}

fn connect_signal<S: State>(lua: &Lua,
                            (obj, signal, func): (Object<S>, String, Function))
                            -> rlua::Result<()> {
    signal::connect_signal(lua, obj.into(), signal, &[func])
}

fn disconnect_signal<S: State>(lua: &Lua,
                               (obj, signal): (Object<S>, String))
                               -> rlua::Result<()> {
    signal::disconnect_signal(lua, obj.into(), signal)
}

fn emit_signal<S: State>(lua: &Lua,
                        (obj, signal, args): (Object<S>, String, Value))
                        -> rlua::Result<()> {
    signal::emit_object_signal(lua, obj.into(), signal, args)
}
