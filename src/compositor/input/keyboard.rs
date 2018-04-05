use wlroots::{self, Compositor, KeyboardHandler, key_events::KeyEvent, xkbcommon::xkb::KEY_Escape};
pub struct Keyboard;

impl KeyboardHandler for Keyboard {
    fn on_key(&mut self,
              compositor: &mut Compositor,
              _: &mut wlroots::Keyboard,
              event: &mut KeyEvent) {
        for key in event.pressed_keys() {
            if key == KEY_Escape {
                compositor.terminate()
            }
        }
    }
}
