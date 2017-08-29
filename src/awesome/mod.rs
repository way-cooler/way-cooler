//! Awesome compatibilty modules
use rlua::{self, Lua};
pub mod keygrabber;

pub use self::keygrabber::keygrabber_handle;

pub fn init(lua: &Lua) -> rlua::Result<()> {
    keygrabber::init(lua)
}
