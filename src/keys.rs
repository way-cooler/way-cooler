//! Contains information for keybindings.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::hash::{Hash, Hasher};

use rustwlc::xkb::{Keysym, NameFlags};
use rustwlc::types::*; // Need * for bitflags...

use super::commands;

lazy_static! {
    static ref BINDINGS: RwLock<HashMap<KeyPress, KeyEvent>> =
        RwLock::new(HashMap::new());
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct KeyPress {
    modifiers: KeyMod,
    keys: Vec<Keysym>
}

pub fn init() {
    macro_rules! insert_all {
        ( $($press:expr => $name:expr);+ ) => {
            register(vec![
                $( ($press, commands::get(&$name.to_string())
                  .expect("Unable to register default command!")) ),+
                ]);
        }
    }

    insert_all! {
        keypress!("Alt", "Escape") => "quit";
        keypress!("Alt", "Return") => "launch_terminal";
        keypress!("Alt", "d") => "launch_dmenu";
        keypress!("Alt", "l") => "dmenu_eval";
        KeyPress::from_key_names(vec!["Alt", "Shift"], vec!["l"])
            .expect("Unable to create default keypress")
            => "dmenu_lua_dofile";

        /* Layout key bindings */
        keypress!("Alt", "e") => "horizontal_vertical_switch";
        keypress!("Alt", "v") => "split_vertical";
        keypress!("Alt", "h") => "split_horizontal";
        /* Focus key bindings */
        keypress!("Alt", "left") => "focus_left";
        keypress!("Alt", "right") => "focus_right";
        keypress!("Alt", "up") => "focus_up";
        keypress!("Alt", "down") => "focus_down";

        /* Workspace switching key bindings */
        keypress!("Alt", "1") => "switch_workspace_1";
        keypress!("Alt", "2") => "switch_workspace_2";
        keypress!("Alt", "3") => "switch_workspace_3";
        keypress!("Alt", "4") => "switch_workspace_4";
        keypress!("Alt", "5") => "switch_workspace_5";
        keypress!("Alt", "6") => "switch_workspace_6";
        keypress!("Alt", "7") => "switch_workspace_7";
        keypress!("Alt", "8") => "switch_workspace_8";
        keypress!("Alt", "9") => "switch_workspace_9";
        keypress!("Alt", "0") => "switch_workspace_0"
    }
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
        keymod_from_names(mods).and_then(|mods| {
            let mut syms: Vec<Keysym> = Vec::with_capacity(keys.len());
            for name in keys {
                // Parse a keysym for each given key
                if let Some(sym) = Keysym::from_name(name.to_string(),
                                                     NameFlags::None) {
                    syms.push(sym);
                }
                // If lowercase cannot be parsed, try case insensitive
                else if let Some(sym) = Keysym::from_name(name.to_string(),
                                                          NameFlags::CaseInsensitive) {
                    syms.push(sym);
                }
                else {
                    return Err(format!("Invalid key: {}", name));
                }
            }
            // Sort and dedup to make sure hashes are the same
            syms.sort_by_key(|s| s.get_code());
            syms.dedup();
            return Ok(KeyPress { modifiers: mods, keys: syms });
        })
    }

    /// Creates a KeyPress from keys that are pressed at the moment
    pub fn new(mods: KeyMod, mut keys: Vec<Keysym>) -> KeyPress {
        // Sort and dedup to make sure hashes are the same
        keys.sort_by_key(|k| k.get_code());
        keys.dedup();

        KeyPress { modifiers: mods, keys: keys }
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
pub type KeyEvent = Arc<Fn() + Send + Sync>;

/// Get a key mapping from the list.
pub fn get(key: &KeyPress) -> Option<KeyEvent> {
    let bindings = BINDINGS.read()
        .expect("Keybindings/get: unable to lock keybindings");
    match bindings.get(key) {
        None => None,
        Some(val) => Some(val.clone())
    }
}

/// Register a new set of key mappings
#[allow(dead_code)]
pub fn register(values: Vec<(KeyPress, KeyEvent)>) {
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
