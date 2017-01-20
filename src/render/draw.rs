use std::ops::{Deref, DerefMut};
use rustwlc::{Geometry};
use cairo::{self, Context};

use super::renderable::Renderable;
use super::color::Color;

/// The different ways drawing can go wrong.
#[derive(Debug, Clone)]
pub enum DrawErr<T> {
    /// There was an error attempting to use Cairo.
    Cairo(cairo::Status, T)
}

/// A drawable type, which can be deconstructed into an underlying `Renderable`
pub trait Drawable<T: Renderable> {
    /// Draws to the surface, using and transforming the geometry
    /// to suit the `Drawable`'s way of drawing to the buffer.
    ///
    /// On success, returns a ready to render `Renderable`.
    fn draw(self, inner_g: Geometry) -> Result<T, DrawErr<T>>;
}

/// Implements basic draw functionality.
pub struct BaseDraw<T: Renderable> {
    /// The inner `Renderable` that holds the buffer and geometry information.
    inner: T,
    /// Cairo context that modifies border's surface data.
    cairo: Context
}

impl<T: Renderable> BaseDraw<T> {
    pub fn new(inner: T, cairo: Context) -> Self {
        BaseDraw {
            inner: inner,
            cairo: cairo
        }
    }
    /// Cairo requires checking after each operation. The wrapper library did not
    /// implement automatic checks after each operation, and wrapping ourselves would
    /// be too much work. So this performs a check, which can be used with try!
    ///
    /// If the status was anything other than `cairo::Status::Success`, it is an `Err`.
    pub fn check_cairo(self) -> Result<Self, DrawErr<T>> {
        match self.cairo.status() {
            cairo::Status::Success => Ok(self),
            err => Err(DrawErr::Cairo(err, self.inner))
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

    /// Finishes drawing on the border, yielding `Renderable`.
    pub fn finish(mut self, border_g: Geometry) -> T {
        //TODO Why not this again?
        //self.reallocate_buffer(border_g);
        self.inner.set_geometry(border_g);
        self.inner
    }

    /// Gets a reference to the inner T, which may have additional data
    /// to use for rendering.
    pub fn inner(&self) -> &T {
        &self.inner
    }
}

impl<T: Renderable> Deref for BaseDraw<T> {
    type Target = Context;

    fn deref(&self) -> &Context {
        &self.cairo
    }
}

impl<T: Renderable> DerefMut for BaseDraw<T> {
    fn deref_mut(&mut self) -> &mut Context {
        &mut self.cairo
    }
}
