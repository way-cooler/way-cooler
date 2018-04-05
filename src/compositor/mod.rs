// TODO

mod output;
mod input;

pub use self::output::*;
pub use self::input::*;
use wlroots::{Compositor, CompositorBuilder, KeyboardHandle, PointerHandle};

#[derive(Debug, Default)]
struct Server {
    keyboards: Vec<KeyboardHandle>,
    pointers: Vec<PointerHandle>,
}

impl Server {
    pub fn new() -> Self {
        Server { ..Server::default() }
    }
}

compositor_data!(Server);

pub fn init() {
    CompositorBuilder::new().gles2(true)
                            .data_device(true)
                            .output_manager(Box::new(OutputManager::new()))
                            .input_manager(Box::new(InputManager::new()))
                            .build_auto(Server::new())
                            .run()
}
