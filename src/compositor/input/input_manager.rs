use compositor::{self, Server};
use wlroots::{Capability, Compositor, InputManagerHandler, Keyboard, KeyboardHandler, Pointer,
              PointerHandler};

pub struct InputManager;

impl InputManager {
    pub fn new() -> Self {
        InputManager
    }
}

impl InputManagerHandler for InputManager {
    fn keyboard_added(&mut self,
                      compositor: &mut Compositor,
                      keyboard: &mut Keyboard)
                      -> Option<Box<KeyboardHandler>> {
        let server: &mut Server = compositor.into();
        server.keyboards.push(keyboard.weak_reference());
        if server.keyboards.len() == 1 {
            // Now that we have at least one keyboard, update the seat capabilities.
            run_handles!([(seat: {&mut server.seat})] => {
                let mut capabilities = seat.capabilities();
                capabilities.insert(Capability::Keyboard);
                seat.set_capabilities(capabilities);
                seat.set_keyboard(keyboard.input_device());
            }).expect("Seat was destroyed");
        }
        Some(Box::new(compositor::Keyboard))
    }

    fn pointer_added(&mut self,
                     compositor: &mut Compositor,
                     pointer: &mut Pointer)
                     -> Option<Box<PointerHandler>> {
        let server: &mut Server = compositor.into();
        server.pointers.push(pointer.weak_reference());
        if server.pointers.len() == 1 {
            // Now that we have at least one keyboard, update the seat capabilities.
            run_handles!([(seat: {&mut server.seat})] => {
                let mut capabilities = seat.capabilities();
                capabilities.insert(Capability::Pointer);
                seat.set_capabilities(capabilities);
            }).expect("Seat was destroyed");
        }

        run_handles!([(cursor: {&mut server.cursor})] => {
            cursor.attach_input_device(pointer.input_device());
        }).expect("Cursor was destroyed");
        Some(Box::new(compositor::Pointer))
    }

    fn keyboard_removed(&mut self, compositor: &mut Compositor, keyboard: &mut Keyboard) {
        let server: &mut Server = compositor.into();
        let weak_reference = keyboard.weak_reference();
        if let Some(index) = server.keyboards.iter().position(|k| *k == weak_reference) {
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

    fn pointer_removed(&mut self, compositor: &mut Compositor, pointer: &mut Pointer) {
        let server: &mut Server = compositor.into();
        let weak_reference = pointer.weak_reference();
        if let Some(index) = server.pointers.iter().position(|p| *p == weak_reference) {
            server.pointers.remove(index);
            if server.pointers.len() == 0 {
                run_handles!([(seat: {&mut server.seat})] => {
                    let mut capabilities = seat.capabilities();
                    capabilities.remove(Capability::Pointer);
                    seat.set_capabilities(capabilities);
                }).expect("Seat was destroyed")
            }
        }
        // TODO Double check this isn't a safety hole actually,
        // because if it isn't then we may not have to do this here...
        run_handles!([(cursor: {&mut server.cursor})] => {
            cursor.deattach_input_device(pointer.input_device());
        }).expect("Cursor was destroyed");
    }
}
