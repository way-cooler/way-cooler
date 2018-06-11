use rlua::{self, Lua};
use wlroots::{key_events::KeyEvent, xkbcommon::xkb::{KEY_Escape, KEY_Super_L, KEY_Super_R},
              Capability, CompositorHandle, KeyboardHandle, KeyboardHandler, KeyboardModifier,
              WLR_KEY_PRESSED};

use awesome::{self, emit_object_signal, Objectable, LUA, ROOT_KEYS_HANDLE};
use compositor::Server;

pub struct Keyboard;

fn key_is_meta(key: u32) -> bool {
    // TODO configure meta key
    key == KEY_Super_L || key == KEY_Super_R
}

impl KeyboardHandler for Keyboard {
    fn on_key(&mut self, compositor: CompositorHandle, keyboard: KeyboardHandle, event: &KeyEvent) {
        let modifiers = with_handles!([(compositor: {compositor})] => {
            if event.key_state() == WLR_KEY_PRESSED {
                for key in event.pressed_keys() {
                    if key == KEY_Escape {
                        // NOTE No need to call awesome::lua::terminate.
                        // that will be handled by wlroots.
                        ::wlroots::terminate();
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
                seat.set_keyboard(keyboard.input_device());
                seat.keyboard_notify_key(event.time_msec(),
                                         event.keycode(),
                                         event.key_state() as u32);
                seat.keyboard_send_modifiers(&mut keyboard.get_modifier_masks());
                keyboard.get_modifiers()
            }).expect("Seat was destroyed")
        }).unwrap();
        LUA.with(|lua| {
                     let lua = lua.borrow();
                     if let Err(err) = emit_awesome_keybindings(&*lua, event, modifiers) {
                         warn!("Could not emit binding for {}: {:?}", event.keycode(), err);
                     }
                 });
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

    fn destroyed(&mut self, compositor: CompositorHandle, keyboard: KeyboardHandle) {
        with_handles!([(compositor: {compositor}), (keyboard: {keyboard})] => {
            let server: &mut Server = compositor.into();
            let weak_reference = keyboard.weak_reference();
            if let Some(index) = server.keyboards.iter().position(|k| *k == weak_reference) {
                server.keyboards.remove(index);
                if server.keyboards.len() == 0 {
                    with_handles!([(seat: {&mut server.seat.seat})] => {
                        let mut capabilities = seat.capabilities();
                        capabilities.remove(Capability::Keyboard);
                        seat.set_capabilities(capabilities);
                    }).expect("Seat was destroyed")
                }
            }
        }).unwrap();
    }
}

/// Emits the Awesome keybindinsg.
fn emit_awesome_keybindings(lua: &Lua,
                            event: &KeyEvent,
                            event_modifiers: KeyboardModifier)
                            -> rlua::Result<()> {
    let state_string = if event.key_state() == WLR_KEY_PRESSED {
        "press"
    } else {
        "release"
    };
    // TODO Should also emit by current focused client so we can
    // do client based rules.
    let keybindings = lua.named_registry_value::<Vec<rlua::AnyUserData>>(ROOT_KEYS_HANDLE)?;
    for event_keysym in event.pressed_keys() {
        for binding in &keybindings {
            let obj: awesome::Object = binding.clone().into();
            let key = awesome::Key::cast(obj.clone()).unwrap();
            let keycode = key.keycode()?;
            let keysym = key.keysym()?;
            let modifiers = key.modifiers()?;
            let binding_match = (keysym != 0 && keysym == event_keysym
                                 || keycode != 0 && keycode == event.keycode())
                                && modifiers == 0
                                || modifiers == event_modifiers.bits();
            if binding_match {
                emit_object_signal(&*lua, obj, state_string.into(), event_keysym)?;
            }
        }
    }
    Ok(())
}
