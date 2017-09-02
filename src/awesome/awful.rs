//! AwesomeWM Awful interface

use super::keygrabber::KEYGRABBER_TABLE;
pub const AWFUL_TABLE: &str = "awful";

use rlua::{self, Lua};
pub fn init(lua: &Lua) -> rlua::Result<()> {
    let awful_table = lua.create_table();
    let globals = lua.globals();
    let keygrabber_table = globals.get::<_, rlua::Table>(KEYGRABBER_TABLE)?;
    awful_table.set(KEYGRABBER_TABLE, keygrabber_table)?;
    globals.set(AWFUL_TABLE, awful_table)
}
