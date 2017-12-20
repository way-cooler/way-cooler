use std::ops::{Deref, DerefMut};
use rustwlc::{Geometry, Size, Point};
use super::super::borders::Borders;
use ::render::{BaseDraw, Drawable, DrawErr};

/// Draws the borders around windows.
/// They are all of the same size, including the top.
pub struct ViewDraw {
    base: BaseDraw<Borders>
}

impl ViewDraw {
    pub fn new(base: BaseDraw<Borders>) -> Self {
        ViewDraw {
            base: base
        }
    }

    fn draw_left_border(mut self,
                        mut x: f64,
                        mut y: f64,
                        mut w: f64,
                        mut h: f64,
                        border_geometry: Geometry,
                        output_res: Size)
                        -> Result<Self, DrawErr<Borders>> {
        let title_size = self.base.inner().title_bar_size() as f64;
        // yay clamping
        if x < 0.0 {
            w += x;
        }
        if y < 0.0 {
            h += y - title_size;
        }
        x = 0.0;
        y = title_size;
        if border_geometry.origin.x + border_geometry.size.w as i32 > output_res.w as i32 {
            let offset = (border_geometry.origin.x + border_geometry.size.w as i32)
                - output_res.w as i32;
            x += offset as f64;
        }
        if border_geometry.origin.y + border_geometry.size.h as i32 > output_res.h as i32 {
            let offset = (border_geometry.origin.y + border_geometry.size.h as i32)
                - output_res.h as i32;
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
                         w: f64,
                         mut h: f64,
                         border_geometry: Geometry,
                         output_res: Size)
                         -> Result<Self, DrawErr<Borders>> {
        let title_size = self.base.inner().title_bar_size() as f64;
        // yay clamping
        if border_geometry.origin.x < 0 {
            x += border_geometry.origin.x as f64;
        }
        if y < 0.0 {
            h += y - title_size;
        }
        y = title_size;
        if border_geometry.origin.x + border_geometry.size.w as i32 > output_res.w as i32 {
            let offset = (border_geometry.origin.x + border_geometry.size.w as i32)
                - output_res.w as i32;
            x += offset as f64;
        }
        if border_geometry.origin.y + border_geometry.size.h as i32 > output_res.h as i32 {
            let offset = (border_geometry.origin.y + border_geometry.size.h as i32)
                - output_res.h as i32;
            y += offset as f64;
        }
        self.base.rectangle(x, y, w, h);
        self.base = try!(self.base.check_cairo());
        self.base.fill();
        self.base = try!(self.base.check_cairo());
        Ok(self)
    }

    fn draw_title_bar(mut self,
                      mut x: f64,
                      _y: f64,
                      mut w: f64,
                      _h: f64,
                      border_geometry: Geometry,
                      output_res: Size) -> Result<Self, DrawErr<Borders>> {
        let title_size = self.base.inner().title_bar_size() as f64;
        let title_color = self.base.inner().title_background_color();
        let title_font_color = self.base.inner().title_font_color();
        let title: String = self.inner().title().into();
        if x < 0.0 {
            w += x;
        }
        let mut title_x = Borders::thickness() as f64;
        let mut title_y = title_size - 5.0;
        x = 0.0;
        let mut y = 0.0;
        if border_geometry.origin.x + border_geometry.size.w as i32 > output_res.w as i32 {
            let offset = (border_geometry.origin.x + border_geometry.size.w as i32)
                - output_res.w as i32;
            x += offset as f64;
            title_x += offset as f64;
        }
        if border_geometry.origin.y + border_geometry.size.h as i32 > output_res.h as i32 {
            let offset = (border_geometry.origin.y + border_geometry.size.h as i32)
                - output_res.h as i32;
            y += offset as f64;
            title_y += offset as f64;
        }
        // Draw background of title bar
        self.base.set_source_rgb(1.0, 0.0, 0.0);
        self.base.set_color_source(title_color);
        self.base.rectangle(x, y, w, title_size);
        self.base = try!(self.base.check_cairo());
        self.base.fill();
        self.base = try!(self.base.check_cairo());

        // Draw title text
        self.base.move_to(title_x, title_y);
        self.base = try!(self.base.check_cairo());
        self.base.set_color_source(title_font_color);
        self.base = try!(self.base.check_cairo());
        self.base.show_text(title.as_str());
        self.base = try!(self.base.check_cairo());
        Ok(self)
    }

    fn draw_top_border(mut self,
                         mut x: f64,
                         mut y: f64,
                         mut w: f64,
                         mut h: f64,
                         border_geometry: Geometry,
                         output_res: Size) -> Result<Self, DrawErr<Borders>> {
        // Don't draw the top bar when we are in tabbed/stacked
        if !self.base.inner().draw_title {
            return Ok(self)
        }
        let title_size = self.base.inner().title_bar_size() as f64;
        // yay clamping
        if x < 0.0 {
            w += x;
        }
        if y < 0.0 {
            h += y;
        }
        x = 0.0;
        y = title_size;
        if border_geometry.origin.x + border_geometry.size.w as i32 > output_res.w as i32 {
            let offset = (border_geometry.origin.x + border_geometry.size.w as i32)
                - output_res.w as i32;
            x += offset as f64;
        }
        if border_geometry.origin.y + border_geometry.size.h as i32 > output_res.h as i32 {
            let offset = (border_geometry.origin.y + border_geometry.size.h as i32)
                - output_res.h as i32;
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
                            h: f64,
                            border_geometry: Geometry,
                            output_res: Size)
                            -> Result<Self, DrawErr<Borders>> {
        // yay clamping
        if x < 0.0 {
            w += x;
        }
        if border_geometry.origin.y < 0 {
            y += border_geometry.origin.y as f64;
        }
        x = 0.0;
        if border_geometry.origin.x + border_geometry.size.w as i32 > output_res.w as i32 {
            let offset = (border_geometry.origin.x + border_geometry.size.w as i32)
                - output_res.w as i32;
            x += offset as f64;
        }
        if border_geometry.origin.y + border_geometry.size.h as i32 > output_res.h as i32 {
            let offset = (border_geometry.origin.y + border_geometry.size.h as i32)
                - output_res.h as i32;
            y += offset as f64;
        }
        self.base.rectangle(x, y, w, h);
        self.base = try!(self.base.check_cairo());
        self.base.fill();
        self.base = try!(self.base.check_cairo());
        Ok(self)
    }
}

impl Drawable<Borders> for ViewDraw {
    fn draw(mut self, view_g: Geometry) -> Result<Borders, DrawErr<Borders>> {
        let mut border_g = view_g;
        let thickness = Borders::thickness();
        let edge_thickness = thickness / 2;
        let output_res = self.inner().get_output().get_resolution()
            .expect("Could not get focused output's resolution");
        let title_size = self.base.inner().title_bar_size();
        border_g.origin.x -= edge_thickness as i32;
        border_g.origin.y -= edge_thickness as i32;
        border_g.origin.y -= title_size as i32;
        border_g.size.w += thickness;
        border_g.size.h += thickness;
        border_g.size.h += title_size;

        self.base.set_source_rgba(0.0, 0.0, 0.0, 0.0);
        self.base.paint();
        let color = self.base.inner().color();
        self.base.set_color_source(color);

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
        .draw_bottom_border(x, h, w, -edge_thickness, border_g, output_res)?
        .draw_title_bar(x, y, w, edge_thickness, border_g, output_res)?;

        Ok(self.base.finish(border_g))
    }
}

impl Deref for ViewDraw {
    type Target = BaseDraw<Borders>;

    fn deref(&self) -> &BaseDraw<Borders> {
        &self.base
    }
}

impl DerefMut for ViewDraw {
    fn deref_mut(&mut self) -> &mut BaseDraw<Borders> {
        &mut self.base
    }
}
