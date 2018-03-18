//! TODO Fill in

use std::fmt::{self, Display, Formatter};
use std::default::Default;
use rlua::{self, Table, Lua, UserData, ToLua, Value, UserDataMethods, AnyUserData};
use super::object::{self, Object, Objectable};
use super::class::{self, Class, ClassBuilder};
use super::property::Property;
use super::signal;

#[derive(Clone, Debug)]
pub struct TagState {
    name: Option<String>,
    // TODO Fill in
    dummy: i32
}

pub struct Tag<'lua>(Object<'lua>);

impl Default for TagState {
    fn default() -> Self {
        TagState {
            name: None,
            dummy: 0
        }
    }
}

impl <'lua> Tag<'lua> {
    fn new(lua: &'lua Lua, args: Table) -> rlua::Result<Object<'lua>> {
        let class = class::class_setup(lua, "tag")?;
        Ok(Tag::allocate(lua, class)?
           .handle_constructor_argument(args)?
           .build())
    }
}

impl Display for TagState {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Tag: {:p}", self)
    }
}

impl <'lua> ToLua<'lua> for Tag<'lua> {
    fn to_lua(self, lua: &'lua Lua) -> rlua::Result<Value<'lua>> {
        self.0.to_lua(lua)
    }
}

impl UserData for TagState {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        object::default_add_methods(methods);
    }
}

pub fn init(lua: &Lua) -> rlua::Result<Class> {
    method_setup(lua, Class::builder(lua, "tag", None)?)?
        .save_class("tag")?
        .build()
}

fn method_setup<'lua>(lua: &'lua Lua, builder: ClassBuilder<'lua>) -> rlua::Result<ClassBuilder<'lua>> {
    // TODO Do properly
    use super::dummy;
    builder.method("connect_signal".into(), lua.create_function(dummy)?)?
           .method("__call".into(), lua.create_function(|lua, args: Table| Tag::new(lua, args))?)?
           .property(Property::new("name".into(),
                                   Some(lua.create_function(set_name)?),
                                   Some(lua.create_function(get_name)?),
                                   Some(lua.create_function(set_name)?)))
}

impl_objectable!(Tag, TagState);

fn set_name<'lua>(lua: &'lua Lua, (obj, val): (AnyUserData<'lua>, String))
                    -> rlua::Result<Value<'lua>> {
    let mut tag = Tag::cast(obj.clone().into())?;
    tag.get_object_mut()?.name = Some(val.clone());
    signal::emit_object_signal(lua,
                        obj.into(),
                        "property::name".into(),
                        ())?;
    Ok(Value::Nil)
}

fn get_name<'lua>(lua: &'lua Lua, obj: AnyUserData<'lua>)
                  -> rlua::Result<Value<'lua>> {
    match obj.borrow::<TagState>()?.name {
        None => Ok(Value::Nil),
        Some(ref name) => Ok(Value::String(lua.create_string(&name)?))
    }
}

#[cfg(test)]
mod test {
    use rlua::Lua;
    use super::super::tag;

    #[test]
    fn tag_name_empty() {
        let lua = Lua::new();
        tag::init(&lua).unwrap();
        lua.eval(r#"
assert(type(tag{}.name) == "nil")
"#, None).unwrap()
    }

    #[test]
    fn tag_name_change() {
        let lua = Lua::new();
        tag::init(&lua).unwrap();
        lua.eval(r#"
local t = tag{ name = "a very cool tag" }
assert(t.name == "a very cool tag")
"#, None).unwrap()
    }

    #[test]
    fn tag_name_signal() {
        let lua = Lua::new();
        tag::init(&lua).unwrap();
        lua.eval(r#"
local t = tag{}

local called = 0
t:connect_signal("property::name", function(t)
    assert(t.name == "bye")
    called = called + 1
end)

t.name = "bye"
assert(t.name == "bye")
assert(called == 1)
"#, None).unwrap()
    }
}
