//! Utilities to talk to Lua

use rlua::{self, Lua, Table, Value};
use rustwlc::*;

const MOD_NAMES: [&str; 8] = ["Shift", "Caps", "Control", "Alt",
                              "Mod2", "Mod3", "Mod4", "Mod5"];
const MOUSE_EVENTS: [u32; 6] = [
    // TODO This only grabs the buttons, not the scroll
    // This is a WLC limitation, it will be lifted later!

    272,  // button mask 1
    273,  // button mask 2
    274, // button mask 3
    275, // button mask 4
    276, // button mask 5

    // TODO currently unused
    1 << 15  /* mask any */];

/// Convert a modifier to the Lua interpretation
pub fn mods_to_lua(lua: &Lua, mut mods: KeyMod) -> rlua::Result<Table> {
    let mut mods_list: Vec<String> = Vec::with_capacity(MOD_NAMES.len());
    for mod_name in &MOD_NAMES {
        if mods == MOD_NONE {
            break;
        }
        if mods.bits() & 1 != 0 {
            mods_list.push((*mod_name).into());
        }
        mods = KeyMod::from_bits_truncate(mods.bits() >> 1);
    }
    lua.create_table_from(mods_list.into_iter().enumerate())
}

/// Convert a modifier to the Rust interpretation, from the Lua interpretation
pub fn mods_to_rust(mods_table: Table) -> rlua::Result<KeyMod> {
    let mut mods = KeyMod::empty();
    for modifier in mods_table.pairs::<Value, String>() {
        match &*modifier?.1 {
            "Shift" => mods.insert(MOD_SHIFT),
            "Caps"|"Lock" => mods.insert(MOD_CAPS),
            "Control"|"Ctrl" => mods.insert(MOD_CTRL),
            "Alt"|"Mod1" => mods.insert(MOD_ALT),
            "Mod2" => mods.insert(MOD_MOD2),
            "Mod3" => mods.insert(MOD_MOD3),
            "Mod4" => mods.insert(MOD_MOD4),
            "Mod5" => mods.insert(MOD_MOD5),
            string => {
                use rlua::Error::RuntimeError;
                Err(RuntimeError(format!("{} is an invalid modifier", string)))?
            }
        }
    }
    Ok(mods)
}

/// Convert a mouse event from Wayland to the representation Lua expcets
pub fn mouse_events_to_lua(_: &rlua::Lua, button: u32,
                           button_state: ButtonState) -> rlua::Result<Vec<bool>> {
    let mut event_list = Vec::with_capacity(MOUSE_EVENTS.len());
    for mouse_event in &MOUSE_EVENTS[..5] {
        let state_pressed = button_state == ButtonState::Pressed;
        let is_pressed = button == *mouse_event && state_pressed;
        event_list.push(is_pressed);
    }
    Ok(event_list)
}
