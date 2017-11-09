//! TODO Fill in

use gdk_pixbuf::Pixbuf;
use glib::translate::ToGlibPtr;
use cairo::{self, ImageSurface};
use std::fmt::{self, Display, Formatter};
use std::default::Default;
use std::rc::Rc;
use rlua::{self, Table, Lua, UserData, ToLua, Value};
use super::object::{Object, Objectable};
use super::class::{self, Class, ClassBuilder};

#[derive(Clone, Debug)]
pub struct AwesomeState {
    // TODO Fill in
    dummy: i32
}

pub struct Awesome<'lua>(Table<'lua>);

impl Default for AwesomeState {
    fn default() -> Self {
        AwesomeState {
            dummy: 0
        }
    }
}

impl <'lua> Awesome<'lua> {
    fn new(lua: &Lua) -> rlua::Result<Object> {
        // TODO FIXME
        let class = class::button_class(lua)?;
        Ok(Awesome::allocate(lua, class)?.build())
    }
}

impl Display for AwesomeState {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Awesome: {:p}", self)
    }
}

impl <'lua> ToLua<'lua> for Awesome<'lua> {
    fn to_lua(self, lua: &'lua Lua) -> rlua::Result<Value<'lua>> {
        self.0.to_lua(lua)
    }
}

impl UserData for AwesomeState {}

pub fn init(lua: &Lua) -> rlua::Result<Class> {
    property_setup(lua, method_setup(lua, Class::builder(lua, Some(Rc::new(Awesome::new)), None, None)?)?)?
        .save_class("awesome")?
        .build()
}

fn method_setup<'lua>(lua: &'lua Lua, builder: ClassBuilder<'lua>) -> rlua::Result<ClassBuilder<'lua>> {
    // TODO Do properly
    use super::dummy;
    builder.method("connect_signal".into(), lua.create_function(dummy))?
           .method("register_xproperty".into(), lua.create_function(register_xproperty))?
           .method("xkb_get_group_names".into(), lua.create_function(xkb_get_group_names))?
           .method("restart".into(), lua.create_function(restart))?
           .method("load_image".into(), lua.create_function(load_image))?
           .method("quit".into(), lua.create_function(dummy))
}

/// Registers a new X property
/// This actually does nothing, since this is Wayland.
fn register_xproperty<'lua>(_: &'lua Lua, _: Value<'lua>) -> rlua::Result<()> {
    warn!("register_xproperty not supported");
    Ok(())
}

/// Get layout short names
fn xkb_get_group_names<'lua>(_: &'lua Lua, _: ()) -> rlua::Result<()> {
    warn!("xkb_get_group_names not supported");
    Ok(())
}

/// Restart Awesome by restarting the Lua thread
fn restart<'lua>(_: &'lua Lua, _: ()) -> rlua::Result<()> {
    use lua::{self, LuaQuery};
    if let Err(err) = lua::send(LuaQuery::Restart) {
        warn!("Could not restart Lua thread {:#?}", err);
    }
    Ok(())
}

/// Load an image from the given path
/// Returns either a cairo surface as light user data, nil and an error message
fn load_image<'lua>(lua: &'lua Lua, file_path: String) -> rlua::Result<Value<'lua>> {
    // TODO Move to render module
    let pixbuf = Pixbuf::new_from_file(file_path.as_str())
        .map_err(|err| rlua::Error::RuntimeError(format!("{}", err)))?;
    let width = pixbuf.get_width();
    let height = pixbuf.get_height();
    let channels = pixbuf.get_n_channels();
    let pix_stride = pixbuf.get_rowstride() as usize;
    // NOTE This is safe because we aren't modifying the bytes, but there's no immutable view
    let pixels = unsafe { pixbuf.get_pixels() };
    let format = if channels == 3 {cairo::Format::Rgb24} else { cairo::Format::ARgb32};
    let mut surface = ImageSurface::create(format, width, height)
        .expect("Could not create image of that size");
    let cairo_stride = surface.get_stride() as usize;
    {
        let mut cairo_data = surface.get_data().unwrap();
        let mut pix_pixels_index = 0;
        let mut cairo_pixels_index = 0;
        for _ in 0..height {
            let mut pix_pixels_index2 = pix_pixels_index;
            let mut cairo_pixels_index2 = cairo_pixels_index;
            for _ in 0..width {
                if channels == 3 {
                    let r = pixels[pix_pixels_index2];
                    let g = pixels[pix_pixels_index2 + 1];
                    let b = pixels[pix_pixels_index2 + 2];
                    cairo_data[cairo_pixels_index2] = b;
                    cairo_data[cairo_pixels_index2 + 1] = g;
                    cairo_data[cairo_pixels_index2 + 2] = r;
                    pix_pixels_index2 += 3;
                    // NOTE Four because of the alpha value we ignore
                    cairo_pixels_index2 += 4;
                } else {
                    // TODO TEST THIS BRANCH
                    let mut r = pixels[pix_pixels_index];
                    let mut g = pixels[pix_pixels_index + 1];
                    let mut b = pixels[pix_pixels_index + 2];
                    let a = pixels[pix_pixels_index + 3];
                    let alpha = a as f64 / 255.0;
                    r *= alpha as u8;
                    g *= alpha as u8;
                    b *= alpha as u8;
                    cairo_data[cairo_pixels_index] = b;
                    cairo_data[cairo_pixels_index + 1] = g;
                    cairo_data[cairo_pixels_index + 2] = r;
                    cairo_data[cairo_pixels_index + 3] = a;
                    pix_pixels_index += 4;
                    cairo_pixels_index += 4;
                }
            }
            pix_pixels_index += pix_stride;
            cairo_pixels_index += cairo_stride;
        }
    }
    surface.get_data().unwrap();
    // UGH, I wanted to do to_glib_full, but that isn't defined apparently
    // So now I have to ignore the lifetime completely and just forget about the surface.
    let surface_ptr = surface.to_glib_none().0;
    ::std::mem::forget(surface);
    rlua::LightUserData(surface_ptr as _).to_lua(lua)
}

fn property_setup<'lua>(lua: &'lua Lua, builder: ClassBuilder<'lua>) -> rlua::Result<ClassBuilder<'lua>> {
    // TODO Do properly
    builder.dummy_property("version".into(), "0".to_lua(lua)?)?
           .dummy_property("themes_path".into(), "/usr/share/awesome/themes".to_lua(lua)?)?
           .dummy_property("conffile".into(), "".to_lua(lua)?)
}

impl_objectable!(Awesome, AwesomeState);
