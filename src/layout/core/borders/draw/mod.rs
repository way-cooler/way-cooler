// TODO This module should be abstracted outside of borders,
/// and become simple drawing utilities used by all of way cooler.
mod color;
mod base;
mod simple_draw;
mod edge_draw;

pub use self::color::Color;
pub use self::base::{BaseDraw, Drawable, DrawErr};
pub use self::simple_draw::SimpleDraw;
pub use self::edge_draw::EdgeDraw;

