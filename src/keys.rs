//! Contains information for keybindings.

use std::collections::{HashMap, HashSet};
use std::sync::{RwLock};
use rustwlc::xkb::Keysym;
use rustwlc::types::*; // Need * for bitflags...
use std::hash::{Hash, Hasher};

lazy_static! {
    static ref BINDINGS: RwLock<HashMap<KeyPress, KeyEvent>> =
        RwLock::new(HashMap::new());
}

#[derive(Eq, PartialEq, Clone)]
pub struct KeyPress {
    modifiers: KeyMod,
    keys: Vec<Keysym>
}

/// Parses a KeyMod from key names.
pub fn keymod_from_names(keys: Vec<&str>) -> Result<KeyMod, String> {
    let mut result = KeyMod::empty();
    for key in keys {
        match key.to_lowercase().as_str() {
            "shift" => result = result | MOD_SHIFT,
            "control" | "ctrl" => result = result | MOD_CTRL,
            err => return Err(format!("Invalid character: {}", err))
        }
    }
    return Ok(result);
}

impl KeyPress {
    pub fn from_key_state() {
        
    }

    pub fn from_key_names(mods: Vec<String>, keys: Vec<String>) -> KeyPress {
        
    }
}

impl Hash for KeyPress {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        hasher.write_u32(self.modifiers.bits());
        for sym in self.keys {
            hasher.write_u32(sym.get_code());
        }
    }
}

/// The type of function a key press handler is.
pub type KeyEvent = Box<FnOnce() + Send + Sync>;

pub fn get(keys: KeyPress) -> Option<KeyEvent> {
    let mut val: Option<KeyEvent>;
    {
        let bindings = BINDINGS.read().unwrap();
        val = bindings.get(keys).cloned();
    }
    val
}

pub fn register(values: Vec<(KeyPress, KeyEvent)>) {
    let bindings = BINDINGS.write().unwrap();
    for value in values {
        bindings.set(value.0, value.1);
    }
}

pub fn register_keypress(press: KeyPress, func: KeyEvent) {
    
}
