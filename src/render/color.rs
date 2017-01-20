//! Colors used for drawing to a Cairo buffer

use std::convert::From;

/// Color to draw to the screen, including the alpha channel.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Color {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8
}


impl Color {
    /// Makes a new solid color, with no transparency.
    pub fn solid_color(red: u8, green: u8, blue: u8) -> Self {
        Color {
            red: red,
            green: green,
            blue: blue,
            alpha: 255
        }
    }

    /// Gets the values of the colors, in this order:
    /// (Red, Green, Blue, Alpha)
    pub fn values(&self) -> (u8, u8, u8, u8) {
        (self.red, self.green, self.blue, self.alpha)
    }
}

impl From<u32> for Color {
    fn from(mut val: u32) -> Self {
        // Ignore first two bits, we don't care about alpha
        val <<= 2;
        let values = unsafe { ::std::mem::transmute::<u32, [u8; 4]>(val) };
        Color::solid_color(values[2], values[1], values[0])
    }
}
