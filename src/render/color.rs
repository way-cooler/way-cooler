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
    fn from(val: u32) -> Self {
        let blue = ((val & 0xff0000) >> 16) as u8;
        let green = ((val & 0x00ff00) >> 8) as u8;
        let red = (val & 0x0000ff) as u8;
        Color::solid_color(red, green, blue)
    }
}
