//! Colors used for drawing to a Cairo buffer

use std::convert::From;
use std::str::Chars;

const WLC_BUG_INVERT_RED_BLUE: bool = true;

/// Color to draw to the screen, including the alpha channel.
/// NOTE: At this point, the parsed colors return the colors red and blue switched.
/// This is due to a bug in WLC, causing the colors to be switched when drawing.
/// Example: "00FF0000" will draw as red (correct), but the Color structure will contain 0 for `red` and 255 for `blue`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Color {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8
}


impl Color {

    /// Creates a new color with an alphachannel
    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
	if WLC_BUG_INVERT_RED_BLUE {
            // There is a bug in wlc, causing red and blue to be inverted:
            // https://github.com/Cloudef/wlc/issues/142
            // We can work around it, by just switching red with blue, until the issue is resolved.
            // When the bug is fixed, this code path can be deleted and the tests must be adjusted.
            Color {
                red:   b,
                green: g,
                blue:  r,
                alpha: a
            }
        } else {
            Color {
                red:   r,
                green: g,
                blue:  b,
                alpha: a
            }
        }
    }

    /// Gets the values of the colors, in this order:
    /// (Red, Green, Blue, Alpha)
    pub fn values(&self) -> (u8, u8, u8, u8) {
        (self.red, self.green, self.blue, self.alpha)
    }

    /// Parses a String into a Color
    /// The following formats are supported:
    /// - "RRGGBB"
    /// - "AARRGGBB"
    /// - "#RRGGBB"
    /// - "#AARRGGBB"
    /// - "0xRRGGBB"
    /// - "0xAARRGGBB"
    pub fn parse(s: String) -> Option<Color> {
        if s.starts_with("#") {
            Color::parse(s[1..].to_string())
        } else if s.starts_with("0x") {
            Color::parse(s[2..].to_string())
        } else if s.len() == 8 {
            Color::parse_argb(s)
        } else if s.len() == 6 {
            Color::parse_rgb(s)
        } else {
            None
        }
    }

    /// Parses an ARGB String into a Color
    fn parse_argb(s: String) -> Option<Color> {
        if s.len() == 8 {
            let mut chars = s.chars();
            let alpha = Color::parse_color(&mut chars);
            let color = Color::parse_rgb(chars.as_str().to_string());
            if WLC_BUG_INVERT_RED_BLUE {
		// Due to the bug, the colors are already inverted, so in the returned color
                // red is blue and blue is red.
                alpha.and_then(|a| color.map(|rgb| Color::rgba(rgb.blue, rgb.green, rgb.red, a)))
            } else {
                alpha.and_then(|a| color.map(|rgb| Color::rgba(rgb.red, rgb.green, rgb.blue, a)))
            }
        } else {
            None
        }
    }

    /// Parses a RGB String into a Color
    fn parse_rgb(s: String) -> Option<Color> {
        if s.len() == 6 {
            let mut chars = s.chars();
            let red   = Color::parse_color(&mut chars);
            let green = Color::parse_color(&mut chars);
            let blue  = Color::parse_color(&mut chars);
            red.and_then(|r| green.and_then(|g| blue.map(|b| Color::rgba(r, g, b, 255))))
        } else {
            None
        }
    }

    /// Parses exactly one single color value from a String (eg "AA", "RR", "GG" or "BB")
    fn parse_color(chars: &mut Chars) -> Option<u8> {
        let mut next_two_chars = chars.take(2);
        let digit1 = Color::parse_next_char(next_two_chars.next());
        let digit2 = Color::parse_next_char(next_two_chars.next());
        digit1.and_then(|i1| digit2.map(|i2| (i1 << 4) | i2))
    }

    fn parse_next_char(c: Option<char>) -> Option<u8> {
        c.and_then(Color::hex_to_u8)
    }

    /// Converts a hex char into a u8
    fn hex_to_u8(c: char) -> Option<u8> {
        c.to_digit(16).map(|x| (x as u8))
    }

}

impl From<u32> for Color {
    fn from(val: u32) -> Self {
        let red   = ((val & 0xff0000) >> 16) as u8;
        let green = ((val & 0x00ff00) >> 8)  as u8;
        let blue  =  (val & 0x0000ff)        as u8;
        Color::rgba(red, green, blue, 255)
    }
}

mod test {

    use ::render::Color;

    #[test]
    fn test_from_u32() {
        let hex_red   = 0xFF0000;
	let hex_green = 0x00FF00;
        let hex_blue  = 0x0000FF;
	let r: Color = hex_red.into();
	let g: Color = hex_green.into();
	let b: Color = hex_blue.into();
	// test red values
	assert_eq!(0x00, r.red);
	assert_eq!(0x00, r.green);
	assert_eq!(0xFF, r.blue);
	// test green values
	assert_eq!(0x00, g.red);
	assert_eq!(0xFF, g.green);
	assert_eq!(0x00, g.blue);
	// test blue values
	assert_eq!(0xFF, b.red);
	assert_eq!(0x00, b.green);
	assert_eq!(0x00, b.blue);
    }
    #[test]
    fn parse_color() {
        // test all numbers, uppercase and lowercase letters
        assert_eq!(17 * 0,  Color::parse_color(&mut "00".chars()).unwrap());
        assert_eq!(17 * 1,  Color::parse_color(&mut "11".chars()).unwrap());
        assert_eq!(17 * 2,  Color::parse_color(&mut "22".chars()).unwrap());
        assert_eq!(17 * 3,  Color::parse_color(&mut "33".chars()).unwrap());
        assert_eq!(17 * 4,  Color::parse_color(&mut "44".chars()).unwrap());
        assert_eq!(17 * 5,  Color::parse_color(&mut "55".chars()).unwrap());
        assert_eq!(17 * 6,  Color::parse_color(&mut "66".chars()).unwrap());
        assert_eq!(17 * 7,  Color::parse_color(&mut "77".chars()).unwrap());
        assert_eq!(17 * 8,  Color::parse_color(&mut "88".chars()).unwrap());
        assert_eq!(17 * 9,  Color::parse_color(&mut "99".chars()).unwrap());
        assert_eq!(17 * 10, Color::parse_color(&mut "aa".chars()).unwrap());
        assert_eq!(17 * 10, Color::parse_color(&mut "AA".chars()).unwrap());
        assert_eq!(17 * 11, Color::parse_color(&mut "bb".chars()).unwrap());
        assert_eq!(17 * 11, Color::parse_color(&mut "BB".chars()).unwrap());
        assert_eq!(17 * 12, Color::parse_color(&mut "cc".chars()).unwrap());
        assert_eq!(17 * 12, Color::parse_color(&mut "CC".chars()).unwrap());
        assert_eq!(17 * 13, Color::parse_color(&mut "dd".chars()).unwrap());
        assert_eq!(17 * 13, Color::parse_color(&mut "DD".chars()).unwrap());
        assert_eq!(17 * 14, Color::parse_color(&mut "ee".chars()).unwrap());
        assert_eq!(17 * 14, Color::parse_color(&mut "EE".chars()).unwrap());
        assert_eq!(17 * 15, Color::parse_color(&mut "ff".chars()).unwrap());
        assert_eq!(17 * 15, Color::parse_color(&mut "FF".chars()).unwrap());
        // test a few mixed values
        assert_eq!(00,      Color::parse_color(&mut "00".chars()).unwrap());
        assert_eq!(50,      Color::parse_color(&mut "32".chars()).unwrap());
        assert_eq!(100,     Color::parse_color(&mut "64".chars()).unwrap());
        assert_eq!(150,     Color::parse_color(&mut "96".chars()).unwrap());
        assert_eq!(200,     Color::parse_color(&mut "c8".chars()).unwrap());
        assert_eq!(250,     Color::parse_color(&mut "fa".chars()).unwrap());
        assert_eq!(255,     Color::parse_color(&mut "ff".chars()).unwrap());
        // test invalid values
        assert_eq!(false,   Color::parse_color(&mut "".chars()).is_some());
        assert_eq!(false,   Color::parse_color(&mut "h".chars()).is_some());
        assert_eq!(false,   Color::parse_color(&mut "h2".chars()).is_some());
        assert_eq!(false,   Color::parse_color(&mut "yz".chars()).is_some());
        assert_eq!(false,   Color::parse_color(&mut "3x".chars()).is_some());
    }

    #[test]
    fn parse_rgb() {
        // test some valid color values
        let rgb_black = Color::parse_rgb("000000".to_string()).unwrap();
        assert_eq!(0,   rgb_black.red);
        assert_eq!(0,   rgb_black.green);
        assert_eq!(0,   rgb_black.blue);
        assert_eq!(255, rgb_black.alpha);
        let rgb_red   = Color::parse_rgb("ff0000".to_string()).unwrap();
        assert_eq!(0,   rgb_red.red);
        assert_eq!(0,   rgb_red.green);
        assert_eq!(255, rgb_red.blue);
        assert_eq!(255, rgb_red.alpha);
        let rgb_green = Color::parse_rgb("00ff00".to_string()).unwrap();
        assert_eq!(0,   rgb_green.red);
        assert_eq!(255, rgb_green.green);
        assert_eq!(0,   rgb_green.blue);
        assert_eq!(255, rgb_green.alpha);
        let rgb_blue  = Color::parse_rgb("0000ff".to_string()).unwrap();
        assert_eq!(255, rgb_blue.red);
        assert_eq!(0,   rgb_blue.green);
        assert_eq!(0,   rgb_blue.blue);
        assert_eq!(255, rgb_blue.alpha);
        let rgb_white = Color::parse_rgb("ffffff".to_string()).unwrap();
        assert_eq!(255, rgb_white.red);
        assert_eq!(255, rgb_white.green);
        assert_eq!(255, rgb_white.blue);
        assert_eq!(255, rgb_white.alpha);
        // test invalid formats
        assert_eq!(false, Color::parse_rgb("".to_string()).is_some());
        assert_eq!(false, Color::parse_rgb("0".to_string()).is_some());
        assert_eq!(false, Color::parse_rgb("00".to_string()).is_some());
        assert_eq!(false, Color::parse_rgb("000".to_string()).is_some());
        assert_eq!(false, Color::parse_rgb("0000".to_string()).is_some());
        assert_eq!(false, Color::parse_rgb("00000".to_string()).is_some());
        assert_eq!(false, Color::parse_rgb("xxxxxx".to_string()).is_some());
        assert_eq!(false, Color::parse_rgb("0000000".to_string()).is_some());
        assert_eq!(false, Color::parse_rgb("00000000".to_string()).is_some());
    }

    #[test]
    fn parse_argb() {
        // test some valid color values
        let rgb_transparent = Color::parse_argb("00000000".to_string()).unwrap();
        assert_eq!(0,   rgb_transparent.red);
        assert_eq!(0,   rgb_transparent.green);
        assert_eq!(0,   rgb_transparent.blue);
        assert_eq!(0,   rgb_transparent.alpha);
        let rgb_red   = Color::parse_argb("40ff0000".to_string()).unwrap();
        assert_eq!(0,   rgb_red.red);
        assert_eq!(0,   rgb_red.green);
        assert_eq!(255, rgb_red.blue);
        assert_eq!(64,  rgb_red.alpha);
        let rgb_green = Color::parse_argb("8000ff00".to_string()).unwrap();
        assert_eq!(0,   rgb_green.red);
        assert_eq!(255, rgb_green.green);
        assert_eq!(0,   rgb_green.blue);
        assert_eq!(128, rgb_green.alpha);
        let rgb_blue  = Color::parse_argb("c00000ff".to_string()).unwrap();
        assert_eq!(255, rgb_blue.red);
        assert_eq!(0,   rgb_blue.green);
        assert_eq!(0,   rgb_blue.blue);
        assert_eq!(192, rgb_blue.alpha);
        let rgb_white = Color::parse_argb("ffffffff".to_string()).unwrap();
        assert_eq!(255, rgb_white.red);
        assert_eq!(255, rgb_white.green);
        assert_eq!(255, rgb_white.blue);
        assert_eq!(255, rgb_white.alpha);
        // test some invalid formats
        assert_eq!(false, Color::parse_argb("".to_string()).is_some());
        assert_eq!(false, Color::parse_argb("0".to_string()).is_some());
        assert_eq!(false, Color::parse_argb("00".to_string()).is_some());
        assert_eq!(false, Color::parse_argb("000".to_string()).is_some());
        assert_eq!(false, Color::parse_argb("0000".to_string()).is_some());
        assert_eq!(false, Color::parse_argb("00000".to_string()).is_some());
        assert_eq!(false, Color::parse_argb("000000".to_string()).is_some());
        assert_eq!(false, Color::parse_argb("0000000".to_string()).is_some());
        assert_eq!(false, Color::parse_argb("xxxxxxxx".to_string()).is_some());
        assert_eq!(false, Color::parse_argb("000000000".to_string()).is_some());
        assert_eq!(false, Color::parse_argb("0000000000".to_string()).is_some());
    }

    #[test]
    fn parse() {
        // #-prefixed (HTML-style)
        assert_eq!(true, Color::parse("#000000".to_string()).is_some());
        assert_eq!(true, Color::parse("#00000000".to_string()).is_some());
        // 0x-prefixed (Hex-style)
        assert_eq!(true, Color::parse("0x000000".to_string()).is_some());
        assert_eq!(true, Color::parse("0x00000000".to_string()).is_some());
        // No prefix
        assert_eq!(true, Color::parse("000000".to_string()).is_some());
        assert_eq!(true, Color::parse("00000000".to_string()).is_some());
        // Actual colors
        let red = Color::parse("0xFFFF0000".to_string()).unwrap();
        assert_eq!(0,   red.red);
        assert_eq!(0,   red.green);
        assert_eq!(255, red.blue);
        assert_eq!(255, red.alpha);
        let green = Color::parse("0xFF00FF00".to_string()).unwrap();
        assert_eq!(0,   green.red);
        assert_eq!(255, green.green);
        assert_eq!(0,   green.blue);
        assert_eq!(255, green.alpha);
        let blue = Color::parse("0xFF0000FF".to_string()).unwrap();
        assert_eq!(255, blue.red);
        assert_eq!(0,   blue.green);
        assert_eq!(0,   blue.blue);
        assert_eq!(255, blue.alpha);
        // wrong formats
        assert_eq!(false, Color::parse("".to_string()).is_some());
        assert_eq!(false, Color::parse("0".to_string()).is_some());
        assert_eq!(false, Color::parse("00".to_string()).is_some());
        assert_eq!(false, Color::parse("000".to_string()).is_some());
        assert_eq!(false, Color::parse("0000".to_string()).is_some());
        assert_eq!(false, Color::parse("00000".to_string()).is_some());
        assert_eq!(false, Color::parse("0000000".to_string()).is_some());
    }

}

