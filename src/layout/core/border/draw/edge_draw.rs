use std::iter;
use std::fmt::{self, Debug};
use std::cmp::{Eq, PartialEq};
use std::ops::{Deref, DerefMut};
use rustwlc::{Geometry, Size, Point};
use rustwlc::render::{write_pixels, wlc_pixel_format};
use cairo::{Context, ImageSurface, Format, Operator, Status, SolidPattern};
use cairo::prelude::{SurfaceExt};

use super::super::borders::{Borders};

/// Draws the borders with variable width.
///
/// This can be used to hide certain borders,
/// eg when touching the side of the screen.
pub struct EdgeDraw {
    borders: Borders,
}
