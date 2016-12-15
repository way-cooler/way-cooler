use std::iter;
use std::fmt::{self, Debug};
use std::cmp::{Eq, PartialEq};
use std::ops::{Deref, DerefMut};
use rustwlc::{Geometry, Size, Point};
use rustwlc::render::{write_pixels, wlc_pixel_format};
use cairo::{Context, ImageSurface, Format, Operator, Status, SolidPattern};
use cairo::prelude::{SurfaceExt};


/// The borders of a container
///
/// This type just deals with rendering,
/// and only renders when it's surface has become "dirty"
pub struct Borders {
    surface: ImageSurface,
    geometry: Geometry,
    dirty: bool
}

impl Borders {
    fn new(surface: ImageSurface, geometry: Geometry) -> Self {
        Borders {
            surface: surface,
            geometry: geometry,
            dirty: true
        }
    }

    pub fn render(&mut self) {
        if self.dirty {
            let mut buffer = self.surface.get_data()
                .expect("Could not get border surface buffer");
            write_pixels(wlc_pixel_format::WLC_RGBA8888, self.geometry, &mut buffer);
            self.dirty = true;
        }
    }
}
