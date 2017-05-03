use std::ops::{Deref, DerefMut};
use rustwlc::{Geometry, Size, Point};
use super::super::borders::Borders;
use ::render::{BaseDraw, Drawable, DrawErr};

/// Draws the borders around the container.
/// The only borders that are drawn are for the title bars, as the individual
/// edge borders are handled by the child views.
pub struct ContainerDraw {
    base: BaseDraw<Borders>,
}

impl ContainerDraw {
    pub fn new(base: BaseDraw<Borders>) -> Self {
        ContainerDraw {
            base
        }
    }
}

impl Drawable<Borders> for ContainerDraw {
    // Draw the title bar around the container
    fn draw(mut self, container_g: Geometry) -> Result<Borders, DrawErr<Borders>> {
        let mut border_g = container_g;
        let thickness = Borders::thickness();
        let title_size = Borders::title_bar_size();
        let edge_thickness = thickness / 2;
        let title_color = self.base.inner().title_background_color();
        let title: String = self.inner().title().into();
        let output_res = self.inner().get_output().get_resolution()
            .expect("Could not gett focused output's resolution");

        border_g.origin.x -= edge_thickness as i32;
        border_g.size.w += thickness;
        let mut title_x = thickness;
        let mut title_y = title_size - 5;

        // If off the screen to the left or right, adjust.
        if border_g.origin.x < 0 {
            border_g.size.w += border_g.origin.x as u32;
        } else if border_g.origin.x + border_g.size.w as i32 > output_res.w as i32 {
            let offset = border_g.origin.x + border_g.size.w as i32 - output_res.w as i32;
            border_g.origin.x += offset;
            title_x += offset as u32;
        }
        // If over the top, adjust.
        // TODO Handle bottom as well, for title bars on the bottom of container.
        if border_g.origin.y + border_g.size.h as i32 > output_res.h as i32 {
            let offset = border_g.origin.y + border_g.size.h as i32 - output_res.h as i32;
            border_g.origin.y += offset;
            title_y += offset as u32;
        }

        // Draw background of title bar
        self.base.set_source_rgb(1.0, 0.0, 0.0);
        self.base.set_color_source(title_color);
        self.base.rectangle(border_g.origin.x as f64,
                            border_g.origin.y as f64,
                            border_g.size.w as f64,
                            title_size as f64);
        self.base = try!(self.base.check_cairo());
        self.base.fill();
        self.base = try!(self.base.check_cairo());

        // Draw title text
        self.base.move_to(title_x as f64, title_y as f64);
        self.base = try!(self.base.check_cairo());
        self.base.set_color_source(title_color);
        self.base = try!(self.base.check_cairo());
        self.base.show_text(title.as_str());
        self.base = try!(self.base.check_cairo());

        Ok(self.base.finish(border_g))
    }
}

impl Deref for ContainerDraw {
    type Target = BaseDraw<Borders>;

    fn deref(&self) -> &BaseDraw<Borders> {
        &self.base
    }
}

impl DerefMut for ContainerDraw {
    fn deref_mut(&mut self) -> &mut BaseDraw<Borders> {
        &mut self.base
    }
}
