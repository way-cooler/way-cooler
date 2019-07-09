//! Utility methods and structures

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
/// Generic geometry-like struct. Contains an origin (x, y) point and bounds
/// (width, height).
pub struct Area {
    pub origin: Origin,
    pub size: Size
}

impl Area {
    /// Makes a new `Area` with width and height set to the values in the given
    /// `Size`.
    pub fn with_size(self, size: Size) -> Self {
        Area { size, ..self }
    }

    /// Makes a new `Area` with x and y set to the value in the given `Origin`.
    #[allow(dead_code)]
    pub fn with_origin(self, origin: Origin) -> Self {
        Area { origin, ..self }
    }
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Origin {
    pub x: i32,
    pub y: i32
}

impl From<Origin> for Area {
    fn from(origin: Origin) -> Self {
        Area {
            origin,
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Size {
    pub width: u32,
    pub height: u32
}

impl From<Size> for Area {
    fn from(size: Size) -> Self {
        Area {
            size,
            ..Default::default()
        }
    }
}
