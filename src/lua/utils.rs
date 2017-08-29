//! Utilities to talk to Lua

use rlua::{self, Table};
use rustwlc::*;

const MOD_NAMES: [&str; 8] = ["Shift", "Caps", "Control", "Alt",
                              "Mod2", "Mod3", "Mod4", "Mod5"];

/// Convert a modifier to the Lua interpretation
pub fn mods_to_lua(lua: &rlua::Lua, mut mods: KeyMod) -> rlua::Result<Table> {
    let mut mods_list: Vec<String> = Vec::with_capacity(MOD_NAMES.len());
    for index in 0..MOD_NAMES.len() {
        if mods == MOD_NONE {
            break;
        }
        if mods.bits() & 1 != 0 {
            mods_list.push(MOD_NAMES[index].into());
        }
        mods = KeyMod::from_bits_truncate(mods.bits() >> 1);
    }
    lua.create_table_from(mods_list.into_iter().enumerate())
}
