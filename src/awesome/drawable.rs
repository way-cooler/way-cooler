//! AwesomeWM drawable interface, for all things that are drawable

use std::fmt::{self, Display, Formatter};
use rustwlc::Geometry;
use cairo::ImageSurface;
use rlua::{self, Lua, UserData, AnyUserData, UserDataMethods, MetaMethod};
use ::render::Renderable;
use super::{object, class, Signal};

pub type DrawableRefreshCallback<T: 'static> = fn (&mut T);

pub struct Drawable<T: 'static> {
    surface: Option<ImageSurface>,
    geometry: Geometry,
    refreshed: bool,
    refresh_callback: DrawableRefreshCallback<T>,
    data: T
}

impl <T> Display for Drawable<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Display: {:p}", self)
    }
}

impl <T: 'static> UserData for Drawable<T> {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        object::add_meta_methods(methods);
        class::add_meta_methods(methods);
        methods.add_method_mut("refresh", Drawable::refresh);
        methods.add_method_mut("geometry", Drawable::geometry);

        // for properties, original awesome has a good solution.
        // store the properties in essentially a { "prop_name": {cb_new, cb_index, cb_newindex}}
        // then store the propreties in a global state somewhere.
        // THAT way you can dynamically addd properties (sweet!) and less obnonxious code to write
        // (double sweet!) becaues we can just iterate through that on the index and newindex methods
        // instead of writing out the checks manually.
        //
        // only downside is then we have a global variable that needs to be locked behind a mutex,
        // which is annoying but whatever.
    }
}

/// Initializes the global Drawable class. This allows Lua to allocate new
/// drawables using that base class as a template.
pub fn init() -> rlua::Result<()> {
    panic!()
    // TODO Just like drawable_class_setup in AwesomeWM
    // This will be a little different, because the methods and meta methods
    // are already bound by rlua in the `add_methods` of `UserData` impl for `Drawable`.

    // This will just set up properties, and the drawable super class global.
}

impl <T: 'static> Drawable<T> {
    /// Allocator for a new drawable to be created in the Lua registry.
    pub fn allocator(lua: &Lua,
                     refresh_callback: DrawableRefreshCallback<T>,
                     data: T) -> AnyUserData {
        // TODO Emit "new" signal
        let drawable = Drawable {
            surface: None,
            geometry: Geometry::zero(),
            refreshed: false,
            refresh_callback,
            data
        };
        // TODO Set meta table to be the class Drawable
        // This allocates an Drawable _object_
        lua.create_userdata(drawable)
    }

    pub fn unset_surface(&mut self) {
        self.surface.take();
        self.refreshed = false;
    }

    fn refresh(lua: &Lua, this: &mut Drawable<T>, _: ()) -> rlua::Result<()> {
        this.refreshed = true;
        (this.refresh_callback)(&mut this.data);
        Ok(())
    }

    fn geometry<'lua>(lua: &'lua Lua, this: &mut Drawable<T>, _: ())
                -> rlua::Result<rlua::Table<'lua>> {
        let area = lua.create_table();
        area.set("x", this.geometry.origin.x)?;
        area.set("y", this.geometry.origin.y)?;
        area.set("width", this.geometry.size.w)?;
        area.set("height", this.geometry.size.h)?;
        Ok(area)
    }
}

#[cfg(test)]
mod test {
    use rlua::{self, Lua, Value, ToLua};
    use super::*;

    fn noop(input: &mut ()) {}

    #[test]
    fn test_drawable_geometry() {
        let lua = Lua::new();
        let drawable = Drawable::allocator(&lua, noop, ());
        lua.globals().set("drawable", drawable).unwrap();
        lua.eval::<()>(r#"
                       print(drawable)
                       table = drawable:geometry()
                       assert(table.x == 0)
                       assert(table.y == 0)
                       assert(table.width == 0)
                       assert(table.height == 0)
                       "#,
                       None).unwrap();
    }

    #[test]
    fn test_drawable_refresh() {
        static mut num_refreshed: u32 = 0;
        let lua = Lua::new();
        fn update(input: &mut String) {
            println!("{:?}", input);
            unsafe {
                num_refreshed = 1;
            }
        }
        let drawable = Drawable::allocator(&lua, update, "input".into());
        lua.globals().set("drawable", drawable).unwrap();
        lua.eval::<()>(r#"
                     print(drawable)
                     drawable:refresh()
                "#,
                 None).unwrap();
        unsafe {
            assert_eq!(num_refreshed, 1);
        }
    }
}
