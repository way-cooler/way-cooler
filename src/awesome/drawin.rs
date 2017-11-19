// NOTE need to store the drawable in lua, because it's a reference to a drawable a lua object


use std::default::Default;
use std::fmt::{self, Display, Formatter};
use std::rc::Rc;
use rustwlc::Geometry;
use rlua::{self, Table, Lua, UserData, ToLua, Value};
use super::drawable::Drawable;
use super::property::Property;

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
        let class = class::class_setup(lua, "drawin")?;
        Ok(object_setup(lua, Drawin::allocate(lua, class)?)?.build())
    }

    fn update_drawing(&mut self) -> rlua::Result<()> {
        let state = self.state()?;
        let table = &self.0;
        let mut drawable = Drawable::cast(table.get::<_, Table>("drawable")?.into())?;
        drawable.set_geometry(state.geometry)?;
        table.set::<_, Table>("drawable", drawable.get_table())?;
        Ok(())
    }

    fn get_visible(&mut self) -> rlua::Result<bool> {
        let drawin = self.state()?;
        Ok(drawin.visible)
    }

    fn set_visible(&mut self, val: bool) -> rlua::Result<()> {
        let mut drawin = self.state()?;
        drawin.visible = val;
        self.map()?;
        self.set_state(drawin)
    }

    fn map(&mut self) -> rlua::Result<()> {
        // TODO other things
        self.update_drawing()
    }
}

impl <'lua> ToLua<'lua> for Drawin<'lua> {
    fn to_lua(self, lua: &'lua Lua) -> rlua::Result<Value<'lua>> {
        self.0.to_lua(lua)
    }
}

impl_objectable!(Drawin, DrawinState);

pub fn init(lua: &Lua) -> rlua::Result<Class> {
    property_setup(lua, method_setup(lua, Class::builder(lua, Some(Rc::new(Drawin::new)), None, None)?)?)?
        .save_class("drawin")?
        .build()
}

fn method_setup<'lua>(lua: &'lua Lua, builder: ClassBuilder<'lua>) -> rlua::Result<ClassBuilder<'lua>> {
    // TODO Do properly
    use super::dummy;
    builder.method("connect_signal".into(), lua.create_function(dummy))?
           .method("__call".into(), lua.create_function(|lua, _: Value| Drawin::new(lua)))
}

fn property_setup<'lua>(lua: &'lua Lua, builder: ClassBuilder<'lua>) -> rlua::Result<ClassBuilder<'lua>> {
    builder
        .property(Property::new("visible".into(),
                                Some(lua.create_function(set_visible)),
                                Some(lua.create_function(get_visible)),
                                Some(lua.create_function(set_visible))))
}

fn object_setup<'lua>(lua: &'lua Lua, builder: ObjectBuilder<'lua>) -> rlua::Result<ObjectBuilder<'lua>> {
    // TODO Do properly
    let table = lua.create_table();
    let drawable_table = Drawable::new(lua)?.to_lua(lua)?;
    table.set("drawable", drawable_table)?;
    builder.add_to_meta(table)
}

fn set_visible<'lua>(_: &'lua Lua, (table, visible): (Table<'lua>, bool))
                     -> rlua::Result<()> {
    let mut drawin = Drawin::cast(table.into())?;
    drawin.set_visible(visible)
    // TODO signal
}

fn get_visible<'lua>(_: &'lua Lua, table: Table<'lua>) -> rlua::Result<bool> {
    let mut drawin = Drawin::cast(table.into())?;
    drawin.get_visible()
    // TODO signal
}
