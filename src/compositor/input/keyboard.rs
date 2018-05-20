use rlua;
use wlroots::{key_events::KeyEvent,
              xkbcommon::xkb::{KEY_Escape, KEY_Super_L, KEY_Super_R}, CompositorHandle,
              KeyboardHandle, KeyboardHandler, WLR_KEY_PRESSED};

use awesome::{self, emit_object_signal, Objectable, LUA, ROOT_KEYS_HANDLE};
use compositor::Server;

pub struct Keyboard;

fn key_is_meta(key: u32) -> bool {
    // TODO configure meta key
    key == KEY_Super_L || key == KEY_Super_R
}

impl KeyboardHandler for Keyboard {
    fn on_key(&mut self,
              compositor: CompositorHandle,
              keyboard: KeyboardHandle,
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
            with_handles!([(seat: {&mut server.seat.seat}),
                           (keyboard: {keyboard})] => {
                seat.keyboard_notify_key(event.time_msec(),
                                         event.keycode(),
                                         event.key_state() as u32);
                seat.keyboard_send_modifiers(&mut keyboard.get_modifier_masks());
                let modifiers = keyboard.get_modifiers();
                LUA.with(|lua| {
                    let lua = lua.borrow_mut();
                    let state_string = if event.key_state() == WLR_KEY_PRESSED {
                        "press"
                    } else {
                        "release"
                    };
                    // TODO Awesome uses xcb_key_symbols_get_keysym,
                    // are we using the right function?


                    // TODO Should also emit by current focused client so we can
                    // do client based rules.

                    // TODO This should really be a hash map instead.

                    // TODO Error handling
                    let keybindings = lua.named_registry_value::<Vec<rlua::AnyUserData>>
                        (ROOT_KEYS_HANDLE).unwrap();
                    for keysym in event.pressed_keys() {
                        for binding in &keybindings {
                            let obj: awesome::Object = binding.clone().into();
                            let key = awesome::Key::cast(obj.clone()).unwrap();
                            if key.keysym().unwrap() == keysym &&
                                key.modifiers().unwrap() == modifiers.bits() {
                                    emit_object_signal(&*lua,
                                                       obj,
                                                       state_string.into(),
                                                       keysym).unwrap();
                            }
                        }
                    }
                });
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
