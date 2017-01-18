use std::iter;
use std::fmt::{self, Debug};
use std::cmp::{Eq, PartialEq};
use std::ops::{Deref, DerefMut};
use rustwlc::{Geometry, Size, Point, WlcOutput};
use rustwlc::render::{write_pixels, wlc_pixel_format, calculate_stride, BITS_PER_PIXEL};
use cairo::{self, Context, ImageSurface, Format, Operator, Status, SolidPattern};
use cairo::prelude::{SurfaceExt};

use super::draw::BaseDraw;
use ::registry;

/// The borders of a container.
///
/// This type just deals with rendering,
#[derive(Clone)]
pub struct Borders {
    /// The surface that contains the bytes we give to wlc to draw.
    surface: ImageSurface,
    /// The geometry where the buffer is written.
    ///
    /// Should correspond with the geometry of the container.
    pub geometry: Geometry
}

impl Borders {
    pub fn new(mut geometry: Geometry) -> Option<Self> {
        let thickness = Borders::thickness();
        if thickness == 0 {
            return None
        }
        // Add the thickness to the geometry.
        // TODO Consolidate this and reallocate_buffer into one function
        geometry.origin.x -= thickness as i32;
        geometry.origin.y -= thickness as i32;
        geometry.size.w += thickness;
        geometry.size.h += thickness;
        let Size { w, h } = geometry.size;
        let stride = calculate_stride(w) as i32;
        let data: Vec<u8> = iter::repeat(0).take(h as usize * stride as usize).collect();
        let buffer = data.into_boxed_slice();
        let surface = ImageSurface::create_for_data(buffer,
                                                    drop_data,
                                                    Format::ARgb32,
                                                    w as i32,
                                                    h as i32,
                                                    stride);
        Some(Borders {
            surface: surface,
            geometry: geometry
        })
    }

    pub fn render(&mut self) {
        let mut buffer = self.surface.get_data()
            .expect("Could not get border surface buffer");
        // TODO Replace this with output of handle.get_output().get_resolution()
        let output_res = WlcOutput::focused().get_resolution()
            .expect("Could not get focused output's resolution");
        let mut geometry = self.geometry;

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
        write_pixels(wlc_pixel_format::WLC_RGBA8888, geometry, &mut buffer);
    }

    /// Updates the position at which we render the buffer.
    ///
    /// Since this is a common operation, and does not resize the buffer,
    /// this is a method is here instead of in a draw struct
    /// for ergonomic and performance reasons.
    pub fn update_pos(&mut self, origin: Point) {
        self.geometry.origin = origin;
    }

    /// Updates/Creates the underlying geometry for the surface/buffer.
    ///
    /// This causes a reallocation of the buffer, do not call this
    /// in a tight loop unless you want memory fragmentation and
    /// bad performance.
    pub fn reallocate_buffer(mut self, mut geometry: Geometry) -> Option<Self>{
        // Add the thickness to the geometry.
        let thickness = Borders::thickness();
        if thickness == 0 {
            return None;
        }
        geometry.origin.x -= thickness as i32;
        geometry.origin.y -= thickness as i32;
        geometry.size.w += thickness;
        geometry.size.h += thickness;
        let Size { w, h } = geometry.size;
        if w == self.geometry.size.w && h == self.geometry.size.h {
            warn!("Geometry is same size, not reallocating");
            return Some(self);
        }
        let stride = calculate_stride(w) as i32;
        let data: Vec<u8> = iter::repeat(0).take(h as usize * stride as usize).collect();
        let buffer = data.into_boxed_slice();
        let surface = ImageSurface::create_for_data(buffer,
                                                    drop_data,
                                                    Format::ARgb32,
                                                    w as i32,
                                                    h as i32,
                                                    stride);
        self.geometry = geometry;
        self.surface = surface;
        Some(self)
    }

    /// Enables Cairo drawing capabilities on the borders.
    ///
    /// While drawing with Cairo, the underlying surface cannot be read.
    /// Borders is consumed in order for this to be checked at compile time.
    pub fn enable_cairo(self) -> Result<BaseDraw, cairo::Status> {
        let cairo = Context::new(&self.surface);
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

    pub fn thickness() -> u32 {
        registry::get_data("border_size")
            .map(registry::RegistryGetData::resolve).and_then(|(_, data)| {
                Ok(data.as_f64().map(|num| {
                    if num <= 0.0 {
                        0u32
                    } else {
                        num as u32
                    }
                }).unwrap_or(0u32))
            }).unwrap_or(0u32)
    }
}

impl Debug for Borders {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Borders")
            .field("geometry", &self.geometry as &Debug)
            .finish()
    }
}

impl PartialEq for Borders {
    fn eq(&self, other: &Borders) -> bool {
        self.geometry == other.geometry
    }
}

impl Eq for Borders {}

unsafe impl Send for Borders {}
unsafe impl Sync for Borders {}


/*
/// Calculates the stride for ARgb32 encoded buffers
fn calculate_stride(width: u32) -> u32 {
    // function stolen from CAIRO_STRIDE_FOR_WIDTH macro in cairoint.h
    // Can be found in the most recent version of the cairo source
    let stride_alignment = ::std::mem::size_of::<u32>() as u32;
    ((BITS_PER_PIXEL * width + 7 ) / 8 + (stride_alignment - 1))  & (stride_alignment.overflowing_neg().0)
}*/

#[allow(dead_code)]
fn drop_data(_: Box<[u8]>) { /*error!("Freeing data")*/ }
