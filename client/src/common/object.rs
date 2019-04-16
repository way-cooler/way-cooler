//! Utility methods and constructors for Lua objects

use std::{cell, convert::From, marker::PhantomData};

use rlua::{
    self, AnyUserData, FromLua, Function, MetaMethod, Table, ToLua, UserData, UserDataMethods, Value
};

use super::{class::Class, property::Property, signal};

/// The ObjectStateType trait is used to constrain the generic data types in the Object and Class structs.
/// They can be transferred to and from Lua user data and force type checking
pub trait ObjectStateType: 'static + UserData + Default + Send {}

impl<T> ObjectStateType for T where T: 'static + UserData + Default + Send {}

/// All Lua objects can be cast to this.
#[derive(Debug)]
pub struct Object<'lua, S: ObjectStateType> {
    obj: AnyUserData<'lua>,
    state: PhantomData<S>
}

impl<'lua, S: ObjectStateType> Clone for Object<'lua, S> {
    fn clone(&self) -> Self {
        Object {
            obj: self.obj.clone(),
            state: PhantomData
        }
    }
}

impl<'lua, S: ObjectStateType> From<AnyUserData<'lua>> for Object<'lua, S> {
    fn from(obj: AnyUserData<'lua>) -> Self {
        Object {
            obj,
            state: PhantomData
        }
    }
}

impl<'lua, S: ObjectStateType> Into<AnyUserData<'lua>> for Object<'lua, S> {
    fn into(self) -> AnyUserData<'lua> {
        self.obj
    }
}

/// Construct a new object, used when using the default Objectable::new.
pub struct ObjectBuilder<'lua, S: ObjectStateType> {
    lua: rlua::Context<'lua>,
    object: Object<'lua, S>
}

impl<'lua, S: ObjectStateType> ObjectBuilder<'lua, S> {
    pub fn add_to_meta(self, new_meta: Table<'lua>) -> rlua::Result<Self> {
        let meta = self.object.get_metatable()?.expect("Object had no meta table");
        for entry in new_meta.pairs::<Value, Value>() {
            let (key, value) = entry?;
            meta.set(key, value)?;
        }
        self.object.set_metatable(meta)?;
        Ok(self)
    }

    #[allow(dead_code)]
    pub fn add_to_signals(self, name: String, func: Function<'lua>) -> rlua::Result<Self> {
        signal::connect_signal(self.lua, self.object.clone(), name, &[func])?;
        Ok(self)
    }

    pub fn handle_constructor_argument(self, args: Table<'lua>) -> rlua::Result<Self> {
        let meta = self.object.get_metatable()?.expect("Object had no meta table");
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
                                break;
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

/// Objects that represent OO lua objects.
///
/// Allows casting an object gotten back from the Lua runtime
/// into a concrete object so that Rust can do things with it.
impl<'lua, S: ObjectStateType> Object<'lua, S> {
    pub fn cast(obj: AnyUserData<'lua>) -> rlua::Result<Self> {
        if obj.is::<S>() {
            Ok(obj.into())
        } else {
            use rlua::Error::RuntimeError;
            Err(RuntimeError("Could not cast object to concrete type".into()))
        }
    }

    /// Gets a reference to the internal state for the concrete object.
    pub fn state(&self) -> rlua::Result<cell::Ref<S>> {
        Ok(self.obj.borrow::<S>()?)
    }

    /// Gets a mutable reference to the internal state for the concrete object.
    pub fn state_mut(&mut self) -> rlua::Result<cell::RefMut<S>> {
        Ok(self.obj.borrow_mut::<S>()?)
    }

    /// Get the signals of the for this object
    pub fn signals(&self) -> rlua::Result<Table<'lua>> {
        self.get_associated_data::<Table>("signals")
    }

    /// Set a value to keep inside lua associate with the object, but
    ///     which should not be transfered to Rust for various reason
    ///     (e.g. reference to other objects which cause GC problems)
    pub fn set_associated_data<D: ToLua<'lua>>(&self, key: &str, value: D) -> rlua::Result<()> {
        self.obj.get_user_value::<Table<'lua>>()?.set::<_, D>(key, value)
    }

    /// Get a value to keep inside lua associate with the object
    pub fn get_associated_data<D: FromLua<'lua>>(&self, key: &str) -> rlua::Result<D> {
        self.obj.get_user_value::<Table<'lua>>()?.get::<_, D>(key)
    }

    /// Get the metatable for this object
    pub fn get_metatable(&self) -> rlua::Result<Option<Table<'lua>>> {
        Ok(self.obj.get_user_value::<Table<'lua>>()?.get_metatable())
    }

    /// Set the metatable for this object
    pub fn set_metatable(&self, meta: Table<'lua>) -> rlua::Result<()> {
        self.obj
            .get_user_value::<Table<'lua>>()?
            .set_metatable(Some(meta));
        Ok(())
    }

    /// Lua objects in Way Cooler are just how they are in Awesome:
    /// * We expose the user data directly.
    /// * ObjectStateType for the object is stored using set_user_value in a "wrapper"
    /// table. * The wrapper table has a data field which hosts the data.
    /// * Class methods/attributes are on the meta table of the wrapper table.
    pub fn allocate(lua: rlua::Context<'lua>, class: Class<'lua, S>) -> rlua::Result<ObjectBuilder<'lua, S>> {
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
        meta.set("__tostring", lua.create_function(default_tostring::<S>)?)?;
        wrapper_table.set_metatable(Some(meta));
        obj.set_user_value(wrapper_table)?;
        // TODO Emit new signal event
        let object = Object {
            obj,
            state: PhantomData
        };
        Ok(ObjectBuilder { object, lua })
    }
}

impl<'lua, S: ObjectStateType> ToLua<'lua> for Object<'lua, S> {
    fn to_lua(self, _: rlua::Context<'lua>) -> rlua::Result<Value<'lua>> {
        Ok(Value::UserData(self.obj))
    }
}

impl<'lua, S: ObjectStateType> FromLua<'lua> for Object<'lua, S> {
    fn from_lua(val: Value<'lua>, _lua: rlua::Context<'lua>) -> rlua::Result<Self> {
        if let Value::UserData(obj) = val {
            Ok(obj.into())
        } else {
            Err(rlua::Error::RuntimeError("Invalid data supplied".into()))
        }
    }
}

/// Can be used for implementing UserData for Lua objects. This provides some
/// default metafunctions.
pub fn default_add_methods<'lua, S, M: UserDataMethods<'lua, S>>(methods: &mut M)
where
    S: ObjectStateType
{
    methods.add_meta_function(MetaMethod::Index, default_index::<S>);
    methods.add_meta_function(MetaMethod::NewIndex, default_newindex::<S>);
    methods.add_meta_function(MetaMethod::ToString, default_tostring::<S>);
}

/// Default indexing of an Awesome object.
///
/// Automatically looks up contents in meta table, so instead of overriding this
/// it's easier to just add the required data in the meta table.
pub fn default_index<'lua, S: ObjectStateType>(
    lua: rlua::Context<'lua>,
    (obj, index): (Object<'lua, S>, Value<'lua>)
) -> rlua::Result<Value<'lua>> {
    // Look up in metatable first
    let meta = obj.get_metatable()?.expect("Object had no metatable");
    if meta.get::<_, AnyUserData>("__class").is_ok() {
        if let Ok(val) = meta.raw_get::<_, Value>(index.clone()) {
            match val {
                Value::Nil => {},
                val => return Ok(val)
            }
        }
    }
    let index = match String::from_lua(index, lua) {
        Ok(s) => s,
        Err(_) => return Ok(Value::Nil)
    };
    match index.as_str() {
        "valid" => Ok(Value::Boolean(
            if let Ok(class) = meta.get::<_, AnyUserData>("__class") {
                let class: Class<S> = class.into();
                class.checker()?.map(|checker| checker(obj)).unwrap_or(true)
            } else {
                false
            }
        )),
        "data" => obj.obj.get_user_value(),
        index => {
            // Try see if there is a property of the class with the name
            if let Ok(class) = meta.get::<_, AnyUserData>("__class") {
                let class_table = class.get_user_value::<Table>()?;
                let props = class_table.get::<_, Vec<Property>>("properties")?;
                for prop in props {
                    if prop.name.as_str() == index {
                        // Property exists and has an index callback
                        if let Some(index) = prop.cb_index {
                            return index.call(obj);
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
pub fn default_newindex<'lua, S: ObjectStateType>(
    _: rlua::Context<'lua>,
    (obj, index, val): (Object<'lua, S>, String, Value<'lua>)
) -> rlua::Result<Value<'lua>> {
    // Look up in metatable first
    if let Some(meta) = obj.get_metatable()? {
        if let Ok(val) = meta.raw_get::<_, Value>(index.clone()) {
            match val {
                Value::Nil => {},
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
                    return newindex.bind(obj.clone())?.call(val);
                }
            }
        }
        if let Some(meta) = class_table.get_metatable() {
            match meta.get::<_, Function>("__newindex_miss_handler") {
                Ok(function) => return function.bind(obj)?.call((index, val)),
                Err(_) => {}
            }
        }
        // TODO property miss handler if index doesn't exist
    }
    Ok(Value::Nil)
}

pub fn default_tostring<'lua, S>(_: rlua::Context<'lua>, obj: Object<'lua, S>) -> rlua::Result<String>
where
    S: ObjectStateType
{
    if let Some(meta) = obj.get_metatable()? {
        let class = meta.get::<_, AnyUserData>("__class")?;
        let class_table = class.get_user_value::<Table>()?;
        let name = class_table.get::<_, String>("name")?;
        return Ok(format!("{}: {:p}", name, &*obj.state()?));
    }
    Err(rlua::Error::UserDataTypeMismatch)
}

fn connect_signal<'lua, S: ObjectStateType>(
    lua: rlua::Context<'lua>,
    (obj, signal, func): (Object<'lua, S>, String, Function<'lua>)
) -> rlua::Result<()> {
    signal::connect_signal(lua, obj.into(), signal, &[func])
}

fn disconnect_signal<'lua, S: ObjectStateType>(
    lua: rlua::Context<'lua>,
    (obj, signal): (Object<'lua, S>, String)
) -> rlua::Result<()> {
    signal::disconnect_signal(lua, obj.into(), signal)
}

fn emit_signal<'lua, S: ObjectStateType>(
    lua: rlua::Context<'lua>,
    (obj, signal, args): (Object<'lua, S>, String, Value<'lua>)
) -> rlua::Result<()> {
    signal::emit_object_signal(lua, obj.into(), signal, args)
}
