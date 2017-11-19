//! TODO Fill in

use rustwlc::{Geometry, WlcOutput};
use std::fmt::{self, Display, Formatter};
use std::default::Default;
use std::rc::Rc;
use rlua::{self, Table, Lua, UserData, ToLua, Value};
use super::object::{Object, Objectable, ObjectBuilder};
use super::property::Property;
use super::class::{self, Class, ClassBuilder};

pub const SCREENS_HANDLE: &'static str = "__screens";

#[derive(Clone, Debug)]
pub struct Output {
    pub name: String,
    pub mm_width: u32,
    pub mm_height: u32,
    // TODO The XID array?
}

impl From<WlcOutput> for Output {
    fn from(output: WlcOutput) -> Output {
        let resolution = output.get_resolution().unwrap();
        Output {
            name: output.get_name(),
            mm_width: resolution.w,
            mm_height: resolution.h
        }
    }
}

#[derive(Clone, Debug)]
pub struct ScreenState {
    // Is this screen still valid and may be used
    pub valid: bool,
    // Screen geometry
    pub geometry: Geometry,
    // Screen workarea
    pub workarea: Geometry,
    // The screen outputs information
    pub outputs: Vec<Output>,
    // Some XID indetifying this screen
    pub xid: u32
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

    fn init_screens(&mut self, outputs: Vec<Output>) -> rlua::Result<()> {
        let mut state = self.state()?;
        state.outputs = outputs;
        self.set_state(state)
    }
}

pub fn init(lua: &Lua) -> rlua::Result<Class> {
    let res = method_setup(lua, Class::builder(lua, Some(Rc::new(Screen::new)), None, None)?)?
        .save_class("screen")?
        .build()?;
    let mut screens: Vec<Screen> = vec![];
    for output in WlcOutput::list() {
        let mut screen = Screen::cast(Screen::new(lua)?)?;
        screen.init_screens(vec![output.into()])?;
        // TODO Move to Screen impl like the others
        screens.push(screen);
    }
    lua.globals().set(SCREENS_HANDLE, screens.to_lua(lua)?)?;
    Ok(res)
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
    let screens: Vec<Screen> = lua.globals().get::<_, Vec<Table>>(SCREENS_HANDLE)?
        .into_iter().map(|t| Screen::cast(t.into()).unwrap())
        .collect();
    match index {
        Value::String(ref string) => {
            let string = string.to_str()?;
            if string == "primary" {
                // TODO Emit primary changed signal
                if screens.len() > 0 {
                    return screens[0].get_table().clone().to_lua(lua)
                }
            }
            for screen in screens.iter() {
                let screen_state = screen.state()?;
                for output in &screen_state.outputs {
                    if output.name.as_str() == string {
                        return screen.get_table().clone().to_lua(lua)
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
            return screens[screen_index as usize].get_table().clone().to_lua(lua).clone()
        },
        Value::Table(ref table) =>{
            // If this is a screen, just return it
            if let Ok(screen) = Screen::cast(table.clone().into()) {
                return screen.to_lua(lua)
            }
        },
        // TODO This checke user data like in luaA_toudata in awesome
        _ => {}
    }
    // TODO checkudata
    let meta = obj_table.get_metatable().unwrap();
    meta.get(index)
}
