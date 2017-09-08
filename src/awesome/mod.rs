//! Awesome compatibilty modules
use rlua::{self, Lua};
pub mod keygrabber;
pub mod mousegrabber;
pub mod awful;
mod signal;

pub use self::keygrabber::keygrabber_handle;
pub use self::mousegrabber::mousegrabber_handle;
pub use self::signal::Signal;

pub fn init(lua: &Lua) -> rlua::Result<()> {
    keygrabber::init(lua)?;
    mousegrabber::init(lua)?;
    awful::init(lua)?;
    Ok(())
}
