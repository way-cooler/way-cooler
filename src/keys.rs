//! Contains information for keybindings.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::hash::{Hash, Hasher};

use rustwlc::xkb::{Keysym, NameFlags};
use rustwlc::types::*; // Need * for bitflags...

use super::layout::tree;
use super::lua;

macro_rules! gen_switch_workspace {
    ($($b:ident, $n:expr);+) => {
        $(fn $b() {
            trace!("Switching to workspace {}", $n);
            tree::switch_workspace(&$n.to_string()).expect("Could not switch to a work-space");
        })+
    };
}

gen_switch_workspace!(switch_workspace_1, 1;
                      switch_workspace_2, 2;
                      switch_workspace_3, 3;
                      switch_workspace_4, 4;
                      switch_workspace_5, 5;
                      switch_workspace_6, 6;
                      switch_workspace_7, 7;
                      switch_workspace_8, 8;
                      switch_workspace_9, 9;
                      switch_workspace_0, 0);



lazy_static! {
    static ref BINDINGS: RwLock<HashMap<KeyPress, KeyEvent>> = {
        let mut map = HashMap::<KeyPress, KeyEvent>::new();

        let press_s = KeyPress::from_key_names(vec!["Mod4"], vec!["s"]).unwrap();
        map.insert(press_s, Arc::new(Box::new(key_s)));

        let press_f4 = KeyPress::from_key_names(vec!["Alt"],vec!["F4"]).unwrap();
        map.insert(press_f4, Arc::new(Box::new(key_f4)));

        let press_k = KeyPress::from_key_names(
            vec!["Ctrl"], vec!["k"]).unwrap();
        map.insert(press_k, Arc::new(Box::new(key_sleep)));

        let press_p = KeyPress::from_key_names(vec!["Ctrl"], vec!["p"]).unwrap();
        map.insert(press_p, Arc::new(Box::new(key_pointer_pos)));

        let press_esc = KeyPress::from_key_names(vec!["Ctrl"],
                                                 vec!["Escape"]).unwrap();
        map.insert(press_esc, Arc::new(Box::new(key_esc)));

        /* Workspace functions*/
        let terminal = KeyPress::from_key_names(vec!["Ctrl"], vec!["Return"]).unwrap();
        map.insert(terminal, Arc::new(Box::new(terminal_fn)));

        let dmenu = KeyPress::from_key_names(vec!["Alt"], vec!["d"]).unwrap();
        map.insert(dmenu, Arc::new(Box::new(dmenu_fn)));

        let switch_1 = KeyPress::from_key_names(vec!["Ctrl"], vec!["1"]).unwrap();
        map.insert(switch_1, Arc::new(Box::new(switch_workspace_1)));

        let switch_2 = KeyPress::from_key_names(vec!["Ctrl"], vec!["2"]).unwrap();
        map.insert(switch_2, Arc::new(Box::new(switch_workspace_2)));

        let switch_3 = KeyPress::from_key_names(vec!["Ctrl"], vec!["3"]).unwrap();
        map.insert(switch_3, Arc::new(Box::new(switch_workspace_3)));

        let switch_4 = KeyPress::from_key_names(vec!["Ctrl"], vec!["4"]).unwrap();
        map.insert(switch_4, Arc::new(Box::new(switch_workspace_4)));

        let switch_5 = KeyPress::from_key_names(vec!["Ctrl"], vec!["5"]).unwrap();
        map.insert(switch_5, Arc::new(Box::new(switch_workspace_5)));

        let switch_6 = KeyPress::from_key_names(vec!["Ctrl"], vec!["6"]).unwrap();
        map.insert(switch_6, Arc::new(Box::new(switch_workspace_6)));

        let switch_7 = KeyPress::from_key_names(vec!["Ctrl"], vec!["7"]).unwrap();
        map.insert(switch_7, Arc::new(Box::new(switch_workspace_7)));

        let switch_8 = KeyPress::from_key_names(vec!["Ctrl"], vec!["8"]).unwrap();
        map.insert(switch_8, Arc::new(Box::new(switch_workspace_8)));

        let switch_9 = KeyPress::from_key_names(vec!["Ctrl"], vec!["9"]).unwrap();
        map.insert(switch_9, Arc::new(Box::new(switch_workspace_9)));

        let switch_0 = KeyPress::from_key_names(vec!["Ctrl"], vec!["0"]).unwrap();
        map.insert(switch_0, Arc::new(Box::new(switch_workspace_0)));

        RwLock::new(map)
    };
}

fn terminal_fn() {
    use std::process::Command;
    Command::new("sh")
        .arg("-c")
        .arg("weston-terminal")
        .spawn().unwrap();
}

fn dmenu_fn() {
    use std::process::Command;
    Command::new("sh")
        .arg("-c")
        .arg("dmenu_run")
        .spawn().unwrap();
}

fn key_sleep() {
    use lua::LuaQuery;

    info!("keyhandler: Beginning thread::sleep keypress!");
    lua::send(LuaQuery::Execute("print('>entering sleep')\
                                 os.execute('sleep 5')\
                                 print('>leaving sleep')".to_string()))
                  .unwrap();
    info!("keyhandler: Finished thread::sleep keypress!");
}

fn key_pointer_pos() {
    use lua::LuaQuery;
    let code = "if wm == nil then print('wm table does not exist')\n\
                elseif wm.pointer == nil then print('wm.pointer table does not exist')\n\
                else\n\
                print('get_position is a func, preparing execution')
                local x, y = wm.pointer.get_position()\n\
                print('The cursor is at ' .. x .. ', ' .. y)\n\
                end".to_string();
    lua::send(LuaQuery::Execute(code)).unwrap();
}

fn key_s() {
    info!("[Key handler] S keypress!");
}

fn key_f4() {
    info!("[Key handler] F4 keypress!");
}

fn key_esc() {
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
    let bindings = BINDINGS.read().unwrap();
    match bindings.get(key) {
        None => None,
        Some(val) => Some(val.clone())
    }
}

/// Register a new set of key mappings
#[allow(dead_code)]
pub fn register(values: Vec<(KeyPress, KeyEvent)>) {
    let mut bindings = BINDINGS.write().unwrap();
    for value in values {
        bindings.insert(value.0, value.1);
    }
}
