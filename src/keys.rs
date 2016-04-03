//! Contains information for keybindings.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use rustwlc::xkb::{Keysym, NameFlags};
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
    return Ok(result);
}

impl KeyPress {
    /// Creates a new KeyPress struct from a list of modifier and key names
    pub fn from_key_names(mods: Vec<&str>, keys: Vec<&str>) -> Result<KeyPress, String> {
        match keymod_from_names(mods) {
            Err(message) => Err(message),
            Ok(mods) => {
                let mut syms: Vec<Keysym> = Vec::with_capacity(keys.len());
                for name in keys {
                    if let Some(sym) = Keysym::from_name(name.to_string(),
                                                         NameFlags::None) {
                            syms.push(sym);
                    }
                    // Else if could not parse
                    else if let Some(sym) = Keysym::from_name(name.to_string(),
                                                        NameFlags::CaseInsensitive) {
                        syms.push(sym);
                    }
                    else {
                        return Err(format!("Invalid key: {}", name));
                    }
                }
                syms.sort_by_key(|s| s.get_code());
                return Ok(KeyPress { modifiers: mods, keys: syms });
            }
        }
    }
}

impl Hash for KeyPress {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        hasher.write_u32(self.modifiers.bits());
        for ref sym in &self.keys {
            hasher.write_u32(sym.get_code());
        }
    }
}

/// The type of function a key press handler is.
pub type KeyEvent = Arc<Box<FnOnce() + Send + Sync>>;

pub fn get(key: &KeyPress) -> Option<KeyEvent> {
    let bindings = BINDINGS.read().unwrap();
    match bindings.get(key) {
        None => None,
        Some(val) => Some(val.clone())
    }
}

pub fn register(values: Vec<(KeyPress, KeyEvent)>) {
    let mut bindings = BINDINGS.write().unwrap();
    for value in values {
        bindings.insert(value.0, value.1);
    }
}
