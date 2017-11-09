mod renderable;
mod draw;
mod color;
pub mod screen_scrape;

use cairo::{self, ImageSurface};
use gdk_pixbuf::Pixbuf;
pub use self::renderable::Renderable;
pub use self::draw::{Drawable, DrawErr, BaseDraw};
pub use self::color::Color;


/// Using a Pixbuf buffer, loads the data into a Cairo surface.
pub fn load_surface_from_pixbuf(pixbuf: Pixbuf) -> ImageSurface {
    let width = pixbuf.get_width();
    let height = pixbuf.get_height();
    let channels = pixbuf.get_n_channels();
    let pix_stride = pixbuf.get_rowstride() as usize;
    // NOTE This is safe because we aren't modifying the bytes, but there's no immutable view
    let pixels = unsafe { pixbuf.get_pixels() };
    let format = if channels == 3 {cairo::Format::Rgb24} else { cairo::Format::ARgb32};
    let mut surface = ImageSurface::create(format, width, height)
        .expect("Could not create image of that size");
    let cairo_stride = surface.get_stride() as usize;
    {
        let mut cairo_data = surface.get_data().unwrap();
        let mut pix_pixels_index = 0;
        let mut cairo_pixels_index = 0;
        for _ in 0..height {
            let mut pix_pixels_index2 = pix_pixels_index;
            let mut cairo_pixels_index2 = cairo_pixels_index;
            for _ in 0..width {
                if channels == 3 {
                    let r = pixels[pix_pixels_index2];
                    let g = pixels[pix_pixels_index2 + 1];
                    let b = pixels[pix_pixels_index2 + 2];
                    cairo_data[cairo_pixels_index2] = b;
                    cairo_data[cairo_pixels_index2 + 1] = g;
                    cairo_data[cairo_pixels_index2 + 2] = r;
                    pix_pixels_index2 += 3;
                    // NOTE Four because of the alpha value we ignore
                    cairo_pixels_index2 += 4;
                } else {
                    let mut r = pixels[pix_pixels_index];
                    let mut g = pixels[pix_pixels_index + 1];
                    let mut b = pixels[pix_pixels_index + 2];
                    let a = pixels[pix_pixels_index + 3];
                    let alpha = a as f64 / 255.0;
                    r *= alpha as u8;
                    g *= alpha as u8;
                    b *= alpha as u8;
                    cairo_data[cairo_pixels_index] = b;
                    cairo_data[cairo_pixels_index + 1] = g;
                    cairo_data[cairo_pixels_index + 2] = r;
                    cairo_data[cairo_pixels_index + 3] = a;
                    pix_pixels_index += 4;
                    cairo_pixels_index += 4;
                }
            }
            pix_pixels_index += pix_stride;
            cairo_pixels_index += cairo_stride;
        }
    }
    surface
}
