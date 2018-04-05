mod output;
mod input;
mod seat;
mod cursor;
mod shells;
mod view;

pub use self::cursor::*;
pub use self::input::*;
pub use self::output::*;
pub use self::seat::*;
pub use self::shells::*;
pub use self::view::*;

use wlroots::{self, Compositor, CompositorBuilder, Cursor, CursorHandle, KeyboardHandle,
              OutputLayout, OutputLayoutHandle, PointerHandle, SeatHandle, XCursorTheme};

#[derive(Debug)]
struct Server {
    xcursor_theme: XCursorTheme,
    layout: OutputLayoutHandle,
    seat: Seat,
    cursor: CursorHandle,
    keyboards: Vec<KeyboardHandle>,
    pointers: Vec<PointerHandle>,
    views: Vec<View>
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
                 views: Vec::default() }
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

pub fn init() {
    let layout = OutputLayout::create(Box::new(OutputLayoutManager::new()));
    let cursor = Cursor::create(Box::new(CursorManager::new()));
    let mut compositor = CompositorBuilder::new().gles2(true)
                                                 .data_device(true)
                                                 .output_manager(Box::new(OutputManager::new()))
                                                 .input_manager(Box::new(InputManager::new()))
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
    compositor.run()
}
