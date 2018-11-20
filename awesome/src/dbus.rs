//! Sets up the dbus interface for Awesome.

use std::{cell::RefCell, os::unix::io::RawFd};

use dbus_rs::{BusType, Connection, Message, MsgHandler, MsgHandlerType,
              MsgHandlerResult};
use rlua::{self, Lua};

thread_local! {
    pub static SESSION_BUS: RefCell<Option<Connection>> = RefCell::new(None);
    pub static SYSTEM_BUS:  RefCell<Option<Connection>> = RefCell::new(None);
}

/// Called from `wayland_glib_interface.c` whenever a request is sent to the
/// current session dbus file descriptor.
///
/// This will kick off the special handling code in `dbus.rs`.
#[no_mangle]
pub extern "C" fn dbus_session_refresh(_: libc::c_void) -> bool {
    SESSION_BUS.with(|session_bus| {
        let session_bus = session_bus.borrow_mut();
        let session_bus = session_bus.as_ref().unwrap();
        session_bus.incoming(0);
    });
    true
}

/// Called from `wayland_glib_interface.c` whenever a request is sent to the
/// system dbus file descriptor.
///
/// This will kick off the special handling code in `dbus.rs`.
#[no_mangle]
pub extern "C" fn dbus_system_refresh(_: libc::c_void) -> bool {
    SYSTEM_BUS.with(|session_bus| {
        let session_bus = session_bus.borrow_mut();
        let session_bus = session_bus.as_ref().unwrap();
        session_bus.incoming(0);
    });
    true
}

struct DBusHandler;

impl MsgHandler for DBusHandler {
    fn handler_type(&self) -> MsgHandlerType {
        MsgHandlerType::All
    }

    fn handle_msg(&mut self, msg: &Message) -> Option<MsgHandlerResult> {
        // TODO
        None
    }
}

/// Set up the connections to the session bus and system bus.
///
/// The message handling is all done within the `dbus` module,
/// it's up to the caller of this function to register the DBus
/// file descriptors with glib so that we can awaken and deal
/// with dbus events when necessary.
pub fn connect() -> Result<(RawFd, RawFd), dbus::Error> {
    let session_con = Connection::get_private(BusType::Session)?;
    let system_con = Connection::get_private(BusType::System)?;
    session_con.add_handler(DBusHandler);
    system_con.add_handler(DBusHandler);
    let session_fds = session_con.watch_fds();
    let system_fds = system_con.watch_fds();
    assert_eq!(session_fds.len(), 1, "Only one fd per dbus connection");
    assert_eq!(system_fds.len(), 1, "Only one fd per dbus connection");

    SESSION_BUS.with(|session_bus| {
        let mut session_bus = session_bus.borrow_mut();
        *session_bus = Some(session_con);
    });
    SYSTEM_BUS.with(|system_bus| {
        let mut system_bus = system_bus.borrow_mut();
        *system_bus = Some(system_con);
    });

    Ok((session_fds[0].fd(), system_fds[0].fd()))
}

/// Set up the DBus object in Lua so that the user libs can interact with
/// Awesome via DBus.
pub fn lua_init(lua: &Lua) -> rlua::Result<()> {
    let dbus_table = lua.create_table()?;
    dbus_table.set("request_name", lua.create_function(request_name)?)?;
    lua.globals().set("dbus", dbus_table)?;
    Ok(())
}

fn request_name(lua: &Lua, (bus, name): (String, String)) -> rlua::Result<bool> {
    let bus = match bus.as_str() {
        "session" => {
            &SESSION_BUS
        },
        "system" => {
            &SYSTEM_BUS
        },
        v => panic!("Unknown bus type {}", v)
    };
    bus.with(|bus| {
        let bus = bus.borrow_mut();
        let bus = bus.as_ref().unwrap();
        let _ = bus.register_name(name.as_str(), 0);
    });
    Ok(true)
}
