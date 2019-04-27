//! Utilities to talk to Lua

use enumflags2::BitFlags;
use evdev::Key::{BTN_EXTRA, BTN_LEFT, BTN_MIDDLE, BTN_RIGHT, BTN_SIDE};
use rlua::{self, Error::RuntimeError, Lua, Table, Value};
use xkbcommon::xkb::{keysyms::*, Keysym};

#[derive(enumflags2_derive::EnumFlags, Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum KeyboardModifiers {
    Shift = 1 << 0,
    Caps = 1 << 1,
    Ctrl = 1 << 2,
    Alt = 1 << 3,
    Mod2 = 1 << 4,
    Mod3 = 1 << 5,
    Logo = 1 << 6,
    Mod5 = 1 << 7
}

/// Human readable versions of the standard modifier keys.
#[allow(dead_code)]
static MOD_NAMES: [&str; 8] = ["Shift", "Caps", "Control", "Alt", "Mod2", "Mod3", "Mod4", "Mod5"];
/// Keycodes corresponding to various button events.
static MOUSE_EVENTS: [evdev::Key; 5] = [BTN_LEFT, BTN_RIGHT, BTN_MIDDLE, BTN_SIDE, BTN_EXTRA];
static MOD_TYPES: [(KeyboardModifiers, Keysym); 7] = [
    (KeyboardModifiers::Shift, KEY_Shift_L),
    (KeyboardModifiers::Caps, KEY_Caps_Lock),
    (KeyboardModifiers::Ctrl, KEY_Control_L),
    (KeyboardModifiers::Alt, KEY_Alt_L),
    (KeyboardModifiers::Mod2, KEY_Meta_L),
    (KeyboardModifiers::Logo, KEY_Super_L),
    (KeyboardModifiers::Mod5, KEY_Hyper_L)
];

/// Convert a modifier to the Lua interpretation
#[allow(non_upper_case_globals)]
pub fn mods_to_lua<'lua>(lua: &'lua Lua, mods: &[Keysym]) -> rlua::Result<Table<'lua>> {
    let mut mods_list: Vec<String> = Vec::with_capacity(MOD_NAMES.len());
    for modifier in mods {
        mods_list.push(
            match *modifier {
                KEY_Shift_L | KEY_Shift_R => "Shift",
                KEY_Control_L | KEY_Control_R => "Control",
                KEY_Caps_Lock => "Caps",
                KEY_Alt_L | KEY_Alt_R => "Alt",
                KEY_Meta_L | KEY_Meta_R => "Mod2",
                KEY_Super_L | KEY_Super_R => "Mod4",
                _ => continue
            }
            .into()
        );
    }
    lua.create_table_from(mods_list.into_iter().enumerate())
}

/// Convert a single number to a modifier list.
#[allow(dead_code)]
pub fn num_to_mods(modifiers: BitFlags<KeyboardModifiers>) -> Vec<Keysym> {
    let mut res = vec![];
    for (mod_km, mod_k) in MOD_TYPES.iter() {
        if (BitFlags::from_bits_truncate(mod_km.clone() as _) & modifiers) !=
            BitFlags::<KeyboardModifiers>::empty()
        {
            res.push(mod_k.clone());
        }
    }
    res
}

/// Convert a modifier list to a single number.
#[allow(non_upper_case_globals)]
pub fn mods_to_num(modifiers: Table) -> rlua::Result<BitFlags<KeyboardModifiers>> {
    let mut res = BitFlags::<KeyboardModifiers>::empty();
    for modifier in mods_to_rust(modifiers)? {
        res.insert(match modifier {
            KEY_Shift_L => KeyboardModifiers::Shift,
            KEY_Caps_Lock => KeyboardModifiers::Caps,
            KEY_Control_L => KeyboardModifiers::Ctrl,
            KEY_Alt_L => KeyboardModifiers::Alt,
            KEY_Meta_L => KeyboardModifiers::Mod2,
            KEY_Super_L => KeyboardModifiers::Logo,
            KEY_Hyper_L => KeyboardModifiers::Mod5,
            k => {
                error!("Unknown modifier {:?}", k);
                panic!("Unknown mod");
            }
        });
    }
    Ok(res)
}

/// Convert a modifier to the Rust interpretation, from the Lua interpretation
pub fn mods_to_rust(mods_table: Table) -> rlua::Result<Vec<Keysym>> {
    let mut mods = Vec::with_capacity(MOD_NAMES.len());
    for modifier in mods_table.pairs::<Value, String>() {
        mods.push(match &*modifier?.1 {
            "Shift" => KEY_Shift_L,
            "Caps" | "Lock" => KEY_Caps_Lock,
            "Control" | "Ctrl" => KEY_Control_L,
            "Alt" | "Mod1" => KEY_Alt_L,
            "Mod2" => KEY_Meta_L,
            "Mod3" => KEY_Alt_L,
            "Mod4" => KEY_Super_L,
            "Mod5" => KEY_Hyper_L,
            string => return Err(RuntimeError(format!("{} is an invalid modifier", string)))?
        })
    }
    Ok(mods)
}

/// Convert a mouse event from Wayland to the representation Lua expcets
// TODO Need a proper type for button_state
pub fn mouse_events_to_lua(_: &Lua, button: u32, button_state: u32) -> rlua::Result<Vec<bool>> {
    let mut event_list = Vec::with_capacity(MOUSE_EVENTS.len());
    for mouse_event in &MOUSE_EVENTS[..5] {
        let state_pressed = button_state == 0;
        let is_pressed = button == *mouse_event as u32 && state_pressed;
        event_list.push(is_pressed);
    }
    Ok(event_list)
}
