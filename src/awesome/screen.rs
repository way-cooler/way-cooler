//! TODO Fill in

use rustwlc::Geometry;
use std::fmt::{self, Display, Formatter};
use std::default::Default;
use std::rc::Rc;
use std::sync::Mutex;
use rlua::{self, Table, Lua, UserData, ToLua, Value};
use super::object::{Object, Objectable, ObjectBuilder};
use super::property::Property;
use super::class::{self, Class, ClassBuilder};

lazy_static! {
    pub static ref SCREENS: Mutex<Vec<ScreenState>> = Mutex::new(vec![]);
}

#[derive(Clone, Debug)]
pub struct Output {
    pub name: String,
    mm_width: u32,
    mm_height: u32,
    // TODO The XID array?
}

#[derive(Clone, Debug)]
pub struct ScreenState {
    // Is this screen still valid and may be used
    valid: bool,
    // Screen geometry
    geometry: Geometry,
    // Screen workarea
    workarea: Geometry,
    // The screen outputs information
    outputs: Vec<Output>,
    // Some XID indetifying this screen
    xid: u32
}

unsafe impl Send for ScreenState {}
unsafe impl Sync for ScreenState {}

pub struct Screen<'lua>(Table<'lua>);

impl_objectable!(Screen, ScreenState);

impl Display for ScreenState {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Screen: {:p}", self)
    }
}

impl <'lua> ToLua<'lua> for Screen<'lua> {
    fn to_lua(self, lua: &'lua Lua) -> rlua::Result<Value<'lua>> {
        self.0.to_lua(lua)
    }
}

impl UserData for ScreenState {}

impl Default for ScreenState {
    fn default() -> Self {
        ScreenState {
            valid: true,
            geometry: Geometry::zero(),
            workarea: Geometry::zero(),
            outputs: vec![],
            xid: 0
        }
    }
}

impl <'lua> Screen<'lua> {
    fn new(lua: &Lua) -> rlua::Result<Object> {
        let class = class::class_setup(lua, "screen")?;
        Ok(object_setup(lua, Screen::allocate(lua, class)?)?.build())
    }
}

pub fn init(lua: &Lua) -> rlua::Result<Class> {
    method_setup(lua, Class::builder(lua, Some(Rc::new(Screen::new)), None, None)?)?
        .save_class("screen")?
        .build()
}

fn method_setup<'lua>(lua: &'lua Lua, builder: ClassBuilder<'lua>) -> rlua::Result<ClassBuilder<'lua>> {
    // TODO Do properly
    use super::dummy;
    builder.method("connect_signal".into(), lua.create_function(dummy))?
           .method("__index".into(), lua.create_function(index))?
           .method("__call".into(), lua.create_function(dummy))
}

fn object_setup<'lua>(lua: &'lua Lua, builder: ObjectBuilder<'lua>) -> rlua::Result<ObjectBuilder<'lua>> {
    builder
           .property(Property::new("screen".into(),
                                   // TODO Implement
                                   Some(lua.create_function(screen_new)),
                                   Some(lua.create_function(get_visible)),
                                   Some(lua.create_function(set_visible))))
}

fn screen_new<'lua>(_: &'lua Lua, _: ()) -> rlua::Result<()> {
    unimplemented!()
}

fn get_visible<'lua>(_: &'lua Lua, _: ()) -> rlua::Result<()> {
    unimplemented!()
}

fn set_visible<'lua>(_: &'lua Lua, _: ()) -> rlua::Result<()> {
    unimplemented!()
}

fn index<'lua>(lua: &'lua Lua,
               (obj_table, index): (Table<'lua>, Value<'lua>))
               -> rlua::Result<Value<'lua>> {
    let screens = SCREENS.lock().expect("Could not lock global SCREENS");
    match index {
        Value::String(ref string) => {
            let string = string.to_str()?;
            if string == "primary" {
                // TODO Emit primary changed signal
                if screens.len() > 0 {
                    return screens[0].clone().to_lua(lua)
                }
            }
            for screen in screens.iter() {
                for output in &screen.outputs {
                    if output.name.as_str() == string {
                        return screen.clone().to_lua(lua)
                    }
                }
            }
        },
        Value::Integer(screen_index) => {
            if screen_index < 1 || screen_index as usize > screens.len() {
                return Err(rlua::Error::RuntimeError(
                    format!("invalid screen number: {} (of {} existing)",
                            screen_index, screens.len())))
            }
            return screens[screen_index as usize].clone().to_lua(lua)
        }
        _ => {}
    }
    // TODO checkudata
    let meta = obj_table.get_metatable().unwrap();
    meta.get(index)
}
