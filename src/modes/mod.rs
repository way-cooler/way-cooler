//! Way Cooler exists in one of several different "Modes"
//! The current mode defines what Way Cooler does in each callback.
//!
//! The central use of this is to define different commands that the user can
//! run.
//!
//! For example, when the lock screen mode is active the user can't do anything
//! other than send input to the lock screen program.
//!
//! We also allow users to define their own custom modes, which allows them to
//! hook into the callbacks from Lua. In the `CustomLua` mode, callbacks do
//! the same thing as they do in the `Default` mode, but at the end of will
//! always execute some custom Lua code.

use std::ops::{Deref, DerefMut};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

mod mode;
mod default;
mod custom_lua;
mod lock_screen;
pub mod commands;
pub use self::mode::Mode;
pub use self::default::Default;
pub use self::custom_lua::CustomLua;
pub use self::lock_screen::{LockScreen, spawn_lock_screen};

/// If the event is handled by way-cooler
pub const EVENT_BLOCKED: bool = true;
/// If the event should be passed through to clients
pub const EVENT_PASS_THROUGH: bool = false;
/// Left click constant, used in `on_pointer_button`
pub const LEFT_CLICK: u32 = 0x110;
/// Right click constant, used in `on_pointer_button`
pub const RIGHT_CLICK: u32 = 0x111;

/// The different modes that Way Cooler can be in
/// * `Default`: The default mode for Way Cooler, this is the standard mode
/// that it starts out in
/// * `CustomLua`: Same as `Default`, except it calls any custom defined
/// callbacks in the Lua configuration file at the end of the call back.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Modes {
    Default(Default),
    CustomLua(CustomLua),
    LockScreen(LockScreen)
}

impl From<Default> for Modes {
    fn from(mode: Default) -> Modes {
        Modes::Default(mode)
    }
}

impl From<CustomLua> for Modes {
    fn from(mode: CustomLua) -> Modes {
        Modes::CustomLua(mode)
    }
}

impl From<LockScreen> for Modes {
    fn from(mode: LockScreen) -> Modes {
        Modes::LockScreen(mode)
    }
}

lazy_static! {
    pub static ref CURRENT_MODE: RwLock<Modes> =
        RwLock::new(Modes::Default(Default));
}

pub fn write_current_mode<'a>() -> RwLockWriteGuard<'a, Modes> {
    CURRENT_MODE.try_write()
        .expect("Unable to write current mode")
}

pub fn read_current_mode<'a>() -> RwLockReadGuard<'a, Modes> {
    CURRENT_MODE.try_read()
        .expect("Unable to read current mode")
}

impl Deref for Modes {
    type Target = Mode;

    fn deref(&self) -> &(Mode + 'static) {
        match *self {
            Modes::Default(ref mode) => mode as &Mode,
            Modes::CustomLua(ref mode) => mode as &Mode,
            Modes::LockScreen(ref mode) => mode as &Mode
        }
    }
}

impl DerefMut for Modes {
    fn deref_mut(&mut self) -> &mut (Mode + 'static) {
        match *self {
            Modes::Default(ref mut mode) => mode as &mut Mode,
            Modes::CustomLua(ref mut mode) => mode as &mut Mode,
            Modes::LockScreen(ref mut mode) => mode as &mut Mode
        }
    }
}
