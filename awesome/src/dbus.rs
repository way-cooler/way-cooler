//! Sets up the dbus interface for Awesome.

use std::{cell::RefCell, os::unix::io::RawFd, thread::LocalKey};

use dbus_rs::{BusType, Connection, Message, MessageItem, MessageType, MsgHandler,
              MsgHandlerType, MsgHandlerResult};
use rlua::{self, Lua, Table, Value, ToLua, ToLuaMulti, MultiValue,
           Error::RuntimeError};

use ::lua::LUA;
use ::common::signal;

type GlobalConnection = RefCell<Option<DBusConnection>>;

const SIGNALS_NAME: &'static str = "signals";

thread_local! {
    static SESSION_BUS: GlobalConnection = RefCell::new(None);
    static SYSTEM_BUS:  GlobalConnection = RefCell::new(None);
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
        let _ = session_bus.incoming(0).collect::<Vec<_>>();
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
        let _ = session_bus.incoming(0).collect::<Vec<_>>();
    });
    true
}

struct DBusConnection {
    connection: Connection
}

struct DBusHandler {
    global_connection: &'static LocalKey<GlobalConnection>
}

impl std::ops::Deref for DBusConnection {
    type Target = Connection;

    fn deref(&self) -> &Self::Target {
        &self.connection
    }
}

impl Drop for DBusConnection {
    fn drop(&mut self) {
        unsafe {
            ::remove_dbus_from_glib();
        }
        // Need to close both of them, no idea which one was just destroyed,
        // so try to destroy both of them.
        SESSION_BUS.try_with(|session_bus| {
            *session_bus.borrow_mut() = None
        }).ok();
        SYSTEM_BUS.try_with(|session_bus| {
            *session_bus.borrow_mut() = None
        }).ok();
    }
}

impl MsgHandler for DBusHandler {
    fn handler_type(&self) -> MsgHandlerType {
        MsgHandlerType::All
    }

    fn handle_msg(&mut self, msg: &Message) -> Option<MsgHandlerResult> {
        let (_, _, interface, member) = msg.headers();
        if msg.msg_type() == MessageType::Signal {
            match (interface.unwrap().as_str(), member.unwrap().as_str()) {
                ("org.freedesktop.DBus.Local", "Disconnected") => {
                    self.global_connection.with(|bus| {
                        *bus.borrow_mut() = None;
                    });
                    // TODO not none
                    return Some(MsgHandlerResult {
                        handled: true,
                        done: true,
                        reply: vec![]
                    });
                },
                _ => {}
            }
        }
        let reply = LUA.with(|lua| {
            let lua = lua.borrow();
            self.process_request(&*lua)
        }).unwrap_or_else(|err| {
            ::lua::log_error(err);
            vec![]
        });

        return Some(MsgHandlerResult {
            handled: true,
            done: false,
            reply
        })
    }
}

impl DBusHandler {
    fn process_request(&mut self, lua: &Lua) -> rlua::Result<Vec<Message>> {
        unimplemented!()
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
    let session_fds = session_con.watch_fds();
    let system_fds = system_con.watch_fds();
    assert_eq!(session_fds.len(), 1, "Only one fd per dbus connection");
    assert_eq!(system_fds.len(), 1, "Only one fd per dbus connection");
    session_con.add_handler(DBusHandler { global_connection: &SESSION_BUS });
    system_con.add_handler(DBusHandler { global_connection: &SYSTEM_BUS });

    SESSION_BUS.with(|session_bus| {
        let mut session_bus = session_bus.borrow_mut();
        *session_bus = Some(DBusConnection { connection: session_con });
    });
    SYSTEM_BUS.with(|system_bus| {
        let mut system_bus = system_bus.borrow_mut();
        *system_bus = Some(DBusConnection{ connection: system_con });
    });

    Ok((session_fds[0].fd(), system_fds[0].fd()))
}

/// Set up the DBus object in Lua so that the user libs can interact with
/// Awesome via DBus.
pub fn lua_init(lua: &Lua) -> rlua::Result<()> {
    let dbus_table = lua.create_table()?;
    dbus_table.set(SIGNALS_NAME, lua.create_table()?)?;
    dbus_table.set("request_name", lua.create_function(request_name)?)?;
    dbus_table.set("release_name", lua.create_function(release_name)?)?;
    dbus_table.set("add_match", lua.create_function(add_match)?)?;
    dbus_table.set("remove_match", lua.create_function(remove_match)?)?;
    dbus_table.set("connect_signal", lua.create_function(connect_signal)?)?;
    dbus_table.set("disconnect_signal", lua.create_function(disconnect_signal)?)?;
    dbus_table.set("emit_signal", lua.create_function(emit_signal)?)?;
    lua.globals().set("dbus", dbus_table)?;
    Ok(())
}

fn get_bus_by_name<'bus>(bus_name: &str)
                         -> rlua::Result<&'bus LocalKey<GlobalConnection>> {
    match bus_name {
        "session" => {
            Ok(&SESSION_BUS)
        },
        "system" => {
            Ok(&SYSTEM_BUS)
        },
        v => Err(RuntimeError(format!("Unknown bus type {}", v)))
    }
}

fn convert_value(lua: &Lua, type_: Value, value: Value) -> rlua::Result<MessageItem> {
    use rlua::Value;
    use ::dbus_rs::arg::ArgType;
    let type_ = match type_ {
        Value::String(s) => s.to_str()?.to_string(),
        _ => return Err(RuntimeError("D-Bus type name was not a string".into()))
    };
    let is_ascii = type_.chars().next()
        .map(|c| !char::is_ascii(&c))
        .unwrap_or(false);
    if type_.len() > 1 ||  !is_ascii {
        return Err(RuntimeError(format!("{} is an invalid type name", type_)))
    }
    let type_ = ArgType::from_i32(type_.chars().next().unwrap() as i32)
        .map_err(|_| RuntimeError(format!(
            "{} is an invalid type name", type_)))?;
    match (type_, value) {
        (ArgType::Array, Value::Table(value)) => {
            let size = value.len()?;
            if size % 2 != 0 {
                return Err(RuntimeError(
                    "your D-Bus signal handling method returned \
                     wrong number of arguments".into()))
            }
            let types = value.clone().sequence_values().step_by(2);
            let values = value.clone().sequence_values().skip(1).step_by(2);
            let mut list = Vec::with_capacity(size as usize);
            for (type_, value) in types.zip(values) {
                list.push(convert_value(lua, type_?, value?)?)
            }
            MessageItem::new_array(list)
               .map_err(|_| RuntimeError("Empty list is invalid".into()))
        },
        (ArgType::Boolean, Value::Boolean(value)) => {
            Ok(value.into())
        },
        (ArgType::String, Value::String(value)) => {
            Ok(value.to_str()?.into())
        },
        (ArgType::Byte, Value::Integer(value)) |
        (ArgType::Int16, Value::Integer(value)) |
        (ArgType::UInt16, Value::Integer(value)) |
        (ArgType::Int32, Value::Integer(value)) |
        (ArgType::UInt32, Value::Integer(value)) |
        (ArgType::Int64, Value::Integer(value)) |
        (ArgType::UInt64, Value::Integer(value))  => {
            Ok(value.into())
        },
        (ArgType::Byte, Value::Number(value)) |
        (ArgType::Int16, Value::Number(value)) |
        (ArgType::UInt16, Value::Number(value)) |
        (ArgType::Int32, Value::Number(value)) |
        (ArgType::UInt32, Value::Number(value)) |
        (ArgType::Int64, Value::Number(value)) |
        (ArgType::UInt64, Value::Number(value)) |
        (ArgType::Double, Value::Number(value)) => {
            Ok(value.into())
        }
        (type_, value) => {
            Err(RuntimeError(format!("Invalid type {:?} or value {:?}",
                                            type_, value)))
        }
    }
}

fn request_name(_: &Lua, (bus, name): (String, String)) -> rlua::Result<bool> {
    let bus = get_bus_by_name(bus.as_str())?;
    bus.with(|bus| {
        let bus = bus.borrow_mut();
        let bus = bus.as_ref().unwrap();
        bus.register_name(name.as_str(), 0)
            .expect(&format!("Could not register name {}", name.as_str()));
    });
    Ok(true)
}

fn release_name(_: &Lua, (bus, name): (String, String)) -> rlua::Result<bool> {
    let bus = get_bus_by_name(bus.as_str())?;
    bus.with(|bus| {
        let bus = bus.borrow_mut();
        let bus = bus.as_ref().unwrap();
        bus.release_name(name.as_str())
            .expect(&format!("Could not release name {}", name.as_str()));
    });
    Ok(true)
}

fn add_match(_: &Lua, (bus, name): (String, String)) -> rlua::Result<()> {
    let bus = get_bus_by_name(bus.as_str())?;
    bus.with(|bus| {
        let bus = bus.borrow_mut();
        let bus = bus.as_ref().unwrap();
        bus.add_match(name.as_str())
            .map_err(|err| RuntimeError(format!("{}", err)))
    })?;
    Ok(())
}

fn remove_match(_: &Lua, (bus, name): (String, String)) -> rlua::Result<()> {
    let bus = get_bus_by_name(bus.as_str())?;
    bus.with(|bus| {
        let bus = bus.borrow_mut();
        let bus = bus.as_ref().unwrap();
        bus.remove_match(name.as_str())
            .map_err(|err| RuntimeError(format!("{}", err)))
    })?;
    Ok(())
}

fn connect_signal<'lua>(lua: &'lua Lua, (name, func): (String, rlua::Function))
                  -> rlua::Result<MultiValue<'lua>> {
    let signals: Table = lua.globals()
        .get::<_, Table>("dbus")
        .unwrap().get(SIGNALS_NAME).unwrap();
    if signals.get(name.as_str())? {
        let error_msg = format!(
            "Cannot add signal {} on D-Bus, already existing", name.as_str());
        warn!("{}", error_msg);
        (rlua::Nil, error_msg).to_lua_multi(lua)
    } else {
        signal::connect_signals(lua, signals, name, &[func])?;
        (true.to_lua_multi(lua))
    }
}

fn disconnect_signal(lua: &Lua, (name, func): (String, rlua::Function))
                     -> rlua::Result<()> {
    let signals: Table = lua.globals()
        .get::<_, Table>("dbus").unwrap()
        .get(SIGNALS_NAME).unwrap();
    signal::disconnect_signals(lua, signals, name)
}

fn emit_signal<'lua>(lua: &'lua Lua, (bus, path, interface, name, args):
                     (String, String, String, String, MultiValue))
                     -> rlua::Result<Value<'lua>> {
    if args.len() % 2 != 0 {
        let error_msg =
            "your D-Bus signal emitting metod has wrong number of arguments";
        warn!("{}", error_msg);
        return false.to_lua(lua)
    }
    let types = args.iter().step_by(2);
    let values = args.iter().skip(1).step_by(2);
    let args = types.zip(values)
        .map(|v| convert_value(lua, v.0.clone(), v.1.clone()))
        .collect::<rlua::Result<Vec<MessageItem>>>()?;
    let bus = get_bus_by_name(bus.as_str())?;
    bus.with(|bus| {
        let bus = bus.borrow_mut();
        let bus = bus.as_ref().unwrap();
        // TODO catch panic, convert to error
        let mut msg = Message::signal(&path.into(), &interface.into(), &name.into());
        msg.append_items(&args);
        bus.send(msg)
    }).map_err(|_| RuntimeError("Could not send D-Bus message".into()))?;
    true.to_lua(lua)
}
