use compositor::{self, Server};
use wlroots::{Compositor, InputManagerHandler, KeyboardHandler, Keyboard, Capability};

pub struct InputManager;

impl InputManager {
    pub fn new() -> Self {
        InputManager
    }
}


impl InputManagerHandler for InputManager {
    fn keyboard_added(&mut self, compositor: &mut Compositor, keyboard: &mut Keyboard)
                      -> Option<Box<KeyboardHandler>> {
        let server: &mut Server = compositor.into();
        run_handles!([(seat: {&mut server.seat})] => {
            let mut capabilities = seat.capabilities();
            capabilities.insert(Capability::Keyboard);
            seat.set_capabilities(capabilities);
        }).expect("Seat was not initialized");
        // TODO Remove the keyboard when removed, should be able to but have to patch wlroots-rs
        server.keyboards.push(keyboard.weak_reference());
        Some(Box::new(compositor::Keyboard))
    }
}
