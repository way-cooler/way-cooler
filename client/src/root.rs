//! API for root resources, such as wallpapers and keybindings.
//! Awesome's equivalent of globalconf's properties are accessible via registry keys

use cairo_sys::cairo_pattern_t;
use rlua::{self, LightUserData, Table, ToLua, Value};

use crate::objects::tag;

/// Handle to the list of global key bindings
pub const ROOT_KEYS_HANDLE: &'static str = "__ROOT_KEYS";

pub fn init(lua: rlua::Context) -> rlua::Result<()> {
    // TODO Do properly
    use crate::objects::dummy;

    let root = lua.create_table()?;
    root.set("connect_signal", lua.create_function(dummy)?)?;
    root.set("buttons", lua.create_function(dummy)?)?;
    root.set("wallpaper", lua.create_function(wallpaper)?)?;
    root.set("tags", lua.create_function(tags)?)?;
    root.set("keys", lua.create_function(root_keys)?)?;
    root.set("size", lua.create_function(dummy_double)?)?;
    root.set("size_mm", lua.create_function(dummy_double)?)?;
    root.set("cursor", lua.create_function(dummy)?)?;

    lua.globals().set("root", root)
}

fn dummy_double<'lua>(_: rlua::Context<'lua>, _: Value) -> rlua::Result<(i32, i32)> {
    Ok((0, 0))
}

/// Gets the wallpaper as a cairo surface or set it as a cairo pattern
fn wallpaper<'lua>(lua: rlua::Context<'lua>, pattern: Option<LightUserData>) -> rlua::Result<Value<'lua>> {
    // TODO FIXME Implement for realz
    if let Some(pattern) = pattern {
        // TODO Wrap before giving it to set_wallpaper
        let pattern = pattern.0 as *mut cairo_pattern_t;
        return set_wallpaper(lua, pattern)?.to_lua(lua);
    }
    // TODO Look it up in global conf (e.g probably super secret lua value)
    return Ok(Value::Nil);
}

fn set_wallpaper<'lua>(_: rlua::Context<'lua>, _pattern: *mut cairo_pattern_t) -> rlua::Result<bool> {
    warn!("Fake setting the wallpaper");
    Ok(true)
}

fn tags<'lua>(lua: rlua::Context<'lua>, _: ()) -> rlua::Result<Table<'lua>> {
    let table = lua.create_table()?;
    let activated_tags = lua.named_registry_value::<str, Table>(tag::TAG_LIST)?;
    for pair in activated_tags.pairs::<Value, Value>() {
        let (key, value) = pair?;
        table.set(key, value)?;
    }
    Ok(table)
}

/// Get or set global key bindings.
///
/// These bindings will be available when you press keys on the root window.
fn root_keys<'lua>(lua: rlua::Context<'lua>, key_array: Value<'lua>) -> rlua::Result<Value<'lua>> {
    match key_array {
        // Set the global keys
        Value::Table(key_array) => {
            let copy = lua.create_table()?;
            // NOTE We make a deep clone so they can't modify references.
            for entry in key_array.clone().pairs() {
                let (key, value) = entry?;
                copy.set::<Value, Value>(key, value)?;
            }
            lua.set_named_registry_value(ROOT_KEYS_HANDLE, copy)?;
            Ok(Value::Table(key_array))
        },
        // Get the global keys
        Value::Nil => {
            let res = lua.create_table()?;
            for entry in lua
                .named_registry_value::<str, Table>(ROOT_KEYS_HANDLE)
                .or(lua.create_table())?
                .pairs()
            {
                let (key, value) = entry?;
                res.set::<Value, Value>(key, value)?;
            }
            Ok(Value::Table(res))
        },
        v => Err(rlua::Error::RuntimeError(format!(
            "Expected nil or array \
             of keys, got {:?}",
            v
        )))
    }
}

#[cfg(test)]
mod test {
    use crate::objects::{key, tag};
    use crate::root;
    use rlua::Lua;

    #[test]
    fn tags_print() {
        let lua = Lua::new();
        lua.context(|ctx| {
            tag::init(ctx).unwrap();
            root::init(ctx).unwrap();
            ctx.load(
                r#"
local first, second = tag{}, tag{}
assert(tostring(first) ~= tostring(second))
                "#
            )
            .eval()
            .unwrap()
        })
    }

    #[test]
    fn tags_none() {
        let lua = Lua::new();
        lua.context(|ctx| {
            tag::init(ctx).unwrap();
            root::init(ctx).unwrap();
            ctx.load(
                r#"
local t = root.tags()
assert(type(t) == "table")
assert(type(next(t)) == "nil")
                "#
            )
            .eval()
            .unwrap()
        })
    }

    #[test]
    fn tags_does_not_copy() {
        let lua = Lua::new();
        lua.context(|ctx| {
            tag::init(ctx).unwrap();
            root::init(ctx).unwrap();
            ctx.load(
                r#"
local t = tag{ activated = true }
local t2 = root.tags()[1]
assert(t == t2)
t2.name = "Foo"
assert(t.name == "Foo")
                "#
            )
            .eval()
            .unwrap()
        })
    }

    #[test]
    fn tags_some() {
        let lua = Lua::new();
        lua.context(|ctx| {
            tag::init(ctx).unwrap();
            root::init(ctx).unwrap();
            ctx.load(
                r#"
local first = tag{ activated = true }
local second = tag{ activated = true }
local t = root.tags()
assert(t[1] == first)
assert(t[2] == second)
                "#
            )
            .eval()
            .unwrap()
        })
    }

    #[test]
    fn tags_removal() {
        let lua = Lua::new();
        lua.context(|ctx| {
            tag::init(ctx).unwrap();
            root::init(ctx).unwrap();
            ctx.load(
                r#"
local first = tag{ activated = true }
local second = tag{ activated = true }
first.activated = false
local t = root.tags()
assert(t[1] == second)
assert(type(t[2]) == "nil")
                "#
            )
            .eval()
            .unwrap()
        })
    }

    #[test]
    fn keys() {
        let lua = Lua::new();
        lua.context(|ctx| {
            key::init(ctx).unwrap();
            root::init(ctx).unwrap();
            ctx.load(
                r#"
assert(next(root.keys()) == nil)

local first = key{}
local second = key{}
local keys = { first, second }

local res = root.keys(keys)
assert(res[1] == first)
assert(res[2] == second)
assert(res[3] == nil)

keys[3] = key{}
local res = root.keys()
assert(res[1] == first)
assert(res[2] == second)
assert(res[3] == nil)
                "#
            )
            .eval()
            .unwrap()
        })
    }
}
