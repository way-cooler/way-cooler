//! TODO Fill in

use rustwlc::{Geometry, Point, Size, WlcOutput};
use std::fmt::{self, Display, Formatter};
use std::default::Default;
use std::rc::Rc;
use rlua::{self, Table, Lua, UserData, ToLua, Value, AnyUserData, UserDataMethods,
           MetaMethod};
use super::object::{self, Object, Objectable};
use super::property::Property;
use super::class::{self, Class, ClassBuilder};

pub const SCREENS_HANDLE: &'static str = "__screens";

#[derive(Clone, Debug, PartialEq, Eq)]
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
pub struct Screen<'lua>(Object<'lua>);

#[derive(Clone, Debug, PartialEq, Eq)]
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

impl UserData for ScreenState {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        methods.add_meta_function(MetaMethod::Index, index);
        methods.add_meta_function(MetaMethod::NewIndex, object::default_newindex);
    }
}

impl <'lua> Screen<'lua> {
    fn new(lua: &Lua) -> rlua::Result<Object> {
        let class = class::class_setup(lua, "screen")?;
        Ok(Screen::allocate(lua, class)?.build())
    }

    fn init_screens(&mut self, output: WlcOutput, outputs: Vec<Output>) -> rlua::Result<()> {
        let mut state = self.state()?;
        let resolution = output.get_resolution().unwrap();
        state.outputs = outputs;
        state.geometry.size = resolution;
        state.workarea.size = resolution;
        self.set_state(state)
    }

    fn get_geometry(&self, lua: &'lua Lua) -> rlua::Result<Table<'lua>> {
        let state = self.state()?;
        let Point { x, y } = state.geometry.origin;
        let Size { w, h } = state.geometry.size;
        let table = lua.create_table()?;
        table.set("x", x)?;
        table.set("y", y)?;
        table.set("width", w)?;
        table.set("height", h)?;
        Ok(table)
    }

    fn get_workarea(&self, lua: &'lua Lua) -> rlua::Result<Table<'lua>> {
        let state = self.state()?;
        let Point { x, y } = state.workarea.origin;
        let Size { w, h } = state.workarea.size;
        let table = lua.create_table()?;
        table.set("x", x)?;
        table.set("y", y)?;
        table.set("width", w)?;
        table.set("height", h)?;
        Ok(table)
    }
}

pub fn init(lua: &Lua) -> rlua::Result<Class> {
    let builder = Class::builder(lua, "screen", Some(Rc::new(Screen::new)), None, None)?;
    let res = property_setup(lua, method_setup(lua, builder)?)?
        .save_class("screen")?
        .build()?;
    let mut screens: Vec<Screen> = vec![];
    for output in WlcOutput::list() {
        let mut screen = Screen::cast(Screen::new(lua)?)?;
        screen.init_screens(output, vec![output.into()])?;
        // TODO Move to Screen impl like the others
        screens.push(screen);
    }
    lua.globals().set(SCREENS_HANDLE, screens.to_lua(lua)?)?;
    Ok(res)
}

fn method_setup<'lua>(lua: &'lua Lua, builder: ClassBuilder<'lua>)
                      -> rlua::Result<ClassBuilder<'lua>> {
    // TODO Do properly
    use super::dummy;
    builder.method("connect_signal".into(), lua.create_function(dummy)?)?
           .method("__call".into(), lua.create_function(iterate_over_screens)?)?
           .method("__index".into(), lua.create_function(index)?)
}

fn property_setup<'lua>(lua: &'lua Lua, builder: ClassBuilder<'lua>)
                        -> rlua::Result<ClassBuilder<'lua>> {
    builder
        .property(Property::new("geometry".into(),
                                None,
                                Some(lua.create_function(get_geometry)?),
                                None))?
        .property(Property::new("workarea".into(),
                                None,
                                Some(lua.create_function(get_workarea)?),
                                None))
}

fn get_geometry<'lua>(lua: &'lua Lua, object: AnyUserData<'lua>) -> rlua::Result<Table<'lua>> {
    let screen = Screen::cast(object.into())?;
    screen.get_geometry(lua)
}

fn get_workarea<'lua>(lua: &'lua Lua, object: AnyUserData<'lua>) -> rlua::Result<Table<'lua>> {
    let screen = Screen::cast(object.into())?;
    screen.get_workarea(lua)
}

/// Ok this requires some explanation...
/// Lua gives us the previous value in the loop, with the first one being nil
/// since there was nothing there before.
///
/// To ensure we loop over everything, we take the index of that value in our list,
/// increment it by 1 (starting at 0 if it's the start) and then once it falls outside
/// the bounds it will stop by returning nil.
fn iterate_over_screens<'lua>(lua: &'lua Lua,
                              (_, prev): (Value<'lua>, Value<'lua>))
                              -> rlua::Result<Value<'lua>> {
    let screens: Vec<Screen> = lua.globals().get::<_, Vec<AnyUserData>>(SCREENS_HANDLE)?
        .into_iter().map(|obj| Screen::cast(obj.into()).unwrap())
        .collect();
    let index = match prev {
        Value::Nil => 0,
        Value::UserData(ref object) => {
            if let Ok(screen) = Screen::cast(object.clone().into()) {
                screens.iter().position(|t| t.state().unwrap() == screen.state().unwrap())
                    .unwrap_or(screens.len()) + 1
            } else {
                panic!("Unexpected non-screen table in loop");
            }
        }
        _ => panic!("Unexpected non-screen or nil value in screens loop")
    };
    if index < screens.len() {
        warn!("here");
        screens[index].clone().to_lua(lua)
    } else {
        Ok(Value::Nil)
    }

}

fn index<'lua>(lua: &'lua Lua,
               (data, index): (AnyUserData<'lua>, Value<'lua>))
               -> rlua::Result<Value<'lua>> {
    let obj: Object = data.clone().into();
    let screens: Vec<Screen> = lua.globals().get::<_, Vec<AnyUserData>>(SCREENS_HANDLE)?
        .into_iter().map(|obj| Screen::cast(obj.into()).unwrap())
        .collect();
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
                let screen_state = screen.state()?;
                for output in &screen_state.outputs {
                    if output.name.as_str() == string {
                        return screen.clone().to_lua(lua)
                    }
                }
            }
        },
        // TODO Might need to do Number instead
        Value::Integer(screen_index) => {
            if screen_index < 1 || screen_index as usize > screens.len() {
                return Err(rlua::Error::RuntimeError(
                    format!("invalid screen number: {} (of {} existing)",
                            screen_index, screens.len())))
            }
            return screens[(screen_index - 1) as usize].clone().to_lua(lua)
        },
        Value::UserData(ref obj) => {
            // If this is a screen, just return it
            if let Ok(screen) = Screen::cast(obj.clone().into()) {
                return screen.to_lua(lua)
            }
        },
        // TODO This checke user data like in luaA_toudata in awesome
        _ => {}
    }
    // TODO checkudata
    let table = obj.table()?;
    let meta = table.get_metatable().expect("screen had no metatable");
    match meta.get(index.clone()) {
        Err(_) | Ok(Value::Nil) => super::object::default_index(lua, (data, index)),
        Ok(value) => Ok(value)
    }
}
