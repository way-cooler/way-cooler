//! A tag is similar to a workspace, except a client can be attached
//! to multiple tags at once.

use std::{collections::HashSet, default::Default};

use rlua::{self, FromLua, Integer, Table, UserData, UserDataMethods, Value};

use crate::common::{
    class::{self, Class, ClassBuilder},
    object::{self, Object, ObjectBuilder},
    property::Property,
    signal
};
use crate::objects::client::Client;

pub const TAG_LIST: &'static str = "__tag_list";

#[derive(Clone, Debug)]
pub struct TagState {
    name: Option<String>,
    selected: bool,
    activated: bool
}

pub type Tag<'lua> = Object<'lua, TagState>;

impl Default for TagState {
    fn default() -> Self {
        TagState {
            name: None,
            selected: false,
            activated: false
        }
    }
}

impl<'lua> Tag<'lua> {
    fn new(lua: rlua::Context<'lua>, args: Table<'lua>) -> rlua::Result<Tag<'lua>> {
        let class = class::class_setup(lua, "tag")?;
        Ok(object_setup(lua, Tag::allocate(lua, class)?)?
            .handle_constructor_argument(args)?
            .build())
    }

    pub fn clients(&self) -> rlua::Result<Vec<Client<'lua>>> {
        self.get_associated_data::<Vec<Client>>("__clients")
    }

    pub fn set_clients(&mut self, clients: Vec<Client<'lua>>) -> rlua::Result<()> {
        {
            let prev_clients = self.clients()?.into_iter().collect::<HashSet<_>>();
            let new_clients = clients.iter().cloned().collect::<HashSet<_>>();

            for _client in new_clients.difference(&prev_clients) {
                // emit signal
            }

            for _client in prev_clients.difference(&new_clients) {
                // TODO: emit signal and garbage if not referenced anymore
            }
        };
        self.set_associated_data("__clients", clients)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn client_index(&self, client: &Client<'lua>) -> rlua::Result<Option<usize>> {
        // TODO: remove the chaining of collect and into_iter
        Ok(self.clients()?.iter().position(|c| *c == *client))
    }

    #[allow(dead_code)]
    pub fn tag_client(&mut self, client: Client<'lua>) -> rlua::Result<()> {
        if let Some(_) = self.client_index(&client)? {
            // if it is already part of the clients
            return Ok(());
        }
        let mut clients = self.clients()?;
        clients.push(client);
        self.set_associated_data("__clients", clients)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn untag_client(&mut self, client: Client<'lua>) -> rlua::Result<()> {
        let clients: Vec<_> = self.clients()?.into_iter().filter(|c| *c != client).collect();
        self.set_associated_data("__clients", clients)?;
        Ok(())
    }
}

impl UserData for TagState {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        object::default_add_methods(methods);
    }
}

pub fn init(lua: rlua::Context) -> rlua::Result<Class<TagState>> {
    lua.set_named_registry_value(TAG_LIST, lua.create_table()?)?;
    method_setup(lua, Class::builder(lua, "tag", None)?)?
        .save_class("tag")?
        .build()
}

fn method_setup<'lua>(
    lua: rlua::Context<'lua>,
    builder: ClassBuilder<'lua, TagState>
) -> rlua::Result<ClassBuilder<'lua, TagState>> {
    // TODO Do properly
    use super::dummy;
    builder
        .method("connect_signal".into(), lua.create_function(dummy)?)?
        .method(
            "__call".into(),
            lua.create_function(|lua, args: Table| Tag::new(lua, args))?
        )?
        .property(Property::new(
            "name".into(),
            Some(lua.create_function(set_name)?),
            Some(lua.create_function(get_name)?),
            Some(lua.create_function(set_name)?)
        ))?
        .property(Property::new(
            "selected".into(),
            Some(lua.create_function(set_selected)?),
            Some(lua.create_function(get_selected)?),
            Some(lua.create_function(set_selected)?)
        ))?
        .property(Property::new(
            "activated".into(),
            Some(lua.create_function(set_activated)?),
            Some(lua.create_function(get_activated)?),
            Some(lua.create_function(set_activated)?)
        ))?
        .property(Property::new(
            "clients".into(),
            Some(lua.create_function(get_clients)?),
            Some(lua.create_function(|lua, _: Value| {
                // TODO:
                // - Right now this is a property that
                //    returns a function. Can we get
                //    rid of some of this indirection?
                Ok(Value::Function(lua.create_function(get_clients)?))
            })?),
            None
        ))
}

fn object_setup<'lua>(
    lua: rlua::Context<'lua>,
    builder: ObjectBuilder<'lua, TagState>
) -> rlua::Result<ObjectBuilder<'lua, TagState>> {
    let table = lua.create_table()?;
    table.set("__clients", lua.create_table()?)?; // store clients in lua
    builder.add_to_meta(table)
}

fn set_name<'lua>(
    lua: rlua::Context<'lua>,
    (mut tag, val): (Tag<'lua>, String)
) -> rlua::Result<Value<'lua>> {
    tag.state_mut()?.name = Some(val.clone());
    signal::emit_object_signal(lua, tag, "property::name".into(), ())?;
    Ok(Value::Nil)
}

fn get_name<'lua>(lua: rlua::Context<'lua>, tag: Tag<'lua>) -> rlua::Result<Value<'lua>> {
    match tag.state()?.name {
        None => Ok(Value::Nil),
        Some(ref name) => Ok(Value::String(lua.create_string(&name)?))
    }
}

fn set_selected<'lua>(lua: rlua::Context<'lua>, (mut tag, val): (Tag<'lua>, bool)) -> rlua::Result<()> {
    {
        let mut tag = tag.state_mut()?;
        if tag.selected == val {
            return Ok(());
        }
        tag.selected = val;
    }
    signal::emit_object_signal(lua, tag, "property::selected".into(), ())?;
    Ok(())
}

fn get_selected<'lua>(_: rlua::Context<'lua>, tag: Tag<'lua>) -> rlua::Result<bool> {
    Ok(tag.state()?.selected)
}

fn set_activated<'lua>(
    lua: rlua::Context<'lua>,
    (mut tag, val): (Tag<'lua>, bool)
) -> rlua::Result<Value<'lua>> {
    {
        let mut tag = tag.state_mut()?;
        if tag.activated == val {
            return Ok(Value::Nil);
        }
        tag.activated = val;
    }
    let activated_tags = lua.named_registry_value::<str, Table>(TAG_LIST)?;
    let activated_tags_count = activated_tags.len()?;
    if val {
        let index = activated_tags_count + 1;
        activated_tags.set(index, tag.clone())?;
    } else {
        // Find and remove the tag in/from the list of tags
        {
            let tag_ref = &*tag.state()? as *const _;
            let mut found = false;
            for pair in activated_tags.clone().pairs::<Integer, Tag>() {
                let (key, value) = pair?;
                if tag_ref == &*value.state()? as *const _ {
                    found = true;
                    // Now remove this by shifting everything down...
                    for index in key..activated_tags_count {
                        activated_tags.set(index, activated_tags.get::<_, Value>(index + 1)?)?;
                    }
                    activated_tags.set(activated_tags_count, Value::Nil)?;
                    break;
                }
            }
            assert!(found);
        }
        set_selected(lua, (tag.clone(), false))?;
    }
    signal::emit_object_signal(lua, tag, "property::activated".into(), ())?;
    Ok(Value::Nil)
}

fn get_activated<'lua>(_: rlua::Context<'lua>, tag: Tag<'lua>) -> rlua::Result<Value<'lua>> {
    Ok(Value::Boolean(tag.state()?.activated))
}

fn get_clients<'lua>(
    lua: rlua::Context<'lua>,
    (mut tag, val): (Tag<'lua>, Value<'lua>)
) -> rlua::Result<Vec<Client<'lua>>> {
    if let Value::Table(_) = val {
        tag.set_clients(Vec::from_lua(val, lua)?)?;
    };
    tag.clients()
}

#[cfg(test)]
mod test {
    use super::super::{
        client::{self, Client},
        tag::{self, Tag}
    };
    use rlua::{self, Lua};

    #[test]
    fn tag_name_empty() -> rlua::Result<()> {
        let lua = Lua::new();
        lua.context(|ctx| {
            tag::init(ctx)?;
            ctx.load(
                r#"
assert(type(tag{}.name) == "nil")
                "#
            )
            .eval()
        })
    }

    #[test]
    fn tag_name_change() -> rlua::Result<()> {
        let lua = Lua::new();
        lua.context(|ctx| {
            tag::init(ctx)?;
            ctx.load(
                r#"
local t = tag{ name = "a very cool tag" }
assert(t.name == "a very cool tag")
                "#
            )
            .eval()
        })
    }

    #[test]
    fn tag_name_signal() -> rlua::Result<()> {
        let lua = Lua::new();
        lua.context(|ctx| {
            tag::init(ctx)?;
            ctx.load(
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
                "#
            )
            .eval()
        })
    }

    #[test]
    fn tag_selected() -> rlua::Result<()> {
        let lua = Lua::new();
        lua.context(|ctx| {
            tag::init(ctx)?;
            ctx.load(
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
                "#
            )
            .eval()
        })
    }

    #[test]
    fn tag_activated() -> rlua::Result<()> {
        let lua = Lua::new();
        lua.context(|ctx| {
            tag::init(ctx)?;
            ctx.load(
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
                "#
            )
            .eval()
        })
    }

    #[test]
    fn tag_activated_selected() -> rlua::Result<()> {
        let lua = Lua::new();
        lua.context(|ctx| {
            tag::init(ctx)?;
            ctx.load(
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
                "#
            )
            .eval()
        })
    }

    #[test]
    fn tag_client() -> rlua::Result<()> {
        let lua = Lua::new();
        lua.context(|ctx| {
            tag::init(ctx)?;
            ctx.load(
                r#"
local t = tag{}
assert(#t:clients() == 0, "Cannot get the clients")
                "#
            )
            .eval()
        })
    }

    #[test]
    fn tag_set_client() -> rlua::Result<()> {
        let lua = Lua::new();
        lua.context(|ctx| {
            tag::init(ctx)?;
            client::init(ctx)?;
            ctx.load(
                r#"
local c = client{}
local t = tag{}
t:clients({ c })
assert(c, "client doesn't exists")
assert(#t:clients() == 1, "Tag doesn't have the clients")
assert(t:clients()[1] == c, "Pass by value, not by reference")
                "#
            )
            .eval()
        })
    }

    #[test]
    fn tag_set_client_double() -> rlua::Result<()> {
        let lua = Lua::new();
        lua.context(|ctx| {
            tag::init(ctx)?;
            client::init(ctx)?;
            ctx.load(
                r#"
local c = client{}
local t = tag{}
t:clients({ c })
t:clients({ c })
assert(c, "client doesn't exists")
assert(#t:clients() == 1, "Tag doesn't have the clients")
assert(t:clients()[1] == c, "Pass by value, not by reference")
                "#
            )
            .eval()
        })
    }

    #[test]
    fn tag_reference() -> rlua::Result<()> {
        let lua = Lua::new();
        lua.context(|ctx| {
            tag::init(ctx)?;
            client::init(ctx)?;
            let globals = ctx.globals();

            let c = Client::new(ctx, ctx.create_table()?)?;
            let mut t = Tag::new(ctx, ctx.create_table()?)?;
            t.tag_client(c.clone())?;
            globals.set("t", t)?;
            globals.set("c", c)?;
            ctx.load(
                r#"
            assert(#t:clients() == 1, "Clients are not tagged")
                "#
            )
            .eval()?;

            let mut t = globals.get::<_, Tag>("t")?;
            let c = globals.get::<_, Client>("c")?;
            t.untag_client(c)?;
            ctx.load(
                r#"
            assert(#t:clients() == 0, "Tags are not passed by reference")
                "#
            )
            .eval()
        })
    }

    #[test]
    fn tag_share_client() -> rlua::Result<()> {
        let lua = Lua::new();
        lua.context(|ctx| {
            tag::init(ctx)?;
            client::init(ctx)?;
            ctx.load(
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
                "#
            )
            .eval()
        })
    }

    #[test]
    fn tag_new_client() -> rlua::Result<()> {
        let lua = Lua::new();
        lua.context(|ctx| {
            tag::init(ctx)?;
            client::init(ctx)?;
            ctx.load(
                r#"
local c = client{}
local t = tag{ clients = { c } }
assert(c, "client doesn't exists")
assert(#t:clients() == 1, "Tag doesn't have the clients")
assert(t:clients()[1] == c, "Pass by value, not by reference")
                "#
            )
            .eval()
        })
    }

    #[test]
    fn tag_client_reference() -> rlua::Result<()> {
        let lua = Lua::new();
        lua.context(|ctx| {
            tag::init(ctx)?;
            client::init(ctx)?;
            let globals = ctx.globals();

            let c = Client::new(ctx, ctx.create_table()?)?;
            let mut t = Tag::new(ctx, ctx.create_table()?)?;
            t.tag_client(c.clone())?;
            globals.set("t", t)?;
            globals.set("c", c)?;
            ctx.load(
                r#"
            assert(#t:clients() == 1, "Clients are not tagged")
                "#
            )
            .eval()?;

            let mut c = globals.get::<_, Client>("c")?;
            c.state_mut()?.dummy = 1;
            ctx.load(
                r#"
            assert(t:clients()[1] == c, "Tags are not passed by reference")
            "#
            )
            .eval()?;

            let mut c = globals.get::<_, Client>("c")?;
            assert!(c.state_mut()?.dummy == 1);
            Ok(())
        })
    }

    #[test]
    fn tag_tag_client() -> rlua::Result<()> {
        let lua = Lua::new();
        lua.context(|ctx| {
            let globals = ctx.globals();
            tag::init(ctx)?;
            client::init(ctx)?;
            let c = Client::new(ctx, ctx.create_table()?)?;
            let mut t = Tag::new(ctx, ctx.create_table()?)?;
            t.tag_client(c.clone())?;
            globals.set("t", t)?;
            globals.set("c", c)?;
            ctx.load(
                r#"
assert(c, "client doesn't exists")
assert(#t:clients() == 1, "Tag doesn't have the clients")
assert(t:clients()[1] == c, "Pass by value, not by reference")
                "#
            )
            .eval()
        })
    }

    #[test]
    fn tag_untag_client() -> rlua::Result<()> {
        let lua = Lua::new();
        lua.context(|ctx| {
            let globals = ctx.globals();
            tag::init(ctx)?;
            client::init(ctx)?;
            let c = Client::new(ctx, ctx.create_table()?)?;
            let mut t = Tag::new(ctx, ctx.create_table()?)?;
            t.tag_client(c.clone())?;
            t.untag_client(c.clone())?;
            globals.set("t", t)?;
            globals.set("c", c)?;
            ctx.load(
                r#"
assert(c, "client doesn't exists")
assert(#t:clients() == 0, "Clients are not untagged")
            "#
            )
            .eval()
        })
    }
}
