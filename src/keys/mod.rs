//! Contains information for keybindings.

use std::collections::HashMap;
use std::sync::RwLock;

use rustwlc::types::*; // Need * for bitflags...

mod keypress;
mod action;
mod event;

pub use self::keypress::KeyPress;
pub use self::event::KeyEvent;
pub use self::action::Action;

lazy_static! {
    static ref BINDINGS: RwLock<HashMap<KeyPress, Action>> =
        RwLock::new(HashMap::new());

    static ref NAME_MAPPING: HashMap<&'static str, &'static str> = {
        let mut map = HashMap::new();
        map.insert("enter", "return");
        map.insert("\n", "return");
        map.insert("\t", "tab");
        map
    };

    static ref MOUSE_MODIFIER: RwLock<KeyMod> = RwLock::new(MOD_CTRL);
}

/// Parses a KeyMod from key names.
pub fn keymod_from_names(keys: &[&str]) -> Result<KeyMod, String> {
    let mut result = KeyMod::empty();
    for key in keys {
        match key.to_lowercase().as_str() {
            "shift"            => result = result | MOD_SHIFT,
            "control" | "ctrl" => result = result | MOD_CTRL,
            "alt"              => result = result | MOD_ALT,
            "mod2"             => result = result | MOD_MOD2,
            "mod3"             => result = result | MOD_MOD3,
            "mod4" | "super" | "logo" => result = result | MOD_MOD4,
            "mod5" | "5mod5me" => result = result | MOD_MOD5,
            err => return Err(format!("Invalid modifier: {}", err))
        }
    }
    return Ok(result)
}

#[allow(deprecated)] // keysyms
pub fn init() {
    use rustwlc::xkb::keysyms;
    use commands;
    if !is_quit_bound() {
        register(KeyPress::new(MOD_ALT | MOD_SHIFT, keysyms::KEY_Escape),
                 KeyEvent::Command(commands::get("way_cooler_quit")
                                   .expect("Error reading commands::way_cooler_quit")),
                 false);
    }
}

/// Clears all the keys from Way Cooler's memory.
pub fn clear_keys() {
    let mut bindings = BINDINGS.write()
        .expect("Keybindings/clear_keys: unable to lock keybindings");
    bindings.drain();
}

/// Get a key mapping from the list.
pub fn get(key: &KeyPress) -> Option<Action> {
    let bindings = BINDINGS.read()
        .expect("Keybindings/get: unable to lock keybindings");
    bindings.get(key).map(Action::clone)
}

/// Gets the current key modifier for mouse control
pub fn mouse_modifier() -> KeyMod {
    let key_mod = MOUSE_MODIFIER.read()
        .expect("Keybindings/register_mouse_modifier: unable to lock MOUSE MODIFIER");
    *key_mod
}

/// Register a new set of key mappings
pub fn register(key: KeyPress, event: KeyEvent, passthrough: bool)
                -> Option<Action> {
    let mut bindings = BINDINGS.write()
        .expect("Keybindings/register: unable to lock keybindings");
    let action = Action { event, passthrough };
    trace!("Registering {} for {:?}", key, action);
    bindings.insert(key, action)
}

/// Unregisters a key mapping
pub fn unregister(key: &KeyPress) -> Option<Action> {
    let mut bindings = BINDINGS.write()
        .expect("Keybindings/unregister: unable to lock keybindings");
    bindings.remove(key)
}

/// Registers a modifier to be used with mouse commands
pub fn register_mouse_modifier(modifier: KeyMod) {
    let mut key_mod = MOUSE_MODIFIER.write()
        .expect("Keybindings/register_mouse_modifier: unable to lock MOUSE MODIFIER");
    *key_mod = modifier;
}

/// Determine if the way_cooler_quit command is already bound
pub fn is_quit_bound() -> bool {
    use commands;

    let bindings = BINDINGS.read()
        .expect("Keybindings/get: unable to lock keybindings");
    let quit = commands::get("way_cooler_quit")
        .expect("Error reading commands::way_cooler_quit");

    for value in bindings.values() {
        let value = &value.event;
        if let &KeyEvent::Command(ref cmd) = value {
            if (&*cmd as *const _) == (&quit as *const _) {
                return true;
            }
        }
    };
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_cmd() {
        assert!(true);
    }

    fn keypress() -> KeyPress {
        keypress!("Ctrl", "t")
    }

    #[test]
    fn add_key() {
        require_rustwlc!();
        register(keypress(), KeyEvent::Command(test_cmd), false);
        assert!(get(&keypress()).is_some(), "Key not registered");
    }
}
