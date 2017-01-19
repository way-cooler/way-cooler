use std::ops::{Deref, DerefMut};
use rustwlc::{Geometry};
use cairo::{self, Context};

use super::super::borders::Borders;
use super::color::Color;

/// The different ways drawing can go wrong.
#[derive(Debug, Clone)]
pub enum DrawErr {
    /// There was an error attempting to use Cairo.
    Cairo(cairo::Status, Borders)
}

/// A drawable type, which can be deconstructed into an underlying
/// `Borders` which can be rendered.
pub trait Drawable {
    /// Draws to the surface, using and transforming the geometry
    /// to suit the `Drawable`'s way of drawing to the buffer.
    ///
    /// On success, returns a renderable `Borders`
    fn draw(self, border_g: Geometry) -> Result<Borders, DrawErr>;
}

/// Implements basic draw functionality.
pub struct BaseDraw {
    /// The inner borders that holds the buffer and geometry information.
    borders: Borders,
    /// Cairo context that modifies border's surface data.
    cairo: Context
}

impl BaseDraw {
    pub fn new(borders: Borders, cairo: Context) -> Self {
        BaseDraw {
            borders: borders,
            cairo: cairo
        }
    }
    /// Cairo requires checking after each operation. The wrapper library did not
    /// implement automatic checks after each operation, and wrapping ourselves would
    /// be too much work. So this performs a check, which can be used with try!
    ///
    /// If the status was anything other than `cairo::Status::Success`, it is an `Err`.
    pub fn check_cairo(self) -> Result<Self, DrawErr> {
        match self.cairo.status() {
            cairo::Status::Success => Ok(self),
            err => Err(DrawErr::Cairo(err, self.borders))
        }
    }

    /// Sets the source to paint with the provided color.
    pub fn set_color_source(&mut self, color: Color) {
        let (r, g, b, a) = color.values();
        self.cairo.set_source_rgba(r as f64 / 255.0,
                                   g as f64 / 255.0,
                                   b as f64 / 255.0,
                                   a as f64 / 255.0);
    }

    /// Finishes drawing on the border, yielding renderable `Borders`.
    pub fn finish(mut self, border_g: Geometry) -> Borders {
        self.borders.geometry = border_g;
        self.borders
    }
}

impl Deref for BaseDraw {
    type Target = Context;

    fn deref(&self) -> &Context {
        &self.cairo
    }
}

impl DerefMut for BaseDraw {
    fn deref_mut(&mut self) -> &mut Context {
        &mut self.cairo
    }
}
