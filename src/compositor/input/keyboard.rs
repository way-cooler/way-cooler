use compositor::Server;
use wlroots::{key_events::KeyEvent,
              xkbcommon::xkb::{KEY_Escape, KEY_Super_L, KEY_Super_R}, CompositorHandle,
              KeyboardHandle, KeyboardHandler, WLR_KEY_PRESSED};
pub struct Keyboard;

fn key_is_meta(key: u32) -> bool {
    // TODO configure meta key
    key == KEY_Super_L || key == KEY_Super_R
}

impl KeyboardHandler for Keyboard {
    fn on_key(&mut self,
              compositor: CompositorHandle,
              _keyboard: KeyboardHandle,
              event: &KeyEvent) {
        with_handles!([(compositor: {compositor})] => {
            if event.key_state() == WLR_KEY_PRESSED {
                for key in event.pressed_keys() {
                    if key == KEY_Escape {
                        compositor.terminate();
                        ::awesome::lua::terminate();
                    }
                    if key_is_meta(key) {
                        let server: &mut Server = compositor.into();
                        server.seat.meta = true;
                    }
                }
            } else {
                for key in event.pressed_keys() {
                    if key_is_meta(key) {
                        let server: &mut Server = compositor.into();
                        server.seat.meta = false;
                    }
                }
            }
            let server: &mut Server = compositor.into();
            with_handles!([(seat: {&mut server.seat.seat})] => {
                seat.keyboard_notify_key(event.time_msec(),
                                         event.keycode(),
                                         event.key_state() as u32);
            }).expect("Seat was destroyed");
        }).unwrap();
    }

    fn modifiers(&mut self, compositor: CompositorHandle, keyboard: KeyboardHandle) {
        with_handles!([(compositor: {compositor})] => {
            let server: &mut Server = compositor.into();
            with_handles!([(seat: {&mut server.seat.seat}),
                           (keyboard: {keyboard})] => {
                seat.keyboard_notify_modifiers(&mut keyboard.get_modifier_masks());
            }).unwrap();
        }).unwrap();
    }
}
