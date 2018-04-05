// TODO

mod output;
mod input;

pub use self::output::*;
pub use self::input::*;
use wlroots::{Compositor, CompositorBuilder};

struct Server;

pub fn init() {
    CompositorBuilder::new().gles2(true)
                            .data_device(true)
                            .output_manager(Box::new(OutputManager::new()))
                            .input_manager(Box::new(InputManager::new()))
                            .build_auto(Server)
                            .run()
}
