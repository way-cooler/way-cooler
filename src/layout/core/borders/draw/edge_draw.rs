use std::iter;
use std::fmt::{self, Debug};
use std::cmp::{Eq, PartialEq};
use std::ops::{Deref, DerefMut};
use rustwlc::{Geometry, Size, Point, WlcOutput};
use rustwlc::render::{write_pixels, wlc_pixel_format};
use cairo::{Context, ImageSurface, Format, Operator, Status, SolidPattern};
use cairo::prelude::{SurfaceExt};

use super::super::borders::Borders;
use super::base::{BaseDraw, Drawable, DrawErr};
use super::color::Color;

/// Draws the borders with variable width.
///
/// This can be used to hide certain borders,
/// eg when touching the side of the screen.
pub struct EdgeDraw {
    base: BaseDraw,
    color: Color
}

impl EdgeDraw {
    pub fn new(base: BaseDraw, color: Color) -> Self {
        EdgeDraw {
            base: base,
            color: color
        }
    }

    fn draw_left_border(mut self,
                        mut x: f64,
                        mut y: f64,
                        mut w: f64,
                        mut h: f64,
                        border_geometry: Geometry,
                        output_res: Size)
                        -> Result<Self, DrawErr> {
        // yay clamping
        if x < 0.0 {
            w += x;
        }
        if y < 0.0 {
            h += y;
        }
        x = 0.0;
        y = 0.0;
        if border_geometry.origin.x + border_geometry.size.w as i32 > output_res.w as i32{
            let offset = (border_geometry.origin.x + border_geometry.size.w as i32) - output_res.w as i32;
            x += offset as f64;
        }
        if border_geometry.origin.y + border_geometry.size.h as i32 > output_res.h as i32 {
            let offset = (border_geometry.origin.y + border_geometry.size.h as i32) - output_res.h as i32;
            y += offset as f64;
        }
        self.base.rectangle(x, y, w, h);
        self.base = try!(self.base.check_cairo());
        self.base.fill();
        self.base = try!(self.base.check_cairo());
        Ok(self)
    }

    fn draw_right_border(mut self,
                         mut x: f64,
                         mut y: f64,
                         mut w: f64,
                         mut h: f64,
                         border_geometry: Geometry,
                         output_res: Size)
                         -> Result<Self, DrawErr> {
        // yay clamping
        if border_geometry.origin.x < 0 {
            x += border_geometry.origin.x as f64;
        }
        if y < 0.0 {
            h += y;
        }
        y = 0.0;
        if border_geometry.origin.x + border_geometry.size.w as i32 > output_res.w as i32 {
            let offset = (border_geometry.origin.x + border_geometry.size.w as i32) - output_res.w as i32;
            x += offset as f64;
        }
        if border_geometry.origin.y + border_geometry.size.h as i32 > output_res.h as i32 {
            let offset = (border_geometry.origin.y + border_geometry.size.h as i32) - output_res.h as i32;
            y += offset as f64;
        }
        self.base.rectangle(x, y, w, h);
        self.base = try!(self.base.check_cairo());
        self.base.fill();
        self.base = try!(self.base.check_cairo());
        Ok(self)
    }

    fn draw_top_border(mut self,
                         mut x: f64,
                         mut y: f64,
                         mut w: f64,
                         mut h: f64,
                         border_geometry: Geometry,
                         output_res: Size)
                         -> Result<Self, DrawErr> {
        // yay clamping
        if x < 0.0 {
            w += x;
        }
        if y < 0.0 {
            h += y;
        }
        x = 0.0;
        y = 0.0;
        if border_geometry.origin.x + border_geometry.size.w as i32 > output_res.w as i32 {
            let offset = (border_geometry.origin.x + border_geometry.size.w as i32) - output_res.w as i32;
            x += offset as f64;
        }
        if border_geometry.origin.y + border_geometry.size.h as i32 > output_res.h as i32 {
            let offset = (border_geometry.origin.y + border_geometry.size.h as i32) - output_res.h as i32;
            y += offset as f64;
        }
        self.base.rectangle(x, y, w, h);
        self.base = try!(self.base.check_cairo());
        self.base.fill();
        self.base = try!(self.base.check_cairo());
        Ok(self)
    }

    fn draw_bottom_border(mut self,
                            mut x: f64,
                            mut y: f64,
                            mut w: f64,
                            mut h: f64,
                            border_geometry: Geometry,
                            output_res: Size)
                            -> Result<Self, DrawErr> {
        // yay clamping
        if x < 0.0 {
            w += x;
        }
        if border_geometry.origin.y < 0 {
            y += border_geometry.origin.y as f64;
        }
        x = 0.0;
        if border_geometry.origin.x + border_geometry.size.w as i32 > output_res.w as i32 {
            let offset = (border_geometry.origin.x + border_geometry.size.w as i32) - output_res.w as i32;
            x += offset as f64;
        }
        self.base.rectangle(x, y, w, h);
        self.base = try!(self.base.check_cairo());
        self.base.fill();
        self.base = try!(self.base.check_cairo());
        Ok(self)
    }
}

impl Drawable for EdgeDraw {
    fn draw(mut self, view_g: Geometry) -> Result<Borders, DrawErr> {
        let mut border_g = view_g;
        let thickness = self.base.borders().thickness;
        let edge_thickness = thickness / 2;
        // TODO This doesn't seem right, wouldn't this break on multi-head output?
        let output_res = WlcOutput::focused().get_resolution()
            .expect("Could not get focused output's resolution");
        // Even though we ignore these origin values,
        // the renderer needs to know where to start drawing the box.
        border_g.origin.x -= edge_thickness as i32;
        border_g.origin.y -= edge_thickness as i32;
        border_g.size.w += thickness;
        border_g.size.h += thickness;
        warn!("Drawing EdgeDraw @ {:#?}", border_g);

        self.base.set_source_rgba(0.0, 0.0, 0.0, 0.0);
        self.base.paint();
        self.base.set_color_source(self.color);

        let Size { w, h } = border_g.size;
        let Point { x, y } = border_g.origin;
        // basically after everything after this point is floating point arithmetic
        // whee!
        let (x, y, w, h) = (x as f64,
                            y as f64,
                            w as f64,
                            h as f64);
        let edge_thickness = edge_thickness as f64;
        self = self.draw_left_border(x, y, edge_thickness, h, border_g, output_res)?
        .draw_right_border(w - edge_thickness, y, edge_thickness, h, border_g, output_res)?
        .draw_top_border(x, y, w, edge_thickness, border_g, output_res)?
        .draw_bottom_border(x, h, w, -edge_thickness, border_g, output_res)?;

        Ok(self.base.finish(border_g))
    }
}
