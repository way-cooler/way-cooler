//! Contains information for keybindings.

use std::collections::{HashMap, HashSet};
use std::sync::{RwLock};
use rustwlc::xkb::Keysym;
use rustwlc::types::{KeyState, KeyboardModifiers};
use std::hash::{Hash, Hasher};

lazy_static! {
    static ref BINDINGS: RwLock<HashMap<Keypress, KeyEvent>> =
        RwLock::new(HashMap::new());
}

#[derive(Eq, PartialEq, Hash, Clone)]
struct Keypress {
    modifiers: KeyboardModifiers,
    keys: Vec<Keysym>
}

/// The type of function a key press handler is.
pub type KeyEvent = Box<FnOnce() + Send + Sync>;

pub fn get(keys: &[Keysym]) -> Option<KeyEvent> {
    let mut val: Option<KeyEvent>;
    {
        let bindings = BINDINGS.read().unwrap();
        val = bindings.get(keys).cloned();
    }
    val
}

pub fn register(values: Vec<(&[Keysym], KeyEvent)>) {
    let bindings = BINDINGS.write().unwrap();
    for value in values {
        bindings.set(value.0, value.1);
    }
}
