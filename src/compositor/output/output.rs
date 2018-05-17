use std::time::{SystemTime, UNIX_EPOCH};

use cairo::ImageSurface;
use cairo_sys;
use glib::translate::ToGlibPtr;
use wlroots::{project_box, Area, CompositorHandle, OutputHandle, OutputHandler,
              OutputLayoutHandle, Renderer, Size, WL_SHM_FORMAT_ARGB8888};

use awesome::{Drawin, Objectable, DRAWINS_HANDLE, LUA};
use compositor::{Server, View};
use rlua::{self, AnyUserData, Lua};

pub struct Output;

impl OutputHandler for Output {
    fn on_frame(&mut self, compositor: CompositorHandle, output: OutputHandle) {
        with_handles!([(compositor: {compositor}), (output: {output})] => {
            let state: &mut Server = compositor.data.downcast_mut().unwrap();
            let Server { ref mut layout,
                         ref mut views,
                         .. } = *state;
            let renderer = compositor.renderer.as_mut().expect("gles2 disabled");
            let mut renderer = renderer.render(output, None);
            renderer.clear([0.25, 0.25, 0.25, 1.0]);
            render_views(&mut renderer, layout, views);
            LUA.with(|lua| {
                let mut lua = lua.borrow_mut();
                match render_drawins(&mut *lua, &mut renderer) {
                    Ok(_) => {},
                    Err(err) => {
                        warn!("Error rendering drawins: {:#?}", err);
                    }
                }
            });
        }).unwrap();
    }
}

/// Render all of the client views.
fn render_views(renderer: &mut Renderer, layout: &mut OutputLayoutHandle, views: &mut [View]) {
    for view in views {
        let mut surface = view.shell.surface();
        with_handles!([(surface: {surface}),
                       (layout: {&mut *layout})] => {
            let (width, height) = surface.current_state().size();
            let (render_width, render_height) =
                (width * renderer.output.scale() as i32,
                 height * renderer.output.scale() as i32);
            let render_box = Area::new(view.origin,
                                       Size::new(render_width,
                                                 render_height));
            if layout.intersects(renderer.output, render_box) {
                let transform = renderer.output.get_transform().invert();
                let matrix = project_box(render_box,
                                         transform,
                                         0.0,
                                         renderer.output
                                         .transform_matrix());
                renderer.render_texture_with_matrix(&surface.texture(),
                                                    matrix);
                let start = SystemTime::now();
                let now = start.duration_since(UNIX_EPOCH)
                    .expect("Time went backwards");
                surface.send_frame_done(now);
            }
        }).expect("Could not render views")
    }
}

/// Render all of the drawins provided by Lua.
fn render_drawins(lua: &Lua, renderer: &mut Renderer) -> rlua::Result<()> {
    let drawins = lua.named_registry_value::<Vec<AnyUserData>>(DRAWINS_HANDLE)?;
    for drawin in drawins {
        let mut drawin = Drawin::cast(drawin.into())?;
        if !drawin.get_visible()? {
            continue
        }
        let geometry = drawin.get_geometry()?;
        let drawable = drawin.drawable()?;
        let mut drawable_state = drawable.state()?;
        let surface = match drawable_state.surface.as_mut() {
            Some(surface) => surface,
            None => continue
        };
        let Area { size: Size { width, height },
                   .. } = geometry;
        let data = get_data(surface);
        let texture = renderer.create_texture_from_pixels(WL_SHM_FORMAT_ARGB8888,
                                                          (width * 4) as _,
                                                          width as _,
                                                          height as _,
                                                          data)
                              .expect("Could not allocate texture");
        let transform_matrix = renderer.output.transform_matrix();
        let inverted_transform = renderer.output.get_transform().invert();
        let matrix = project_box(geometry, inverted_transform, 0.0, transform_matrix);
        renderer.render_texture_with_matrix(&texture, matrix);
    }
    Ok(())
}

/// Get the data associated with an ImageSurface.
fn get_data(surface: &mut ImageSurface) -> &[u8] {
    use std::slice;
    // NOTE This is safe to do because there's one thread.
    //
    // We know Lua is not modifying it because it's not running.
    //
    // Otherwise we'd need to make a copy of the buffer. This ensure we
    // don't need to do that.
    unsafe {
        let len = surface.get_stride() as usize * surface.get_height() as usize;
        let surface = surface.to_glib_none().0;
        slice::from_raw_parts(cairo_sys::cairo_image_surface_get_data(surface as _), len)
    }
}
