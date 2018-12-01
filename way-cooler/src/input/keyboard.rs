use wlroots::{key_events::KeyEvent,
              wlroots_dehandle,
              xkbcommon::xkb::{KEY_Escape, KEY_Super_L, KEY_Super_R},
              Capability, CompositorHandle, KeyboardHandle, KeyboardHandler, WLR_KEY_PRESSED};

pub struct Keyboard;

fn key_is_meta(key: u32) -> bool {
    // TODO configure meta key
    key == KEY_Super_L || key == KEY_Super_R
}

impl KeyboardHandler for Keyboard {
    #[wlroots_dehandle(compositor, seat, keyboard)]
    fn on_key(&mut self,
              compositor_handle: CompositorHandle,
              keyboard_handle: KeyboardHandle,
              event: &KeyEvent)
    {
        use compositor_handle as compositor;
        if event.key_state() == WLR_KEY_PRESSED {
            for key in event.pressed_keys() {
                // TODO Keep it hardcoded, make this configurable by awesome
                if key == KEY_Escape {
                    ::wlroots::terminate();
                }
                if key_is_meta(key) {
                    let server: &mut ::Server = compositor.into();
                    server.seat.meta = true;
                }
            }
        } else {
            for key in event.pressed_keys() {
                if key_is_meta(key) {
                    let server: &mut ::Server = compositor.into();
                    server.seat.meta = false;
                }
            }
        };
        let server: &mut ::Server = compositor.into();
        let seat_handle = &server.seat.seat;
        use keyboard_handle as keyboard;
        use seat_handle as seat;
        seat.set_keyboard(keyboard.input_device());
        seat.keyboard_notify_key(event.time_msec(), event.keycode(), event.key_state() as u32);
        seat.keyboard_send_modifiers(&mut keyboard.get_modifier_masks());
        keyboard.get_modifiers();
        // TODO
        // LUA.with(|lua| {
        //             let lua = lua.borrow();
        //             if let Err(err) = emit_awesome_keybindings(&*lua, event, modifiers) {
        //                 warn!("Could not emit binding for {}: {:?}", event.keycode(), err);
        //             }
        //         });
    }

    #[wlroots_dehandle(compositor, seat, keyboard)]
    fn modifiers(&mut self, compositor_handle: CompositorHandle, keyboard_handle: KeyboardHandle) {
        use compositor_handle as compositor;
        let server: &mut ::Server = compositor.into();
        let seat_handle = &server.seat.seat;
        use keyboard_handle as keyboard;
        use seat_handle as seat;
        seat.keyboard_notify_modifiers(&mut keyboard.get_modifier_masks())
    }

    #[wlroots_dehandle(compositor, keyboard, seat)]
    fn destroyed(&mut self, compositor_handle: CompositorHandle, keyboard_handle: KeyboardHandle) {
        use compositor_handle as compositor;
        use keyboard_handle as keyboard;
        let server: &mut ::Server = compositor.into();
        let weak_reference = keyboard.weak_reference();
        server.keyboards.retain(|k| *k != weak_reference);
        if server.keyboards.is_empty() {
            let seat_handle = &server.seat.seat;
            use seat_handle as seat;
            let mut capabilities = seat.capabilities();
            capabilities.remove(Capability::Keyboard);
            seat.set_capabilities(capabilities);
        }
    }
}
