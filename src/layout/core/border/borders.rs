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
    pub fn render(&self, view_g: Geometry) {
        let borders = vec![self.top.as_ref().map(|g| g as &Border),
                           self.bottom.as_ref().map(|g| g as &Border),
                           self.left.as_ref().map(|g| g as &Border),
                           self.right.as_ref().map(|g| g as &Border)];
        for border in borders.iter().flat_map(|maybe_g| maybe_g.into_iter()){
            border.render(view_g);
        }
    }
}
