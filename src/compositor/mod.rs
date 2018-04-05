mod output;
mod input;
mod seat;
mod cursor;

pub use self::cursor::*;
pub use self::input::*;
pub use self::output::*;
pub use self::seat::*;
use wlroots::{Compositor, CompositorBuilder, Cursor, CursorHandle, KeyboardHandle, OutputLayout,
              OutputLayoutHandle, PointerHandle, Seat, SeatHandle, XCursorTheme};

#[derive(Debug)]
struct Server {
    xcursor_theme: XCursorTheme,
    layout: OutputLayoutHandle,
    seat: SeatHandle,
    cursor: CursorHandle,
    keyboards: Vec<KeyboardHandle>,
    pointers: Vec<PointerHandle>
}

impl Default for Server {
    fn default() -> Server {
        let xcursor_theme =
            XCursorTheme::load_theme(None, 16).expect("Could not load default theme");
        Server { xcursor_theme,
                 layout: OutputLayoutHandle::default(),
                 seat: SeatHandle::default(),
                 cursor: CursorHandle::default(),
                 keyboards: Vec::default(),
                 pointers: Vec::default() }
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
                                                 .build_auto(Server::new(layout, cursor));
    // NOTE We need to create this afterwards because it needs the compositor
    // running to announce the seat.
    let seat = Seat::create(&mut compositor,
                            "seat0".into(),
                            Box::new(SeatManager::new()));
    {
        let server: &mut Server = (&mut compositor).into();
        server.seat = seat;
    }
    compositor.run()
}
