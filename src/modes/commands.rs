//! Commands to control the modes.

use super::{Modes, Default, CustomLua, write_current_mode};


/// Sets the mode to the default (don't execute custom Lua code).
pub fn set_default_mode() {
    *write_current_mode() = Modes::Default(Default)
}

/// Sets the mode to the Custom Lua mode (execute any custom Lua code that
/// the user has defined).
pub fn set_custom_mode() {
    *write_current_mode() = Modes::CustomLua(CustomLua)
}
