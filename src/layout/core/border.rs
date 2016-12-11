use std::iter;
use rustwlc::Size;
use std::fmt::{self, Debug};
use std::cmp::{Eq, PartialEq};
use cairo::{Context, ImageSurface, Format, Operator, Status, SolidPattern};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Color {
    red: u32,
    green: u32,
    blue: u32
}

#[derive(Clone)]
/// A border for a container.
pub struct Border {
    context: Context,
    thickness: u32,
    color: Color,
    size: Size
}

/// All the borders of a container (top, bottom, left, right)
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Borders {
    top: Option<Border>,
    bottom: Option<Border>,
    right: Option<Border>,
    left: Option<Border>
}

impl Color {
    pub fn new(red: u32, green: u32, blue: u32) -> Self {
        Color { red: red, green: green, blue: blue}
    }
}

impl Border {
    pub fn new(size: Size, thickness: u32, color: Color) -> Self {
        let Size { w, h } = size;
        let h = h as i32;
        let w = w as i32;
        // TODO Magic numbers
        let stride = calculate_stride(w as u32) as i32;
        let data: Vec<u8> = iter::repeat(0).take(w as usize * stride as usize).collect();
        let buffer = data.into_boxed_slice();
        let surface = ImageSurface::create_for_data(buffer, drop_data, Format::ARgb32, h, w, stride);
        let context = Context::new(&surface);
        context.set_operator(Operator::Source);
        match context.status() {
            Status::Success => {},
            err => {
                panic!("Cairo context failed with {:#?}", err);
            }
        }
        Border {
            context: context,
            thickness: thickness,
            color: color,
            size: size
        }
    }

    /// Renders a line, starting from (x, y) to (x + width, y + height).
    pub fn render(&mut self, mut x: f64, mut y: f64) {
        let Color { red, green, blue} = self.color;
        let Size { w, h } = self.size;
        let mut w = w as f64;
        let mut h = h as f64;
        let pattern = SolidPattern::from_rgb(red as f64 / 255.0,
                                             green as f64 / 255.0,
                                             blue as f64 / 255.0);
        self.context.set_source(&pattern);
        if w > 1.0 && h > 1.0 {
            self.context.rectangle(x, y, w, h);
            self.context.fill();
        } else {
            if w == 1.0 {
                x += 0.5;
                h += y;
                w = x;
            }

            if h == 1.0 {
                y += 0.5;
                w += x;
                h = y;
            }

            self.context.move_to(x, y);
            self.context.set_line_width(1.0);
            self.context.line_to(w, h);
            self.context.stroke();
        }
    }
}

impl Debug for Border {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Debug")
            .field("thickness", &self.thickness as &Debug)
            .field("color", &self.color as &Debug)
            .finish()
    }
}

impl PartialEq for Border {
    fn eq(&self, other: &Border) -> bool {
        (self.thickness == other.thickness && self.color == other.color
         && self.size == other.size)
    }
}

impl Eq for Border {}

unsafe impl Send for Border {}
unsafe impl Sync for Border {}

impl Borders {
    pub fn new(top: Option<Border>, bottom: Option<Border>,
               left: Option<Border>, right: Option<Border>) -> Self {
        Borders {
            top: top,
            bottom: bottom,
            left: left,
            right: right
        }
    }
}

/// Calculates the stride for ARgb32 encoded buffers
fn calculate_stride(width: u32) -> u32 {
    // function stolen from CAIRO_STRIDE_FOR_WIDTH macro in cairoint.h
    // Can be found in the most recent version of the cairo source
    (32 * width + 7) / 8 + 4 & !4
}

fn drop_data(_: Box<[u8]>) {}


#[cfg(tests)]
mod tests {
    use super::calculate_stride;

    #[test]
    fn test_calculate_stride() {
        let test_data = [
            (100, 400),
            (200, 800),
            (300, 1200),
            (400, 1600),
            (500, 2000),
            (600, 2400),
            (700, 2800),
            (800, 3200),
            (900, 3600)
        ];
        for &(width, stride) in test_data.iter() {
            assert_eq!(calculate_stride(width), stride);
        }
    }
}
