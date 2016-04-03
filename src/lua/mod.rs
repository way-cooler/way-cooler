//! Lua functionality

#[macro_use]
extern crate hlua;

lazy_static! {
    static ref LUA = Lua::new();
}
