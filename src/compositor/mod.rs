// TODO

mod output;
mod input;
mod seat;

pub use self::input::*;
pub use self::output::*;
pub use self::seat::*;
use wlroots::{Compositor, CompositorBuilder, KeyboardHandle, PointerHandle, Seat, SeatHandle};

#[derive(Debug, Default)]
struct Server {
    // TODO Support multiple seats
    seat: SeatHandle,
    keyboards: Vec<KeyboardHandle>,
    pointers: Vec<PointerHandle>
}

impl Server {
    pub fn new() -> Self {
        Server { ..Server::default() }
    }
}

compositor_data!(Server);

pub fn init() {
    let mut compositor = CompositorBuilder::new().gles2(true)
                                                 .data_device(true)
                                                 .output_manager(Box::new(OutputManager::new()))
                                                 .input_manager(Box::new(InputManager::new()))
                                                 .build_auto(Server::new());
    {
        let seat = Seat::create(&mut compositor,
                                "Main Seat".into(),
                                Box::new(SeatManager::new()));
        let server: &mut Server = (&mut compositor).into();
        server.seat = seat;
    }
    compositor.run()
}
