//! Awesome compatibilty modules
use rlua::{self, Lua};
pub mod keygrabber;
pub mod mousegrabber;
pub mod awful;
mod button;
mod signal;
mod object;
mod class;
mod property;

pub use self::object::Object;
pub use self::keygrabber::keygrabber_handle;
pub use self::mousegrabber::mousegrabber_handle;

pub fn init(lua: &Lua) -> rlua::Result<()> {
    set_up_awesome_path(lua)?;
    let globals = lua.globals();
    let capi = lua.create_table();
    capi.set("keygrabber", keygrabber::init(lua)?)?;
    capi.set("mousegrabber", mousegrabber::init(lua)?)?;
    capi.set("button", button::init(lua)?.table)?;
    //awful::init(lua)?;
    globals.set("capi", capi)
}

fn set_up_awesome_path(lua: &Lua) -> rlua::Result<()> {
    let globals = lua.globals();
    let package: rlua::Table = globals.get("package")?;
    //let paths: String = package.get("path")?;
    // TODO Do this right, I'm too lazy and just scrapped from my awesome env
    package.set("path", "/usr/share/lua/5.3/?.lua;/usr/share/lua/5.3/?/init.lua;/usr/lib/lua/5.3/?.lua;/usr/lib/lua/5.3/?/init.lua;./?.lua;./?/init.lua;/home/timidger/.config/awesome/?.lua;/home/timidger/.config/awesome/?/init.lua;/etc/xdg/awesome/?.lua;/etc/xdg/awesome/?/init.lua;/usr/share/awesome/lib/?.lua;/usr/share/awesome/lib/?/init.lua")?;
    package.set("cpath", "/usr/lib/lua/5.3/?.so;/usr/lib/lua/5.3/loadall.so;./?.so;/home/timidger/.config/awesome/?.so;/etc/xdg/awesome/?.so;/usr/share/awesome/lib/?.so")?;
    // TODO Real debug, bug in rlua
    globals.set("debug", lua.create_table())
}
