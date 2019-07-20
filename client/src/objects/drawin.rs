//! A wrapper around a drawable. This controls all the other state about
//! the surface, such as the cursor used or where it on the screen.

// NOTE need to store the drawable in lua, because it's a reference to a
// drawable a lua object

use {
    cairo::ImageSurface,
    rlua::{
        self, prelude::LuaInteger, Table, ToLua, UserData, UserDataMethods,
        Value
    }
};

use crate::{
    area::{Area, Origin, Size},
    common::{
        class::{self, Class, ClassBuilder},
        object::{self, Object, ObjectBuilder},
        property::Property
    },
    objects::drawable::Drawable
};

pub const DRAWINS_HANDLE: &'static str = "__drawins";

#[derive(Debug, Default)]
pub struct DrawinState {
    // NOTE This has a drawable, but it is stored in lua.
    ontop: bool,
    visible: bool,
    cursor: String,
    geometry: Area,
    geometry_dirty: bool,
    surface: Option<ImageSurface>
}

pub type Drawin<'lua> = Object<'lua, DrawinState>;

impl UserData for DrawinState {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        object::default_add_methods(methods);
    }
}

impl<'lua> Drawin<'lua> {
    fn new(
        lua: rlua::Context<'lua>,
        args: Table<'lua>
    ) -> rlua::Result<Drawin<'lua>> {
        let class = class::class_setup(lua, "drawin")?;
        let mut drawins =
            lua.named_registry_value::<str, Vec<Drawin>>(DRAWINS_HANDLE)?;
        let drawin = object_setup(lua, Drawin::allocate(lua, class)?)?
            .handle_constructor_argument(args)?
            .build();
        drawins.push(drawin.clone());
        lua.set_named_registry_value(DRAWINS_HANDLE, drawins.to_lua(lua)?)?;
        Ok(drawin)
    }

    /// Get the drawable associated with this drawin.
    ///
    /// It has the surface that is needed to render to the screen.
    #[allow(dead_code)]
    pub fn drawable(&mut self) -> rlua::Result<Drawable<'lua>> {
        self.get_associated_data::<Drawable>("drawable")
    }

    fn update_drawing(&mut self, lua: rlua::Context<'lua>) -> rlua::Result<()> {
        let mut drawable: Drawable = self.get_associated_data("drawable")?;
        {
            let mut state = self.state_mut()?;
            if state.geometry_dirty {
                drawable.set_geometry(lua, state.geometry)?;
                state.geometry_dirty = false;
            }
        }
        self.set_associated_data("drawable", drawable)?;
        Ok(())
    }

    pub fn get_visible(&mut self) -> rlua::Result<bool> {
        let drawin = self.state()?;
        Ok(drawin.visible)
    }

    fn set_visible(
        &mut self,
        lua: rlua::Context<'lua>,
        val: bool
    ) -> rlua::Result<()> {
        {
            let mut drawin = self.state_mut()?;
            drawin.visible = val;
        }
        if val {
            self.map(lua)
        } else {
            self.unmap()
        }
    }

    fn map(&mut self, lua: rlua::Context<'lua>) -> rlua::Result<()> {
        // TODO other things
        self.update_drawing(lua)?;
        Ok(())
    }

    fn unmap(&mut self) -> rlua::Result<()> {
        // TODO?
        Ok(())
    }

    pub fn get_geometry(&self) -> rlua::Result<Area> {
        Ok(self.state()?.geometry)
    }

    fn resize(
        &mut self,
        lua: rlua::Context<'lua>,
        geometry: Area
    ) -> rlua::Result<()> {
        {
            let mut state = self.state_mut()?;
            let old_geometry = state.geometry;
            state.geometry = geometry;
            {
                let Size {
                    ref mut width,
                    ref mut height
                } = state.geometry.size;
                if *width == 0 {
                    *width = old_geometry.size.width;
                }
                if *height == 0 {
                    *height = old_geometry.size.height
                }
            }
            state.geometry_dirty = true;
            // TODO emit signals
            // TODO update screen workareas like in awesome? Might not be necessary
        }
        self.update_drawing(lua)
    }
}

pub fn init(lua: rlua::Context) -> rlua::Result<Class<DrawinState>> {
    let drawins: Vec<Drawin> = Vec::new();
    lua.set_named_registry_value(DRAWINS_HANDLE, drawins.to_lua(lua)?)?;
    property_setup(
        lua,
        method_setup(lua, Class::builder(lua, "drawin", None)?)?
    )?
    .save_class("drawin")?
    .build()
}

fn method_setup<'lua>(
    lua: rlua::Context<'lua>,
    builder: ClassBuilder<'lua, DrawinState>
) -> rlua::Result<ClassBuilder<'lua, DrawinState>> {
    // TODO Do properly
    builder
           // TODO This should be adding properties, e.g like luaA_class_new
           .method("__call".into(), lua.create_function(|lua, args: Table| Drawin::new(lua, args))?)
}

fn property_setup<'lua>(
    lua: rlua::Context<'lua>,
    builder: ClassBuilder<'lua, DrawinState>
) -> rlua::Result<ClassBuilder<'lua, DrawinState>> {
    builder
        .property(Property::new(
            "x".into(),
            Some(lua.create_function(set_x)?),
            Some(lua.create_function(get_x)?),
            Some(lua.create_function(set_x)?)
        ))?
        .property(Property::new(
            "y".into(),
            Some(lua.create_function(set_y)?),
            Some(lua.create_function(get_y)?),
            Some(lua.create_function(set_y)?)
        ))?
        .property(Property::new(
            "width".into(),
            Some(lua.create_function(set_width)?),
            Some(lua.create_function(get_width)?),
            Some(lua.create_function(set_width)?)
        ))?
        .property(Property::new(
            "height".into(),
            Some(lua.create_function(set_height)?),
            Some(lua.create_function(get_height)?),
            Some(lua.create_function(set_height)?)
        ))?
        .property(Property::new(
            "visible".into(),
            Some(lua.create_function(set_visible)?),
            Some(lua.create_function(get_visible)?),
            Some(lua.create_function(set_visible)?)
        ))
}

fn object_setup<'lua>(
    lua: rlua::Context<'lua>,
    builder: ObjectBuilder<'lua, DrawinState>
) -> rlua::Result<ObjectBuilder<'lua, DrawinState>> {
    let drawable_table = Drawable::new(lua)?.to_lua(lua)?;

    let table = lua.create_table()?;
    table.set("drawable", drawable_table)?;
    table.set("geometry", lua.create_function(drawin_geometry)?)?;
    table.set("struts", lua.create_function(drawin_struts)?)?;
    table.set("buttons", lua.create_function(super::dummy)?)?;
    builder.add_to_meta(table)
}

fn set_visible<'lua>(
    lua: rlua::Context<'lua>,
    (mut drawin, visible): (Drawin<'lua>, bool)
) -> rlua::Result<()> {
    drawin.set_visible(lua, visible)?;
    Object::emit_signal(lua, &drawin, "property::visible".into(), Value::Nil)
}

fn get_visible<'lua>(
    _: rlua::Context<'lua>,
    mut drawin: Drawin<'lua>
) -> rlua::Result<bool> {
    drawin.get_visible()
    // TODO signal
}

fn drawin_geometry<'lua>(
    lua: rlua::Context<'lua>,
    (mut drawin, geometry): (Drawin<'lua>, Option<Table<'lua>>)
) -> rlua::Result<Table<'lua>> {
    if let Some(geometry) = geometry {
        let width = geometry.get::<_, u32>("width")?;
        let height = geometry.get::<_, u32>("height")?;
        let x = geometry.get::<_, i32>("x")?;
        let y = geometry.get::<_, i32>("y")?;
        if width > 0 && height > 0 {
            let geo = Area {
                origin: Origin { x, y },
                size: Size { width, height }
            };
            drawin.resize(lua, geo)?;
        }
    }
    let new_geo = drawin.get_geometry()?;
    let Size { width, height } = new_geo.size;
    let Origin { x, y } = new_geo.origin;
    let res = lua.create_table()?;
    res.set("x", x)?;
    res.set("y", y)?;
    res.set("height", height)?;
    res.set("width", width)?;
    Ok(res)
}

fn get_x<'lua>(
    _: rlua::Context<'lua>,
    drawin: Drawin<'lua>
) -> rlua::Result<LuaInteger> {
    let Origin { x, .. } = drawin.get_geometry()?.origin;
    Ok(x as LuaInteger)
}

fn set_x<'lua>(
    lua: rlua::Context<'lua>,
    (mut drawin, x): (Drawin<'lua>, LuaInteger)
) -> rlua::Result<()> {
    let mut geo = drawin.get_geometry()?;
    geo.origin.x = x as i32;
    drawin.resize(lua, geo)?;
    Ok(())
}

fn get_y<'lua>(
    _: rlua::Context<'lua>,
    drawin: Drawin<'lua>
) -> rlua::Result<LuaInteger> {
    let Origin { y, .. } = drawin.get_geometry()?.origin;
    Ok(y as LuaInteger)
}

fn set_y<'lua>(
    lua: rlua::Context<'lua>,
    (mut drawin, y): (Drawin<'lua>, LuaInteger)
) -> rlua::Result<()> {
    let mut geo = drawin.get_geometry()?;
    geo.origin.y = y as i32;
    drawin.resize(lua, geo)?;
    Ok(())
}

fn get_width<'lua>(
    _: rlua::Context<'lua>,
    drawin: Drawin<'lua>
) -> rlua::Result<LuaInteger> {
    let Size { width, .. } = drawin.get_geometry()?.size;
    Ok(width as LuaInteger)
}

fn set_width<'lua>(
    lua: rlua::Context<'lua>,
    (mut drawin, width): (Drawin<'lua>, LuaInteger)
) -> rlua::Result<()> {
    let mut geo = drawin.get_geometry()?;
    if width > 0 {
        geo.size.width = width as u32;
        drawin.resize(lua, geo)?;
    }
    Ok(())
}

fn get_height<'lua>(
    _lua: rlua::Context<'lua>,
    drawin: Drawin<'lua>
) -> rlua::Result<LuaInteger> {
    let Size { height, .. } = drawin.get_geometry()?.size;
    Ok(height as LuaInteger)
}

fn set_height<'lua>(
    lua: rlua::Context<'lua>,
    (mut drawin, height): (Drawin<'lua>, LuaInteger)
) -> rlua::Result<()> {
    let mut geo = drawin.get_geometry()?;
    if height > 0 {
        geo.size.height = height as u32;
        drawin.resize(lua, geo)?;
    }
    Ok(())
}

fn drawin_struts<'lua>(
    lua: rlua::Context<'lua>,
    _drawin: Drawin<'lua>
) -> rlua::Result<Table<'lua>> {
    // TODO: Implement this properly. Struts means this drawin reserves some space
    // on the screen that it is visible on, shrinking the workarea in the
    // specified directions.
    let res = lua.create_table()?;
    res.set("left", 0)?;
    res.set("right", 0)?;
    res.set("top", 0)?;
    res.set("bottom", 0)?;
    Ok(res)
}
