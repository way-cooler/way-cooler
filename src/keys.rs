//! Contains information for keybindings.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::hash::{Hash, Hasher};

use rustwlc::xkb::{Keysym, NameFlags};
use rustwlc::types::*; // Need * for bitflags...

use super::layout::tree;
use super::lua;

/// Register default keypresses to a map
macro_rules! register_defaults {
    ( $map:expr; $($func:expr, $press:expr);+ ) => {
        $(
            let _ = $map.insert($press, Arc::new(Box::new($func)));
            )+
    }
}

/// Generate switch_workspace methods and register them in $map
macro_rules! gen_switch_workspace {
    ($map:expr; $($b:ident, $n:expr);+) => {
        $(fn $b() {
            trace!("Switching to workspace {}", $n);
            tree::switch_workspace(&$n.to_string())
                .expect("Could not switch to a work-space");
                }
          register_defaults!( $map; $b, keypress!("Alt", $n) );
          )+
    };
}

lazy_static! {
    static ref BINDINGS: RwLock<HashMap<KeyPress, KeyEvent>> = {
        let mut map = HashMap::<KeyPress, KeyEvent>::new();


        gen_switch_workspace!(map; switch_workspace_1, "1";
                              switch_workspace_2, "2";
                              switch_workspace_3, "3";
                              switch_workspace_4, "4";
                              switch_workspace_5, "5";
                              switch_workspace_6, "6";
                              switch_workspace_7, "7";
                              switch_workspace_8, "8";
                              switch_workspace_9, "9";
                              switch_workspace_0, "0");

        register_defaults! {
            map;
            quit_fn, keypress!("Alt", "Escape");
            terminal_fn, keypress!("Alt", "Return");
            dmenu_fn, keypress!("Alt", "d");
            pointer_fn, keypress!("Alt", "p")
        }

        RwLock::new(map)
    };
}

fn terminal_fn() {
    use std::process::Command;
    Command::new("sh")
        .arg("-c")
        .arg("weston-terminal")
        .spawn().expect("Error launching terminal");
}

fn dmenu_fn() {
    use std::process::Command;
    Command::new("sh")
        .arg("-c")
        .arg("dmenu_run")
        .spawn().expect("Error launching terminal");
}

fn pointer_fn() {
    use lua::LuaQuery;
    let code = "if wm == nil then print('wm table does not exist')\n\
                elseif wm.pointer == nil then print('wm.pointer table does not exist')\n\
                else\n\
                print('get_position is a func, preparing execution')
                local x, y = wm.pointer.get_position()\n\
                print('The cursor is at ' .. x .. ', ' .. y)\n\
                end".to_string();
    lua::send(LuaQuery::Execute(code))
        .expect("Error telling Lua to get pointer coords");
}

fn quit_fn() {
    info!("handler: Esc keypress!");
    ::rustwlc::terminate();
}

#[derive(Eq, PartialEq, Clone, Debug)]
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
pub type KeyEvent = Arc<Box<Fn() + Send + Sync>>;

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
