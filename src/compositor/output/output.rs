use cairo::ImageSurface;
use cairo_sys;
use glib::translate::ToGlibPtr;
use wlroots::utils::current_time;
use wlroots::wlroots_sys;
use wlroots::{project_box, Area, CompositorHandle, Origin, OutputHandle, OutputHandler,
            OutputLayoutHandle, Renderer, Size, SurfaceHandle, WL_SHM_FORMAT_ARGB8888};

use awesome::{Drawin, Objectable, DRAWINS_HANDLE, LUA};
use compositor::{Server, View};
use rlua::{self, AnyUserData, Lua};
use std::rc::Rc;

pub struct Output;

impl OutputHandler for Output {
fn on_frame(&mut self, compositor: CompositorHandle, output: OutputHandle) {
    dehandle!(
        @compositor = {compositor};
        @output = {output};
        let state: &mut Server = compositor.data.downcast_mut().unwrap();
        let Server { ref mut layout,
                        ref mut views,
                        .. } = *state;
        let renderer = compositor.renderer.as_mut().expect("gles2 disabled");
        let mut renderer = renderer.render(output, None);
            renderer.clear([0.25, 0.25, 0.25, 1.0]);
            render_views(&mut renderer, layout, views);
            LUA.with(|lua| {
                let lua = lua.borrow();
                match render_drawins(&*lua, &mut renderer) {
                    Ok(_) => {},
                    Err(err) => {
                        warn!("Error rendering drawins: {:#?}", err);
                    }
                }
            }));
    }
}

fn render_surface(renderer: &mut Renderer,
                  layout: &mut OutputLayoutHandle,
                  surface: &mut SurfaceHandle,
                  lx: i32,
                  ly: i32) {
    with_handles!([(surface: {surface}), (layout: {&mut *layout})] => {
        let (width, height) = surface.current_state().size();
        let (render_width, render_height) =
            (width * renderer.output.scale() as i32,
            height * renderer.output.scale() as i32);
        let render_box = Area::new(Origin::new(lx, ly),
        Size::new(render_width,
                  render_height));

        if layout.intersects(renderer.output, render_box) {
            let transform = renderer.output.get_transform().invert();
            let matrix = project_box(render_box,
                                     transform,
                                     0.0,
                                     renderer.output
                                     .transform_matrix());
            renderer.render_texture_with_matrix(&surface.texture(), matrix);
            surface.send_frame_done(current_time());
        }

    }).unwrap();
}

/// Render all of the client views.
fn render_views(renderer: &mut Renderer,
                layout: &mut OutputLayoutHandle,
                views: &mut Vec<Rc<View>>) {
    for view in views.iter_mut().rev() {
        let origin = view.origin.get();
        view.for_each_surface(&mut |surface: SurfaceHandle, sx, sy| {
            dehandle!(
                @surface = {surface};
                @layout = {&*layout};
                let (width, height) = surface.current_state().size();
                let (render_width, render_height) =
                    (width * renderer.output.scale() as i32,
                     height * renderer.output.scale() as i32);
                let render_box = Area::new(Origin::new(origin.x + sx, origin.y + sy),
                                           Size::new(render_width,
                                                     render_height));

                if layout.intersects(renderer.output, render_box) {
                    let transform = renderer.output.get_transform().invert();
                    let matrix = project_box(render_box,
                                             transform,
                                             0.0,
                                             renderer.output
                                             .transform_matrix());
                    renderer.render_texture_with_matrix(&surface.texture(), matrix);
                    surface.send_frame_done(current_time());
                })
        });
    }
}

/// Render all of the drawins provided by Lua.
fn render_drawins(lua: &Lua, renderer: &mut Renderer) -> rlua::Result<()> {
    let drawins = lua.named_registry_value::<Vec<AnyUserData>>(DRAWINS_HANDLE)?;
    error!("Drawins: {}", drawins.len());
    for drawin in drawins {
        let mut drawin = Drawin::cast(drawin.into())?;
        if let Some(texture) = drawin.texture()?.take() {
            unsafe {
                error!("Deleting {:p}", texture.as_ptr());
                wlroots_sys::wlr_texture_destroy(texture.as_ptr());
            }
        }
        if !drawin.get_visible()? {
            continue
        }
        let geometry = drawin.get_geometry()?;
        let texture;
        {
            let drawable = drawin.drawable()?;
            let mut drawable_state = drawable.state()?;
            let surface = match drawable_state.surface.as_mut() {
                Some(surface) => surface,
                None => continue
            };
            let Area { size: Size { width, height },
                       .. } = geometry;
            let data = get_data(surface);
            texture = renderer.create_texture_from_pixels(WL_SHM_FORMAT_ARGB8888,
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
        unsafe {
            error!("Saving {:p}", texture.as_ptr());
        }
        *drawin.texture()? = Some(texture);
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
