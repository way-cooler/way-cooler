// NOTE need to store the drawable in lua, because it's a reference to a drawable a lua object


use std::default::Default;
use std::fmt::{self, Display, Formatter};
use std::rc::Rc;
use rustwlc::Geometry;
use rlua::{self, Table, Lua, UserData, ToLua, Value};
use super::drawable::Drawable;

use super::class::{self, Class, ClassBuilder};
use super::object::{Object, Objectable, ObjectBuilder};

#[derive(Clone, Debug)]
pub struct DrawinState {
    // Note that the drawable is stored in Lua.
    // TODO WINDOW_OBJECT_HEADER??
    ontop: bool,
    visible: bool,
    cursor: String,
    geometry: Geometry,
    geometry_dirty: bool
}

#[derive(Clone, Debug)]
pub struct Drawin<'lua>(Table<'lua>);

impl UserData for DrawinState {}

impl Default for DrawinState {
    fn default() -> Self {
        DrawinState {
            ontop: false,
            visible: false,
            cursor: String::default(),
            geometry: Geometry::zero(),
            geometry_dirty: false
        }
    }
}

impl Display for DrawinState {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Drawin: {:p}", self)
    }
}

impl <'lua> Drawin<'lua> {
    fn new(lua: &Lua) -> rlua::Result<Object> {
        // TODO FIXME
        let class = class::class_setup(lua, "drawin")?;
        Ok(object_setup(lua, Drawin::allocate(lua, class)?)?.build())
    }
}

impl <'lua> ToLua<'lua> for Drawin<'lua> {
    fn to_lua(self, lua: &'lua Lua) -> rlua::Result<Value<'lua>> {
        self.0.to_lua(lua)
    }
}

impl_objectable!(Drawin, DrawinState);

pub fn init(lua: &Lua) -> rlua::Result<Class> {
    method_setup(lua, Class::builder(lua, Some(Rc::new(Drawin::new)), None, None)?)?
        .save_class("drawin")?
        .build()
}

fn method_setup<'lua>(lua: &'lua Lua, builder: ClassBuilder<'lua>) -> rlua::Result<ClassBuilder<'lua>> {
    // TODO Do properly
    use super::dummy;
    builder.method("connect_signal".into(), lua.create_function(dummy))?
           .method("__call".into(), lua.create_function(dummy_create))
}
fn object_setup<'lua>(lua: &'lua Lua, builder: ObjectBuilder<'lua>) -> rlua::Result<ObjectBuilder<'lua>> {
    // TODO Do properly
    let table = lua.create_table();
    let drawable_table = Drawable::new(lua)?.to_lua(lua)?;
    table.set("drawable", drawable_table)?;
    builder.add_to_meta(table)
}
fn dummy_create<'lua>(lua: &'lua Lua, _: rlua::Value) -> rlua::Result<Object<'lua>> {
    Drawin::new(lua)
}
