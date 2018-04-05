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
        }).expect("Seat was destroyed");
        server.keyboards.push(keyboard.weak_reference());
        Some(Box::new(compositor::Keyboard))
    }

    fn keyboard_removed(&mut self, compositor: &mut Compositor, keyboard: &mut Keyboard) {
        let server: &mut Server = compositor.into();
        let weak_reference = keyboard.weak_reference();
        let index = server.keyboards.iter().position(|k| *k == weak_reference);
        if let Some(index) = index {
            server.keyboards.remove(index);
            if server.keyboards.len() == 0 {
                run_handles!([(seat: {&mut server.seat})] => {
                    let mut capabilities = seat.capabilities();
                    capabilities.remove(Capability::Keyboard);
                    seat.set_capabilities(capabilities);
                }).expect("Seat was destroyed")
            }
        }
    }
}
