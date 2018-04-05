use wlroots::Area;

/// A representation of an Output for use in the Awesome module.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Output {
    pub name: String,
    pub effective_resolution: (i32, i32),
    pub focused: bool
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Pointer {
    pub position: (f64, f64)
}

impl Pointer {
    /// Set the position of the pointer.
    pub fn set_position(&mut self, pos: (f64, f64)) {
        // TODO: post to the server
        self.position = pos
    }
}
