use std::ops::{Deref, DerefMut};
use rustwlc::Geometry;
use super::super::borders::Borders;
use ::render::{BaseDraw, Drawable, DrawErr};
use super::super::container::Layout;

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

    fn draw_title_bar(mut self,
                      mut x: f64,
                      mut w: f64,) -> Result<Self, DrawErr<Borders>> {
        let gap = Borders::gap_size() as f64;
        let title_size = Borders::title_bar_size() as f64;
        let title_color = self.base.inner().title_background_color();
        let title_font_color = self.base.inner().title_font_color();
        if x < 0.0 {
            w += x;
        }
        let title_x = Borders::thickness() as f64 + gap / 2.0;
        let title_y = title_size - 5.0;
        x = gap / 2.0;
        w -= gap;

        // TODO: This shouldn't be clone for performance reasons, but self
        // needs to not be borrowed later with all the paint operations, and
        // I didn't know how to work around that.
        let children = self.base.inner().children.clone();
        let layout = self.base.inner().layout;

        match (layout, children) {
            (Some(Layout::Tabbed), Some(children)) => {
                let tab_width = w / children.titles.len() as f64;

                for (i, title) in children.titles.iter().enumerate() {
                    let active = children.index == i;

                    let title_color = if active { title_color }
                        else { Borders::default_title_color() };

                    let title_font_color = if active { title_font_color }
                        else { Borders::default_title_font_color() };

                    let x = x + tab_width*i as f64;
                    let title_x = title_x + tab_width*i as f64;

                    // Draw background of title bar
                    self.base.set_source_rgb(1.0, 0.0, 0.0);
                    self.base.set_color_source(title_color);
                    self.base.rectangle(x, 0.0, tab_width, title_size);
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
                }
            }
            (Some(Layout::Stacked), Some(children)) => {
                for (i, title) in children.titles.iter().enumerate() {
                    let active = children.index == i;

                    let title_color = if active { title_color }
                        else { Borders::default_title_color() };

                    let title_font_color = if active { title_font_color }
                        else { Borders::default_title_font_color() };

                    let y = title_size*i as f64;
                    let title_y = title_y as f64 + y;

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
                }
            }
            _ => {
                let title: String = self.inner().title().into();

                // Draw background of title bar
                self.base.set_source_rgb(1.0, 0.0, 0.0);
                self.base.set_color_source(title_color);
                self.base.rectangle(x, 0.0, w, title_size);
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
            }
        }

        Ok(self)
    }
}

impl Drawable<Borders> for ContainerDraw {
    // Draw the title bar around the container
    fn draw(mut self, container_g: Geometry) -> Result<Borders, DrawErr<Borders>> {
        let mut border_g = container_g;
        let thickness = Borders::thickness();

        let title_count = match (self.inner().layout, &self.inner().children) {
            (Some(Layout::Stacked), &Some(ref children)) =>
                // I don't understand why I have to use this
                // instead of just child_count, but seems to work...
                2 * children.titles.len() as u32 - 1,
            _ => 1
        };

        let title_size = Borders::title_bar_size() * title_count;
        let edge_thickness = thickness / 2;

        border_g.origin.x -= edge_thickness as i32;
        border_g.origin.y -= edge_thickness as i32;
        border_g.origin.y -= (title_size / 2) as i32;
        border_g.size.w += thickness;
        border_g.size.h = edge_thickness + title_size/2;

        self.base.set_source_rgba(0.0, 0.0, 0.0, 0.0);
        self.base.paint();
        let color = self.base.inner().color();
        self.base.set_color_source(color);

        let (x, w) = (border_g.origin.x as f64,
                      border_g.size.w as f64);
        self = self.draw_title_bar(x, w)?;
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
