//! Utilities to talk to Lua

use wlroots::{xkbcommon::xkb::keysyms::*,
              events::{pointer_events::{ButtonEvent, wlr_button_state, BTN_LEFT, BTN_RIGHT, BTN_MIDDLE,
                                        BTN_SIDE, BTN_EXTRA, BTN_FORWARD, BTN_BACK, BTN_TASK},
                       key_events::{KeyEvent, Key}}};


use rlua::{self, Lua, Table, Value, Error::RuntimeError};

/// Human readable versions of the standard modifier keys.
const MOD_NAMES: [&str; 8] = ["Shift", "Caps", "Control", "Alt", "Mod2", "Mod3", "Mod4", "Mod5"];
/// Keycodes corresponding to various button events.
const MOUSE_EVENTS: [u32; 5] = [BTN_LEFT, BTN_RIGHT, BTN_MIDDLE, BTN_SIDE, BTN_EXTRA];

/// Convert a modifier to the Lua interpretation
pub fn mods_to_lua<'lua>(lua: &'lua Lua, mods: &[Key]) -> rlua::Result<Table<'lua>> {
    let mut mods_list: Vec<String> = Vec::with_capacity(MOD_NAMES.len());
    for modifier in mods {
        mods_list.push(match *modifier {
	          KEY_Shift_L | KEY_Shift_R => "Shift",
	          KEY_Control_L | KEY_Control_R => "Control",
	          KEY_Caps_Lock => "Caps",
	          KEY_Alt_L | KEY_Alt_R => "Alt",
	          KEY_Meta_L | KEY_Meta_R => "Mod2",
	          KEY_Super_L | KEY_Super_R => "Mod4",
            _ => continue
        }.into());
    }
    lua.create_table_from(mods_list.into_iter().enumerate())
}

/// Convert a modifier to the Rust interpretation, from the Lua interpretation
pub fn mods_to_rust(mods_table: Table) -> rlua::Result<Vec<Key>> {
    let mut mods = Vec::with_capacity(MOD_NAMES.len());
    for modifier in mods_table.pairs::<Value, String>() {
        mods.push(match &*modifier?.1 {
            "Shift" => KEY_Shift_L,
            "Caps"|"Lock" => KEY_Caps_Lock,
            "Control"|"Ctrl" => KEY_Control_L,
            "Alt"|"Mod1" => KEY_Alt_L,
            "Mod2" => KEY_Meta_L,
            "Mod3" => KEY_Alt_L,
            "Mod4" => KEY_Super_L,
            "Mod5" => KEY_Hyper_L,
            string => {
                return Err(RuntimeError(format!("{} is an invalid modifier", string)))?
            }
        })
    }
    Ok(mods)
}

/// Convert a mouse event from Wayland to the representation Lua expcets
pub fn mouse_events_to_lua(_: &rlua::Lua, button: u32,
                           button_state: wlr_button_state) -> rlua::Result<Vec<bool>> {
    let mut event_list = Vec::with_capacity(MOUSE_EVENTS.len());
    for mouse_event in &MOUSE_EVENTS[..5] {
        let state_pressed = button_state == wlr_button_state::WLR_BUTTON_PRESSED;
        let is_pressed = button == *mouse_event && state_pressed;
        event_list.push(is_pressed);
    }
    Ok(event_list)
}
