
use std::iter;
use std::fmt::{self, Debug};
use std::cmp::{Eq, PartialEq};
use std::ops::{Deref, DerefMut};
use rustwlc::{Geometry, Size, Point};
use rustwlc::render::{write_pixels, wlc_pixel_format};
use cairo::{Context, ImageSurface, Format, Operator, Status, SolidPattern};
use cairo::prelude::{SurfaceExt};

use super::super::borders::Borders;
use super::base::{BaseDraw, Drawable, DrawErr};
use super::color::Color;

/// Draws the borders simply, with a solid color at the same thickness.
pub struct SimpleDraw {
    base: BaseDraw,
    color: Color,
    thickness: u32
}

impl SimpleDraw {
    pub fn new(base: BaseDraw, color: Color, thickness: u32) -> Self {
        SimpleDraw {
            base: base,
            color: color,
            thickness: thickness
        }
    }
}

impl Drawable for SimpleDraw {
    fn draw(self, mut border_g: Geometry) -> Result<Borders, DrawErr> {
        border_g.size.w += self.thickness;
        border_g.size.h += self.thickness;
        border_g.origin.x -= self.thickness as i32 / 2;
        border_g.origin.y -= self.thickness as i32 / 2;
        warn!("Drawing: {:#?}", border_g);

        let mut base = self.base;
        base.set_color_source(self.color);
        base.rectangle(border_g.origin.x as f64,
                       border_g.origin.y as f64,
                       border_g.size.w as f64,
                       border_g.size.h as f64);
        base = try!(base.check_cairo());
        base.fill();
        base = try!(base.check_cairo());
        Ok(base.finish(border_g))
    }
}
