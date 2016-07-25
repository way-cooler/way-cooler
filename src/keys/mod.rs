//! Contains information for keybindings.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::hash::{Hash, Hasher};

use rustwlc::xkb::{Keysym, NameFlags};
use rustwlc::types::*; // Need * for bitflags...

use super::commands::{self, CommandFn};

mod keypress;
pub use self::keypress::KeyPress;

mod event;
pub use self::event::KeyEvent;

lazy_static! {
    static ref BINDINGS: RwLock<HashMap<KeyPress, KeyEvent>> =
        RwLock::new(HashMap::new());

    static ref NAME_MAPPING: HashMap<&'static str, &'static str> = {
        let mut map = HashMap::new();
        map.insert("enter", "return");
        map.insert("\n", "return");
        map.insert("\t", "tab");
        map
    };
}


pub fn init() {
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

/// Get a key mapping from the list.
pub fn get(key: &KeyPress) -> Option<KeyEvent> {
    trace!("Searching for {}", key);
    let bindings = BINDINGS.read()
        .expect("Keybindings/get: unable to lock keybindings");
    bindings.get(key).map(KeyEvent::clone)
}

/// Register a new set of key mappings
pub fn register(values: Vec<(KeyPress, KeyEvent)>) {
    trace!("Registering some keypress: {}", values[0].0);
    let mut bindings = BINDINGS.write()
        .expect("Keybindings/register: unable to lock keybindings");
    for value in values {
        bindings.insert(value.0, value.1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn test_cmd() {
        assert!(true);
    }

    fn keypress() -> KeyPress {
        keypress!("Ctrl", "t")
    }

    #[test]
    fn add_key() {
        require_rustwlc!();
        register(vec![(keypress(), Arc::new(test_cmd))]);
        assert!(get(&keypress()).is_some(), "Key not registered");
    }
}
