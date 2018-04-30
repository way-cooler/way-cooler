mod output;
mod input;
mod seat;
mod cursor;
mod shells;
mod view;
mod xwayland;

pub use self::cursor::*;
pub use self::input::*;
pub use self::output::*;
pub use self::seat::*;
pub use self::shells::*;
pub use self::view::*;
pub use self::xwayland::*;

use wlroots::{self, Area, Compositor, CompositorBuilder, Cursor, CursorHandle, KeyboardHandle,
              OutputHandle, OutputLayout, OutputLayoutHandle, PointerHandle, XCursorTheme};

use awesome::SharedImage;

#[derive(Debug)]
pub struct Server {
    pub xcursor_theme: XCursorTheme,
    pub layout: OutputLayoutHandle,
    pub seat: Seat,
    pub cursor: CursorHandle,
    pub keyboards: Vec<KeyboardHandle>,
    pub pointers: Vec<PointerHandle>,
    pub outputs: Vec<OutputHandle>,
    pub views: Vec<View>,
    pub drawins: Vec<(SharedImage, Area)>
}

impl Default for Server {
    fn default() -> Server {
        let xcursor_theme =
            XCursorTheme::load_theme(None, 16).expect("Could not load default theme");
        Server { xcursor_theme,
                 layout: OutputLayoutHandle::default(),
                 seat: Seat::default(),
                 cursor: CursorHandle::default(),
                 keyboards: Vec::default(),
                 pointers: Vec::default(),
                 outputs: Vec::default(),
                 views: Vec::default(),
                 drawins: Vec::default() }
    }
}

impl Server {
    pub fn new(layout: OutputLayoutHandle, cursor: CursorHandle) -> Self {
        Server { layout,
                 cursor,
                 ..Server::default() }
    }
}

compositor_data!(Server);

pub fn init() -> Compositor {
    let layout = OutputLayout::create(Box::new(OutputLayoutManager::new()));
    let cursor = Cursor::create(Box::new(CursorManager::new()));
    let mut compositor = CompositorBuilder::new().gles2(true)
                                                 .data_device(true)
                                                 .output_manager(Box::new(OutputManager::new()))
                                                 .input_manager(Box::new(InputManager::new()))
                                                 .xwayland(Box::new(XWaylandManager::new()))
                                                 .xdg_shell_v6_manager(Box::new(XdgV6ShellManager))
                                                 .build_auto(Server::new(layout, cursor));
    // NOTE We need to create this afterwards because it needs the compositor
    // running to announce the seat.
    let seat = wlroots::Seat::create(&mut compositor,
                                     "seat0".into(),
                                     Box::new(SeatManager::new()));
    {
        let server: &mut Server = (&mut compositor).into();
        server.seat = Seat::new(seat);
    }
    compositor
}
