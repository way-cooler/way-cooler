use rustwlc::{Geometry};
use super::{Border, TopBorder, BottomBorder, LeftBorder, RightBorder};

/// All the borders of a container (top, bottom, left, right)
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Borders {
    top: Option<TopBorder>,
    bottom: Option<BottomBorder>,
    right: Option<RightBorder>,
    left: Option<LeftBorder>
}


impl Borders {
    pub fn new(top: Option<TopBorder>, bottom: Option<BottomBorder>,
               left: Option<LeftBorder>, right: Option<RightBorder>) -> Self {
        Borders {
            top: top,
            bottom: bottom,
            left: left,
            right: right
        }
    }

    /// Renders each border at their respective geometries.
    pub fn render(&mut self, view_g: Geometry) {
        let mut borders = vec![self.top.as_mut().map(|g| g as &mut Border),
                           self.bottom.as_mut().map(|g| g as &mut Border),
                           self.left.as_mut().map(|g| g as &mut Border),
                           self.right.as_mut().map(|g| g as &mut Border)];
        for border in borders.iter_mut().flat_map(|maybe_g| maybe_g.into_iter()){
            border.draw(view_g);
        }
    }
}
