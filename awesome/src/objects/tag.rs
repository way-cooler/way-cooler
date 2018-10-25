//! A tag is similar to a workspace, except a client can be attached
//! to multiple tags at once.

use std::default::Default;
use std::fmt::{self, Display, Formatter};

use rlua::{self, AnyUserData, Integer, Lua, Table, ToLua, UserData,
           UserDataMethods, Value};

use common::{class::{self, Class, ClassBuilder},
             object::{self, Object, ObjectBuilder, Objectable},
             property::Property,
             signal};

use objects::client::{ClientState, Client};

pub const TAG_LIST: &'static str = "__tag_list";

#[derive(Clone, Debug)]
pub struct TagState {
    name: Option<String>,
    selected: bool,
    activated: bool
}

pub struct Tag<'lua>(Object<'lua>);

impl Default for TagState {
    fn default() -> Self {
        TagState { name: None,
                   selected: false,
                   activated: false }
    }
}

impl<'lua> Tag<'lua> {
    fn new(lua: &'lua Lua, args: Table) -> rlua::Result<Object<'lua>> {
        let class = class::class_setup(lua, "tag")?;
        Ok(object_setup(lua, Tag::allocate(lua, class)?)?
                .handle_constructor_argument(args)?
                .build())
    }

    pub fn get_clients(&self) -> rlua::Result<Table<'lua>> {
        self.0.table()?.get("__clients")
    }

    pub fn set_clients(&mut self, clients: rlua::Table) -> rlua::Result<Value> {
        // TODO: this is a really inefficient solution ( O(n*m) for the search )
        //   Since it does not generally treat big arrays, this may be acceptable,
        //   However a faster algorithm would not hurt if someone finds one

        let prev_clients = self.get_clients()?;

        // Indexes refering to prev_clients
        let mut not_common_clients: Vec<u32> = (1..(prev_clients.len()? + 1) as u32)
                                                    .collect();
        let mut num_common_clients = 0;
            
        for client in clients.clone().sequence_values::<AnyUserData>() {
            let mut idx = -1;
            let client = Client::cast(client?.clone().into())?;
            let self_ref = &*client.state()? as *const _;

            for pair in prev_clients.clone().pairs::<Integer, AnyUserData>() {
                let (i, c) = pair?;
                if &*c.borrow::<ClientState>()? as *const _ == self_ref {
                    idx = i;
                    num_common_clients += 1;
                    not_common_clients.remove((i - num_common_clients) as usize);
                    break;
                }
            }
            if idx != -1 {
                // TODO: emit signal
            }
        }
        for idx in not_common_clients.iter() {
            // prev_clients.get::<Integer, ClientState>(idx)?
            // TODO: emit signal and garbage if not referenced anymore
        }
        self.0.table()?.set("__clients", clients)?;
        Ok(Value::Nil)
    }

    pub fn client_index(&self, client: &Client) -> rlua::Result<Integer> {
        let clients = self.get_clients()?;
        let self_ref = &*client.state()? as *const _;
        for pair in clients.pairs::<Integer, AnyUserData<'lua>>() {
            let (i, c) = pair?;
            if &*c.borrow::<ClientState>()? as *const _ == self_ref {
                return Ok(i);
            }
        }
        Err(rlua::Error::RuntimeError("Client not found".to_string()))
    }

    pub fn tag_client(&mut self, obj: AnyUserData<'lua>) -> rlua::Result<Value> {
        let client = Client::cast(obj.clone().into())?;
        if let Ok(_) = self.client_index(&client) {
            return Ok(Value::Nil);
        }
        let clients = self.get_clients()?;
        clients.set(clients.len()? + 1, obj)?;
        self.set_clients(clients)?;
        Ok(Value::Nil)
    }

    pub fn untag_client(&mut self, obj: AnyUserData<'lua>) -> rlua::Result<Value> {
        let client = Client::cast(obj.clone().into())?;
        let index = self.client_index(&client)?;
        let clients = self.get_clients()?;

        for i in index..clients.len()? {
            clients.set(i, clients.get::<_, Value>(i + 1)?)?;
        }
        clients.set(clients.len()?, Value::Nil)?;
        self.set_clients(clients)?;
        Ok(Value::Nil)
    }
}

impl Display for TagState {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Tag: {:p}", self)
    }
}

impl<'lua> ToLua<'lua> for Tag<'lua> {
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
    lua.set_named_registry_value(TAG_LIST, lua.create_table()?)?;
    method_setup(lua, Class::builder(lua, "tag", None)?)?.save_class("tag")?
                                                         .build()
}

fn method_setup<'lua>(lua: &'lua Lua,
                      builder: ClassBuilder<'lua>)
                      -> rlua::Result<ClassBuilder<'lua>> {
    // TODO Do properly
    use super::dummy;
    builder.method("connect_signal".into(), lua.create_function(dummy)?)?
           .method("__call".into(),
                   lua.create_function(|lua, args: Table| Tag::new(lua, args))?)?
           .property(Property::new("name".into(),
                                   Some(lua.create_function(set_name)?),
                                   Some(lua.create_function(get_name)?),
                                   Some(lua.create_function(set_name)?)))?
           .property(Property::new("selected".into(),
                                   Some(lua.create_function(set_selected)?),
                                   Some(lua.create_function(get_selected)?),
                                   Some(lua.create_function(set_selected)?)))?
           .property(Property::new("activated".into(),
                                   Some(lua.create_function(set_activated)?),
                                   Some(lua.create_function(get_activated)?),
                                   Some(lua.create_function(set_activated)?)))?
           .property(Property::new("clients".into(),
                                   Some(lua.create_function(get_clients)?),
                                   Some(lua.create_function(|lua, _: Value| {
                                    Ok(Value::Function(lua.create_function(get_clients)?))
                                   })?),
                                   None))
}

impl_objectable!(Tag, TagState);

fn object_setup<'lua>(lua: &'lua Lua,
                      builder: ObjectBuilder<'lua>)
                      -> rlua::Result<ObjectBuilder<'lua>> {
    let table = lua.create_table()?;
    table.set("__clients", lua.create_table()?)?; // store clients in lua
    builder.add_to_meta(table)
}

fn set_name<'lua>(lua: &'lua Lua,
                  (obj, val): (AnyUserData<'lua>, String))
                  -> rlua::Result<Value<'lua>> {
    let mut tag = Tag::cast(obj.clone().into())?;
    tag.get_object_mut()?.name = Some(val.clone());
    signal::emit_object_signal(lua, obj.into(), "property::name".into(), ())?;
    Ok(Value::Nil)
}

fn get_name<'lua>(lua: &'lua Lua, obj: AnyUserData<'lua>) -> rlua::Result<Value<'lua>> {
    match obj.borrow::<TagState>()?.name {
        None => Ok(Value::Nil),
        Some(ref name) => Ok(Value::String(lua.create_string(&name)?))
    }
}

fn set_selected<'lua>(lua: &'lua Lua,
                      (obj, val): (AnyUserData<'lua>, bool))
                      -> rlua::Result<Value<'lua>> {
    let mut tag = Tag::cast(obj.clone().into())?;
    {
        let mut tag = tag.get_object_mut()?;
        if tag.selected == val {
            return Ok(Value::Nil)
        }
        tag.selected = val;
    }
    signal::emit_object_signal(lua, obj.into(), "property::selected".into(), ())?;
    Ok(Value::Nil)
}

fn get_selected<'lua>(_: &'lua Lua, obj: AnyUserData<'lua>) -> rlua::Result<Value<'lua>> {
    Ok(Value::Boolean(obj.borrow::<TagState>()?.selected))
}

fn set_activated<'lua>(lua: &'lua Lua,
                       (obj, val): (AnyUserData<'lua>, bool))
                       -> rlua::Result<Value<'lua>> {
    let mut tag = Tag::cast(obj.clone().into())?;
    {
        let mut tag = tag.get_object_mut()?;
        if tag.activated == val {
            return Ok(Value::Nil)
        }
        tag.activated = val;
    }
    let activated_tags = lua.named_registry_value::<Table>(TAG_LIST)?;
    let activated_tags_count = activated_tags.len()?;
    if val {
        let index = activated_tags_count + 1;
        activated_tags.set(index, obj.clone())?;
    } else {
        // Find and remove the tag in/from the list of tags
        {
            let tag_ref = &*obj.borrow::<TagState>()? as *const _;
            let mut found = false;
            for pair in activated_tags.clone().pairs::<Integer, AnyUserData>() {
                let (key, value) = pair?;
                if tag_ref == &*value.borrow::<TagState>()? as *const _ {
                    found = true;
                    // Now remove this by shifting everything down...
                    for index in key..activated_tags_count {
                        activated_tags.set(index, activated_tags.get::<_, Value>(index + 1)?)?;
                    }
                    activated_tags.set(activated_tags_count, Value::Nil)?;
                    break
                }
            }
            assert!(found);
        }
        set_selected(lua, (obj.clone(), false))?;
    }
    signal::emit_object_signal(lua, obj.into(), "property::activated".into(), ())?;
    Ok(Value::Nil)
}

fn get_activated<'lua>(_: &'lua Lua, obj: AnyUserData<'lua>) -> rlua::Result<Value<'lua>> {
    Ok(Value::Boolean(obj.borrow::<TagState>()?.activated))
}

fn get_clients<'lua>(_lua: &'lua Lua,  (obj, val): (AnyUserData<'lua>, Value<'lua>)) -> rlua::Result<Table<'lua>> {
    // TODO:
    // - Right now this is a property that returns a function. Can we get rid of
    // some of this indirection?
    let mut tag = Tag(obj.into());
    match val {
        Value::Table(clients) => tag.set_clients(clients),
        _ => Ok(Value::Nil),
    }?;
    tag.get_clients()
}

#[cfg(test)]
mod test {
    use super::super::{tag::{self, Tag}, client::{self, Client}};
    use common::object::Objectable;
    use rlua::{Lua, Value, ToLua};

    #[test]
    fn tag_name_empty() {
        let lua = Lua::new();
        tag::init(&lua).unwrap();
        lua.eval(
                 r#"
assert(type(tag{}.name) == "nil")
"#,
                 None
        ).unwrap()
    }

    #[test]
    fn tag_name_change() {
        let lua = Lua::new();
        tag::init(&lua).unwrap();
        lua.eval(
                 r#"
local t = tag{ name = "a very cool tag" }
assert(t.name == "a very cool tag")
"#,
                 None
        ).unwrap()
    }

    #[test]
    fn tag_name_signal() {
        let lua = Lua::new();
        tag::init(&lua).unwrap();
        lua.eval(
                 r#"
local t = tag{}

local called = 0
t:connect_signal("property::name", function(t)
    assert(t.name == "bye")
    called = called + 1
end)

t.name = "bye"
assert(t.name == "bye")
assert(called == 1)
"#,
                 None
        ).unwrap()
    }

    #[test]
    fn tag_selected() {
        let lua = Lua::new();
        tag::init(&lua).unwrap();
        lua.eval(
                 r#"
local t = tag{}
assert(t.selected == false)

local called = 0
t:connect_signal("property::selected", function(t)
    called = called + 1
end)

t.selected = false
assert(t.selected == false)
assert(called == 0)

t.selected = true
assert(t.selected == true)
assert(called == 1)

t.selected = true
assert(t.selected == true)
assert(called == 1)

t.selected = false
assert(t.selected == false)
assert(called == 2)
"#,
                 None
        ).unwrap()
    }

    #[test]
    fn tag_activated() {
        let lua = Lua::new();
        tag::init(&lua).unwrap();
        lua.eval(
                 r#"
local t = tag{}
assert(t.activated == false)

local called = 0
t:connect_signal("property::activated", function(t)
    called = called + 1
end)

t.activated = false
assert(t.activated == false)
assert(called == 0)

t.activated = true
assert(t.activated == true)
assert(called == 1)

t.activated = true
assert(t.activated == true)
assert(called == 1)

t.activated = false
assert(t.activated == false)
assert(called == 2)
"#,
                 None
        ).unwrap()
    }

    #[test]
    fn tag_activated_selected() {
        let lua = Lua::new();
        tag::init(&lua).unwrap();
        lua.eval(
                 r#"
local t = tag{selected = true, activated = true}

local called_selected, called_activated = 0, 0
t:connect_signal("property::activated", function(t)
    called_activated = called_activated + 1
end)
t:connect_signal("property::selected", function(t)
    called_selected = called_selected + 1
end)

t.activated = false
assert(t.activated == false)
assert(t.selected == false)
assert(called_activated == 1)
assert(called_selected == 1)
"#,
                 None
        ).unwrap()
    }

    #[test]
    fn tag_client() {
        let lua = Lua::new();
        tag::init(&lua).unwrap();
        lua.eval(
             r#"
local t = tag{}
assert(#t:clients() == 0, "Cannot get the clients")
"#,
             None
        ).unwrap()
    }

    #[test]
    fn tag_set_client() {
        let lua = Lua::new();
        tag::init(&lua).unwrap();
        client::init(&lua).unwrap();
        lua.eval(
             r#"
local c = client{}
local t = tag{}
t:clients({ c })
assert(c, "client doesn't exists")
assert(#t:clients() == 1, "Tag doesn't have the clients")
assert(t:clients()[1] == c, "Pass by value, not by reference")
"#,
             None
        ).unwrap()
    }

    #[test]
    fn tag_share_client() {
        let lua = Lua::new();
        tag::init(&lua).unwrap();
        client::init(&lua).unwrap();
        lua.eval(
             r#"
local c = client{}

local t = tag{}
t:clients({ c })
local t2 = tag{}
t2:clients({ c })

assert(c, "client doesn't exists")
assert(#t:clients() == 1, "Tag doesn't have the clients")
assert(t:clients()[1] == c, "Pass by value, not by reference")
assert(#t2:clients() == 1, "Tag doesn't have the clients")
assert(t2:clients()[1] == c, "Pass by value, not by reference")
assert(t2:clients()[1] == t:clients()[1], "Tags does not share the clients")
"#,
             None
        ).unwrap()
    }

    #[test]
    fn tag_new_client() {
        let lua = Lua::new();
        tag::init(&lua).unwrap();
        client::init(&lua).unwrap();
        lua.eval(
             r#"
local c = client{}
local t = tag{ clients = { c } }
assert(c, "client doesn't exists")
assert(#t:clients() == 1, "Tag doesn't have the clients")
assert(t:clients()[1] == c, "Pass by value, not by reference")
"#,
             None
        ).unwrap()
    }

    #[test]
    fn tag_tag_client() {
        let lua = Lua::new();
        let globals = lua.globals();
        tag::init(&lua).unwrap();
        client::init(&lua).unwrap();
        let client = Client::new(&lua, lua.create_table().unwrap()).unwrap().to_lua(&lua).unwrap();
        if let Value::UserData(c) = client {
            let mut t = Tag::cast(Tag::new(&lua, lua.create_table().unwrap()).unwrap().into()).unwrap();
            t.tag_client(c.clone()).unwrap();
            globals.set("t", t).unwrap();
            globals.set("c", c).unwrap();
            lua.eval(
                 r#"
    assert(c, "client doesn't exists")
    assert(#t:clients() == 1, "Tag doesn't have the clients")
    assert(t:clients()[1] == c, "Pass by value, not by reference")
    "#,
                 None
            ).unwrap()
        }
    }

    #[test]
    fn tag_untag_client() {
        let lua = Lua::new();
        let globals = lua.globals();
        tag::init(&lua).unwrap();
        client::init(&lua).unwrap();
        let client = Client::new(&lua, lua.create_table().unwrap()).unwrap().to_lua(&lua).unwrap();
        if let Value::UserData(c) = client {
            let mut t = Tag::cast(Tag::new(&lua, lua.create_table().unwrap()).unwrap().into()).unwrap();
            t.tag_client(c.clone()).unwrap();
            t.untag_client(c.clone()).unwrap();
            globals.set("t", t).unwrap();
            globals.set("c", c).unwrap();
            lua.eval(
                 r#"
    assert(c, "client doesn't exists")
    assert(#t:clients() == 0, "Client are not untagged")
    "#,
                 None
            ).unwrap()
        }
    }
}
