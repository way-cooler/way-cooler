pub mod button;
pub mod client;
pub mod drawable;
pub mod drawin;
pub mod key;
pub mod mouse;
pub mod screen;
pub mod tag;

use rlua::{self, Lua};

/// A dummy function to use as a stub.
///
/// It can take the place of any Lua function that the Awesome libs exppect,
/// and will always return nil (which is probably not what you want).
pub fn dummy<'lua>(_: &'lua Lua, _: rlua::Value) -> rlua::Result<()> { Ok(()) }
