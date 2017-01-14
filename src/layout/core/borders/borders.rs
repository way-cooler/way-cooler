use std::iter;
use std::fmt::{self, Debug};
use std::cmp::{Eq, PartialEq};
use std::ops::{Deref, DerefMut};
use rustwlc::{Geometry, Size, Point};
use rustwlc::render::{write_pixels, wlc_pixel_format, BITS_PER_PIXEL};
use cairo::{self, Context, ImageSurface, Format, Operator, Status, SolidPattern};
use cairo::prelude::{SurfaceExt};

use super::draw::BaseDraw;

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
    pub fn new(geometry: Geometry) -> Self {
        let Size { w, h } = geometry.size;
        let stride = calculate_stride(w) as i32;
        // TODO Remove 100 so that we don't start with a gray ghost box
        let data: Vec<u8> = iter::repeat(100).take(h as usize * stride as usize).collect();
        let buffer = data.into_boxed_slice();
        let surface = ImageSurface::create_for_data(buffer,
                                                    drop_data,
                                                    Format::ARgb32,
                                                    w as i32,
                                                    h as i32,
                                                    stride);
        Borders {
            surface: surface,
            geometry: geometry
        }
    }

    pub fn render(&mut self) {
        let mut buffer = self.surface.get_data()
            .expect("Could not get border surface buffer");
        write_pixels(wlc_pixel_format::WLC_RGBA8888, self.geometry, &mut buffer);
    }

    /// Updates the position at which we render the buffer.
    ///
    /// Since this is a common operation, and does not resize the buffer,
    /// this is a method is here instead of in a draw struct
    /// for ergonomic and performance reasons.
    pub fn update_pos(&mut self, origin: Point) {
        self.geometry.origin = origin;
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
}

impl Debug for Borders {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Debug")
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


/// Calculates the stride for ARgb32 encoded buffers
fn calculate_stride(width: u32) -> u32 {
    // function stolen from CAIRO_STRIDE_FOR_WIDTH macro in cairoint.h
    // Can be found in the most recent version of the cairo source
    let stride_alignment = ::std::mem::size_of::<u32>() as u32;
    ((BITS_PER_PIXEL * width + 7 ) / 8 + (stride_alignment - 1))  & (stride_alignment.overflowing_neg().0)
}

#[allow(dead_code)]
fn drop_data(_: Box<[u8]>) {}
