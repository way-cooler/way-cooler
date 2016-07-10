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
}


pub fn init() {
    macro_rules! register {
        ( $($press:expr => $name:expr);+ ) => {
            register(vec![
                $( ($press, KeyEvent::Command(commands::get(&$name.to_string())
                  .expect("Unable to register default command!"))) ),+
                ]);
        }
    }

    register! {
        keypress!("Alt", "Escape") => "quit";
        keypress!("Alt", "Return") => "launch_terminal";
        keypress!("Alt", "d") => "launch_dmenu";
        keypress!("Alt", "l") => "dmenu_eval";
        KeyPress::from_key_names(vec!["Alt", "Shift"], "l")
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

        keypress!("Alt", "q") => "remove_active";

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
        keypress!("Alt", "0") => "switch_workspace_0";

        /* Moving active container to another Workspace key bindings */
        KeyPress::from_key_names(vec!("Alt", "Shift"), "1").unwrap() => "move_to_workspace_1";
        KeyPress::from_key_names(vec!("Alt", "Shift"), "2").unwrap() => "move_to_workspace_2";
        KeyPress::from_key_names(vec!("Alt", "Shift"), "3").unwrap() => "move_to_workspace_3";
        KeyPress::from_key_names(vec!("Alt", "Shift"), "4").unwrap() => "move_to_workspace_4";
        KeyPress::from_key_names(vec!("Alt", "Shift"), "5").unwrap() => "move_to_workspace_5";
        KeyPress::from_key_names(vec!("Alt", "Shift"), "6").unwrap() => "move_to_workspace_6";
        KeyPress::from_key_names(vec!("Alt", "Shift"), "7").unwrap() => "move_to_workspace_7";
        KeyPress::from_key_names(vec!("Alt", "Shift"), "8").unwrap() => "move_to_workspace_8";
        KeyPress::from_key_names(vec!("Alt", "Shift"), "9").unwrap() => "move_to_workspace_9";
        KeyPress::from_key_names(vec!("Alt", "Shift"), "0").unwrap() => "move_to_workspace_0"
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

/// Get a key mapping from the list.
pub fn get(key: &KeyPress) -> Option<KeyEvent> {
    let bindings = BINDINGS.read()
        .expect("Keybindings/get: unable to lock keybindings");
    bindings.get(key).map(KeyEvent::clone)
}

/// Register a new set of key mappings
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
