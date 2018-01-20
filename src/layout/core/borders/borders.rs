use std::cmp::{Eq, PartialEq};
use rustwlc::{Geometry, Size, WlcOutput};
use rustwlc::render::{calculate_stride};
use cairo::{ImageSurface, Format};

use uuid::Uuid;
use ::registry;
use ::render::{Color, Renderable};
use super::super::container::Layout;

/// Data of the container's children, necessary to draw Tabbed and Stacked layouts
#[derive(Clone, Debug)]
pub struct Children {
    pub titles: Vec<String>,
    /// Index of the currently selected children
    pub index: Option<usize>,
}

/// The borders of a container.
///
/// This type just deals with rendering,
#[derive(Clone, Debug)]
pub struct Borders {
    /// The title displayed in the title border.
    pub title: String,
    /// The surface that contains the bytes we give to wlc to draw.
    surface: ImageSurface,
    /// Children titles to be used tabbed/stacked layouts
    pub children: Option<Children>,
    /// Layout of the container, only used if the border is from a container
    pub layout: Option<Layout>,
    /// The geometry where the buffer is written.
    ///
    /// Should correspond with the geometry of the container.
    pub geometry: Geometry,
    /// The output where the buffer is written to.
    output: WlcOutput,
    /// The specific color these borders should be colored.
    ///
    /// If unspecified, the default is used.
    color: Option<Color>,
    /// The specific color the title bar should be colored.
    ///
    /// If unspecified, the default is used.
    title_color: Option<Color>,
    /// The specific color the font for the title bar should be colored.
    ///
    /// If unspecified, the default is used.
    title_font_color: Option<Color>,
    /// Specifies if we should draw the title or not
    pub draw_title: bool
}

impl Renderable for Borders {
    fn new(mut geometry: Geometry, output: WlcOutput) -> Option<Self> {
        let thickness = Borders::thickness();
        let title_size = Borders::fetch_title_bar_size();
        if thickness == 0 {
            return None
        }
        // Add the thickness to the geometry.
        geometry.origin.x -= thickness as i32;
        geometry.origin.y -= thickness as i32;
        geometry.origin.y -= title_size as i32;
        geometry.size.w += thickness;
        geometry.size.h += thickness;
        geometry.size.h += title_size;
        let Size { w, h } = geometry.size;
        let stride = calculate_stride(w) as i32;
        let data: Vec<u8> = vec![0; h as usize * stride as usize];
        let buffer = data.into_boxed_slice();
        let surface = ImageSurface::create_for_data(buffer,
                                                    drop_data,
                                                    Format::ARgb32,
                                                    w as i32,
                                                    h as i32,
                                                    stride)
            .expect("Could not make ImageSurface");
        Some(Borders {
            title: "".into(),
            surface: surface,
            children: None,
            layout: None,
            geometry: geometry,
            output: output,
            color: None,
            title_color: None,
            title_font_color: None,
            draw_title: true
        })
    }

    fn get_surface(&mut self) -> &mut ImageSurface {
        &mut self.surface
    }

    fn get_geometry(&self) -> Geometry {
        self.geometry
    }

    fn set_geometry(&mut self, geometry: Geometry) {
        self.geometry = geometry;
    }

    fn get_output(&self) -> WlcOutput {
        self.output
    }

    /// Updates/Creates the underlying geometry for the surface/buffer.
    ///
    /// This causes a reallocation of the buffer, do not call this
    /// in a tight loop unless you want memory fragmentation and
    /// bad performance.
    fn reallocate_buffer(mut self, mut geometry: Geometry) -> Option<Self>{
        // Add the thickness to the geometry.
        let thickness = Borders::thickness();

        let title_count = match (self.layout, self.children.as_ref()) {
            (Some(Layout::Stacked), Some(children)) =>
                // I don't understand why I have to use this
                // instead of just child_count, but seems to work...
                (2 * children.titles.len()).saturating_sub(1) as u32,
            _ => 1
        };
        let title_size = self.title_bar_size() * title_count;

        if thickness == 0 {
            return None;
        }
        geometry.origin.x -= thickness as i32;
        geometry.origin.y -= thickness as i32;
        geometry.origin.y -= title_size as i32;
        geometry.size.w += thickness;
        geometry.size.h += thickness;
        geometry.size.h += title_size;
        let output_res = self.output.get_resolution()
            .expect("Could not get output's resolution");

        // Need to do geometry check here, because it's possible we'll allocate
        // a buffer that is the right size to hold the data, but ultimately
        // will be too big to be rendered, which will cause an ugly wrap-around
        if geometry.origin.x + geometry.size.w as i32 > output_res.w as i32 {
            let offset = (geometry.origin.x + geometry.size.w as i32) - output_res.w as i32;
            geometry.origin.x -= offset as i32;
        }
        if geometry.origin.y + geometry.size.h as i32 > output_res.h as i32 {
            let offset = (geometry.origin.y + geometry.size.h as i32) - output_res.h as i32;
            geometry.origin.y -= offset as i32;
        }
        if geometry.origin.x < 0 {
            geometry.origin.x = 0;
        }
        if geometry.origin.y < 0 {
            geometry.origin.y = 0;
        }
        if geometry.size.w > output_res.w {
            geometry.size.w = output_res.w;
        }
        if geometry.size.h > output_res.h {
            geometry.size.h = output_res.h;
        }

        let Size { w, h } = geometry.size;
        if w == self.geometry.size.w && h == self.geometry.size.h {
            return Some(self);
        }
        let stride = calculate_stride(w) as i32;
        let data: Vec<u8> = vec![0; h as usize * stride as usize];
        let buffer = data.into_boxed_slice();
        let surface = ImageSurface::create_for_data(buffer,
                                                    drop_data,
                                                    Format::ARgb32,
                                                    w as i32,
                                                    h as i32,
                                                    stride)
            .expect("Could not create ImageSurface");
        self.geometry = geometry;
        self.surface = surface;
        Some(self)
    }
}

impl Borders {
    /// Gets the gap size
    pub fn gap_size() -> u32 {
        let lock = registry::clients_read();
        let client = lock.client(Uuid::nil()).unwrap();
        let handle = registry::ReadHandle::new(&client);
        handle.read("windows".into()).ok()
            .and_then(|windows|windows.get("gaps".into()))
            .and_then(|gaps| gaps.as_object()
                      .and_then(|gaps| gaps.get("size"))
                      .and_then(|gaps| gaps.as_f64()))
            .map(|num| num as u32)
            .unwrap_or(0u32)
    }


    /// Gets the thickness of the borders (not including title bar).
    ///
    /// Defaults to 0 if not set.
    pub fn thickness() -> u32 {
        let lock = registry::clients_read();
        let client = lock.client(Uuid::nil()).unwrap();
        let handle = registry::ReadHandle::new(&client);
        handle.read("windows".into()).ok()
            .and_then(|windows| windows.get("borders".into()))
            .and_then(|borders| borders.as_object()
                      .and_then(|borders| borders.get("size"))
                      .and_then(|gaps| gaps.as_f64()))
            .map(|num| num as u32)
            .unwrap_or(0u32)
    }

    /// Gets the size of the title bar.
    ///
    /// If the view doesn't want to display it, returns 0.
    ///
    /// Defaults to 0 if not set.
    pub fn title_bar_size(&self) -> u32 {
        if !self.draw_title {
            0
        } else {
            Borders::fetch_title_bar_size()
        }
    }

    /// Gets the size of the title bar.
    ///
    /// Defaults to 0 if not set.
    fn fetch_title_bar_size() -> u32 {
        let lock = registry::clients_read();
        let client = lock.client(Uuid::nil()).unwrap();
        let handle = registry::ReadHandle::new(&client);
        handle.read("windows".into()).ok()
            .and_then(|windows| windows.get("title_bar".into()))
            .and_then(|title_bar| title_bar.as_object()
                      .and_then(|title_bar| title_bar.get("size"))
                      .and_then(|size| size.as_f64()))
            .map(|num| num as u32)
            .unwrap_or(0u32).into()
    }

    /// Fetches the default color from the registry.
    ///
    /// If the value is unset, black borders are returned.
    pub fn default_color() -> Color {
        let lock = registry::clients_read();
        let client = lock.client(Uuid::nil()).unwrap();
        let handle = registry::ReadHandle::new(&client);
        handle.read("windows".into()).ok()
            .and_then(|windows| windows.get("borders".into()))
            .and_then(|borders| borders.as_object()
                      .and_then(|borders| borders.get("inactive_color"))
                      .and_then(|gaps| gaps.as_string()))
            .and_then(|s| Color::parse(s))
            .unwrap_or(0u32.into())
    }

    /// Gets the active border color, if one is set.
    pub fn active_color() -> Option<Color> {
        let lock = registry::clients_read();
        let client = lock.client(Uuid::nil()).unwrap();
        let handle = registry::ReadHandle::new(&client);
        handle.read("windows".into()).ok()
            .and_then(|windows| windows.get("borders".into()))
            .and_then(|borders| borders.as_object()
                      .and_then(|borders| borders.get("active_color"))
                      .and_then(|gaps| gaps.as_string()))
            .and_then(|s| Color::parse(s))
    }

    /// Construct root borders, if the option is enabled.
    pub fn make_root_borders(geo: Geometry, output: WlcOutput)
                             -> Option<Borders> {
        let lock = registry::clients_read();
        let client = lock.client(Uuid::nil()).unwrap();
        let handle = registry::ReadHandle::new(&client);
        let root_borders_on = handle.read("windows".into()).ok()
            .and_then(|windows| windows.get("borders".into()))
            .and_then(|borders| borders.as_object()
                      .and_then(|borders| borders.get("root_borders"))
                      .and_then(|active| active.as_boolean())
            ).unwrap_or(true);
        if root_borders_on {
            Borders::new(geo, output)
        } else {
            None
        }
    }

    /// Fetches the default title background color from the registry.
    ///
    /// If the value is unset, black borders are returned.
    pub fn default_title_color() -> Color {
        let lock = registry::clients_read();
        let client = lock.client(Uuid::nil()).unwrap();
        let handle = registry::ReadHandle::new(&client);
        handle.read("windows".into()).ok()
            .and_then(|windows| windows.get("title_bar"))
            .and_then(|title_bar| title_bar.as_object()
                      .and_then(|title_bar| title_bar.get("background_color"))
                      .and_then(|color| color.as_string()))
            .and_then(|s| Color::parse(s))
            .unwrap_or(0u32.into())
    }

    /// Gets the active border color, if one is set
    pub fn active_title_color() -> Option<Color> {
        let lock = registry::clients_read();
        let client = lock.client(Uuid::nil()).unwrap();
        let handle = registry::ReadHandle::new(&client);
        handle.read("windows".into()).ok()
            .and_then(|windows| windows.get("title_bar"))
            .and_then(|title_bar| title_bar.as_object()
                      .and_then(|title_bar| title_bar.get("active_background_color"))
                      .and_then(|color| color.as_string()))
            .and_then(|s| Color::parse(s))
    }

    /// Fetches the default title font color from the registry.
    ///
    /// If the value is unset, white font are returned.
    pub fn default_title_font_color() -> Color {
        let lock = registry::clients_read();
        let client = lock.client(Uuid::nil()).unwrap();
        let handle = registry::ReadHandle::new(&client);
        handle.read("windows".into()).ok()
            .and_then(|windows| windows.get("title_bar"))
            .and_then(|title_bar| title_bar.as_object()
                      .and_then(|title_bar| title_bar.get("font_color"))
                      .and_then(|color| color.as_string()))
            .and_then(|s| Color::parse(s))
            .unwrap_or(0xffffff.into())
    }

    /// Gets the active title border font, if one is set
    pub fn active_title_font_color() -> Option<Color> {
        let lock = registry::clients_read();
        let client = lock.client(Uuid::nil()).unwrap();
        let handle = registry::ReadHandle::new(&client);
        handle.read("windows".into()).ok()
            .and_then(|windows| windows.get("title_bar"))
            .and_then(|title_bar| title_bar.as_object()
                      .and_then(|title_bar| title_bar.get("active_font_color"))
                      .and_then(|color| color.as_string()))
            .and_then(|s| Color::parse(s))
    }

    /// Gets the color for these borders.
    ///
    /// If a specific one is unset, then the default color is returned.
    pub fn color(&self) -> Color {
        self.color.unwrap_or_else(Borders::default_color)
    }

    /// Gets the color for the title border of these borders.
    ///
    /// If a specific one is unset, then the default color is returned.
    pub fn title_background_color(&self) -> Color {
        self.title_color.unwrap_or_else(Borders::default_title_color)
    }

    /// Gets the color for the title font of these borders.
    ///
    /// If a specific one is unset, then the default color is returned.
    pub fn title_font_color(&self) -> Color {
        self.title_font_color.unwrap_or_else(Borders::default_title_font_color)
    }

    /// Sets or clears the specific color for these borders.
    pub fn set_color(&mut self, color: Option<Color>) {
        self.color = color
    }

    /// Sets or clears the specific color for these borders.
    pub fn set_title_color(&mut self, color: Option<Color>) {
        self.title_color = color
    }

    /// Sets or clears the specific color for these borders.
    pub fn set_title_font_color(&mut self, color: Option<Color>) {
        self.title_font_color = color
    }

    pub fn get_output(&self) -> WlcOutput {
        self.output
    }

    /// Changes the output that this border resides in.
    /// This will automatically re-trigger a reallocation
    /// so that it renders correctly.
    pub fn set_output(mut self, output: WlcOutput) -> Option<Self> {
        self.output = output;
        // Force copy
        let geo = self.geometry;
        self.reallocate_buffer(geo)
    }

    pub fn title(&self) -> &str {
        self.title.as_str()
    }

    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    pub fn set_children(&mut self, titles: Vec<String>, index: Option<usize>) {
        self.children = Some(Children{
            titles: titles,
            index: index
        });
    }
}

impl PartialEq for Borders {
    fn eq(&self, other: &Borders) -> bool {
        self.geometry == other.geometry
    }
}

impl Eq for Borders {}

unsafe impl Send for Borders {}
unsafe impl Sync for Borders {}

#[allow(dead_code)]
fn drop_data(_: Box<[u8]>) { }
