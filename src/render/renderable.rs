//! Defines the renderable trait.
//! A renderable object holds the buffer that is written to wlc_pixel_write.
//! The buffer can only be modified by casting the type to a `Drawable`.

use rustwlc::{Geometry, WlcOutput};
use rustwlc::render::{write_pixels, wlc_pixel_format};
use cairo::{self, Context, ImageSurface, Operator};
use super::draw::BaseDraw;

/// Something that can be rendered by wlc.
/// At minimum, it must store a buffer of bytes to write to wlc,
/// a geometry to tell wlc where to place the bytes,
/// and the `WlcOutput` that it will be rendered on.
pub trait Renderable {
    /// Makes a new `Renderable`
    ///
    /// Allowed to return `None` so that a needless allocation doesn't occur
    /// (e.g when the geometry is 0).
    fn new(geometry: Geometry, output: WlcOutput) -> Option<Self>
        where Self: :: std::marker::Sized;

    /// Gets the underlying `ImageSurface` used by the `Renderable` to render
    /// to wlc.
    ///
    /// Is mutable to ensure no other threads are touching the data.
    fn get_surface(&mut self) -> &mut ImageSurface;

    /// Gets the geometry for where the buffer should be rendered on the output.
    fn get_geometry(&self) -> Geometry;

    // TODO Why can't I just use reallocate_buffer here?

    /// Sets the geometry for where the buffer should be render on the output.
    fn set_geometry(&mut self, geometry: Geometry);

    /// Gets the output that this will be rendered on.
    fn get_output(&self) -> WlcOutput;

    /// Reallocates the buffer based on the new geometry.
    ///
    /// Allowed to return `None` so that a needless allocation doesn't occur
    /// (e.g when the geometry is 0).
    fn reallocate_buffer(self, geometry: Geometry) -> Option<Self>
        where Self: ::std::marker::Sized;

    fn enable_cairo(mut self) -> Result<BaseDraw<Self>, cairo::Status>
        where Self: ::std::marker::Sized
    {
        let cairo = Context::new(&*self.get_surface());
        match cairo.status() {
            cairo::Status::Success => {},
            err => return Err(err)
        }
        cairo.set_operator(Operator::Source);
        match cairo.status() {
            cairo::Status::Success => {},
            err => return Err(err)
        }
        Ok(BaseDraw::new(self, cairo))
    }

    /// Renders the data buffer at the internal geometry.
    ///
    /// Automatically ensures that the buffer does not clip the sides
    fn render(&mut self) {
        let output_res = self.get_output().get_resolution()
            .expect("Output had no resolution");
        let mut geometry = self.get_geometry();
        let surface = self.get_surface();
        let buffer = surface.get_data()
            .expect("Could not get surface buffer");

        // If the buffer would clip the side, keep it within the bounds
        // This is done because wlc wraps if it goes beyond, which we don't
        // want for the borders.
        if geometry.origin.x + geometry.size.w as i32 > output_res.w as i32 {
            let offset = (geometry.origin.x + geometry.size.w as i32) - output_res.w as i32;
            geometry.origin.x -= offset as i32;
        }
        if geometry.origin.y + geometry.size.h as i32 > output_res.h as i32 {
            let offset = (geometry.origin.y + geometry.size.h as i32) - output_res.h as i32;
            geometry.origin.y -= offset as i32;
        }
        if geometry.origin.x < 0 {
            geometry.origin.x = 0;
        }
        if geometry.origin.y < 0 {
            geometry.origin.y = 0;
        }
        write_pixels(wlc_pixel_format::WLC_RGBA8888, geometry, &buffer);
    }
}
