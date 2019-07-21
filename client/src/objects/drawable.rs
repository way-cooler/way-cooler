//! A wrapper around a Cairo image surface.

use std::{default::Default, fs::File, io::Write, slice};

use {
    cairo::{Format, ImageSurface},
    glib::translate::ToGlibPtr,
    rlua::{
        self, Error::RuntimeError, LightUserData, Table, UserData,
        UserDataMethods, Value
    },
    tempfile
};

use crate::{
    area::{Area, Origin, Size},
    common::{
        class::{self, Class},
        object::{self, Object},
        property::Property
    },
    wayland_obj::{self, Layer, LayerSurface}
};

#[derive(Debug)]
pub struct DrawableState {
    temp_file: File,
    wayland_shell: Option<LayerSurface>,
    pub surface: Option<ImageSurface>,
    geo: Area,
    refreshed: bool
}

pub type Drawable<'lua> = Object<'lua, DrawableState>;

impl Default for DrawableState {
    fn default() -> Self {
        let temp_file = tempfile::tempfile()
            .expect("Could not make a temp file for the buffer");
        DrawableState {
            temp_file,
            wayland_shell: None,
            surface: None,
            geo: Area::default(),
            refreshed: false
        }
    }
}

impl<'lua> Drawable<'lua> {
    pub fn new(lua: rlua::Context<'lua>) -> rlua::Result<Drawable> {
        let class = class::class_setup(lua, "drawable")?;
        let builder = Drawable::allocate(lua, class)?;
        let table = lua.create_table()?;
        table.set("geometry", lua.create_function(geometry)?)?;
        table.set("refresh", lua.create_function(refresh)?)?;
        Ok(builder.add_to_meta(table)?.build())
    }

    pub fn get_geometry(&self) -> rlua::Result<Area> {
        let drawable = self.state()?;
        Ok(drawable.geo)
    }

    pub fn get_surface(&self) -> rlua::Result<Value<'lua>> {
        let drawable = self.state()?;
        Ok(match drawable.surface {
            None => Value::Nil,
            Some(ref image) => {
                let stash = image.to_glib_none();
                let ptr = stash.0;
                // NOTE
                // We bump the reference count because now Lua has a reference which
                // it manages via LGI.
                unsafe {
                    ::cairo_sys::cairo_surface_reference(ptr);
                }
                Value::LightUserData(LightUserData(ptr as _))
            }
        })
    }

    /// Sets the geometry, and allocates a new surface.
    pub fn set_geometry(
        &mut self,
        lua: rlua::Context<'lua>,
        geometry: Area
    ) -> rlua::Result<()> {
        let obj_clone = self.clone();
        let mut drawable = self.state_mut()?;
        let size_changed = drawable.geo != geometry;
        let old_geo = drawable.geo;
        drawable.geo = geometry;

        if size_changed {
            drawable.refreshed = false;
            drawable.surface = None;
            drawable.wayland_shell = Some(
                wayland_obj::create_layer_surface(None, Layer::Top, "".into())
                    .expect(
                        "Could not construct an xdg toplevel for a drawable"
                    )
            );
            let size: Size = geometry.size;

            if size.width > 0 && size.height > 0 {
                let temp_file =
                    tempfile::tempfile().expect("Could not make new temp file");
                temp_file
                    .set_len(size.width as u64 * size.height as u64 * 4)
                    .expect("Could not set file length");
                drawable.surface = Some(
                    ImageSurface::create(
                        Format::ARgb32,
                        size.width as i32,
                        size.height as i32
                    )
                    .map_err(|err| {
                        RuntimeError(format!("Could not allocate {:?}", err))
                    })?
                );
                {
                    let shell = drawable.wayland_shell.as_mut().unwrap();
                    shell.set_size(size);
                    shell.set_surface(&temp_file, size).map_err(|_| {
                        RuntimeError(format!(
                            "Could not set surface for drawable"
                        ))
                    })?;
                }
                drawable.temp_file = temp_file;
                Object::emit_signal(
                    lua,
                    &obj_clone,
                    "property::surface".into(),
                    Value::Nil
                )?;
            }
            // emit signals if our geometry has changed
            if old_geo != drawable.geo {
                Object::emit_signal(
                    lua,
                    &obj_clone,
                    "property::geometry".into(),
                    Value::Nil
                )?;
            }
            if old_geo.origin.x != drawable.geo.origin.x {
                Object::emit_signal(
                    lua,
                    &obj_clone,
                    "property::x".into(),
                    Value::Nil
                )?;
            }
            if old_geo.origin.y != drawable.geo.origin.y {
                Object::emit_signal(
                    lua,
                    &obj_clone,
                    "property::y".into(),
                    Value::Nil
                )?;
            }
            if old_geo.size.width != drawable.geo.size.width {
                Object::emit_signal(
                    lua,
                    &obj_clone,
                    "property::width".into(),
                    Value::Nil
                )?;
            }
            if old_geo.size.height != drawable.geo.size.height {
                Object::emit_signal(
                    lua,
                    &obj_clone,
                    "property::height".into(),
                    Value::Nil
                )?;
            }
        }
        Ok(())
    }

    /// Signals that the drawable's surface was updated.
    pub fn refresh(&mut self) -> rlua::Result<()> {
        let mut drawable = self.state_mut()?;
        let drawable = &mut *drawable;

        if let Some(data) = drawable.surface.as_mut().map(get_data) {
            drawable
                .temp_file
                .write(&*data)
                .expect("Could not write data to buffer");
            drawable.temp_file.flush().expect("Could not flush buffer");
            drawable.refreshed = true;
        }
        Ok(())
    }
}

impl UserData for DrawableState {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        object::default_add_methods(methods);
    }
}

pub fn init(lua: rlua::Context) -> rlua::Result<Class<DrawableState>> {
    Class::<DrawableState>::builder(lua, "drawable", None)?
        .method("geometry".into(), lua.create_function(geometry)?)?
        .property(Property::new(
            "surface".into(),
            None,
            Some(lua.create_function(get_surface)?),
            None
        ))?
        .save_class("drawable")?
        .build()
}

fn get_surface<'lua>(
    _: rlua::Context<'lua>,
    drawable: Drawable<'lua>
) -> rlua::Result<Value<'lua>> {
    drawable.get_surface()
}

fn geometry<'lua>(
    lua: rlua::Context<'lua>,
    drawable: Drawable<'lua>
) -> rlua::Result<Table<'lua>> {
    let geometry = drawable.get_geometry()?;
    let Origin { x, y } = geometry.origin;
    let Size { width, height } = geometry.size;
    let table = lua.create_table()?;
    table.set("x", x)?;
    table.set("y", y)?;
    table.set("width", width)?;
    table.set("height", height)?;
    Ok(table)
}

fn refresh<'lua>(
    _: rlua::Context<'lua>,
    mut drawable: Drawable<'lua>
) -> rlua::Result<()> {
    drawable.refresh()
}

/// Get the data associated with the ImageSurface.
fn get_data(surface: &mut ImageSurface) -> &[u8] {
    // NOTE This is safe to do because there's one thread.
    //
    // We know Lua is not modifying it because it's not running.
    unsafe {
        let len = surface.get_stride() as usize * surface.get_height() as usize;
        let surface = surface.to_glib_none().0;
        slice::from_raw_parts(
            cairo_sys::cairo_image_surface_get_data(surface as _),
            len
        )
    }
}
