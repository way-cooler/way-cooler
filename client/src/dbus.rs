//! Sets up the dbus interface for Awesome.

use std::{os::unix::io::RawFd, slice, thread::LocalKey};

use {
    dbus::{
        arg::ArgType, BusType, Connection, Message, MessageItem, MessageType
    },
    rlua::{
        self, Error::RuntimeError, MultiValue, Table, ToLua, ToLuaMulti, Value
    }
};

use crate::{common::signal, lua::LUA};

const SIGNALS_NAME: &'static str = "signals";

thread_local! {
    static SESSION_BUS: DBusConnection = {
        let connection =
            DBusConnection {
                connection: Connection::get_private(BusType::Session)
                    .expect("Could not set up session bus")
            };
        connection.replace_message_callback(Some(Box::new(handle_msg_session)));
        connection
    };
    static SYSTEM_BUS: DBusConnection = {
        let connection = DBusConnection {
            connection: Connection::get_private(BusType::System)
                .expect("Could not set up system bus")
        };
        connection.replace_message_callback(Some(Box::new(handle_msg_system)));
        connection
    };
}

/// Called from `wayland_glib_interface.c` whenever a request is sent to the
/// session dbus file descriptor.
#[no_mangle]
pub extern "C" fn dbus_session_refresh(_: libc::c_void) -> bool {
    SESSION_BUS.with(|session_bus| {
        session_bus.incoming(0).for_each(drop);
    });
    true
}

/// Called from `wayland_glib_interface.c` whenever a request is sent to the
/// system dbus file descriptor.
#[no_mangle]
pub extern "C" fn dbus_system_refresh(_: libc::c_void) -> bool {
    SYSTEM_BUS.with(|system_bus| {
        system_bus.incoming(0).for_each(drop);
    });
    true
}

struct DBusConnection {
    connection: Connection
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
            crate::remove_dbus_from_glib();
        }
    }
}

fn handle_msg_session(bus: &Connection, msg: Message) -> bool {
    handle_msg(bus, BusType::Session, msg)
}

fn handle_msg_system(bus: &Connection, msg: Message) -> bool {
    handle_msg(bus, BusType::System, msg)
}

fn handle_msg(bus: &Connection, bus_type: BusType, msg: Message) -> bool {
    if msg.msg_type() == MessageType::Signal {
        let (_, _, interface, member) = msg.headers();
        match (interface.unwrap().as_str(), member.unwrap().as_str()) {
            ("org.freedesktop.DBus.Local", "Disconnected") => return false,
            _ => {}
        }
    }
    let reply = LUA.with(|lua| {
        let lua = lua.borrow();
        lua.context(|lua_ctx| process_request(bus_type, lua_ctx, &msg))
    });
    match reply {
        Ok(Some(reply)) => {
            bus.send(reply).expect("Could not send D-Bus reply");
        },
        Ok(None) => {},
        Err(err) => crate::lua::log_error(err)
    }
    true
}

/// Gives the D-Bus message to Lua for processing the reply.
fn process_request(
    bus_type: BusType,
    lua: rlua::Context,
    msg: &Message
) -> rlua::Result<Option<Message>> {
    let message_metadata = lua.create_table()?;

    let msg_type = match msg.msg_type() {
        MessageType::Signal => "signal",
        MessageType::MethodCall => "method_call",
        MessageType::MethodReturn => "method_return",
        MessageType::Error => "error",
        MessageType::Invalid => "unknown"
    };
    message_metadata.set("type", msg_type)?;

    let (_, path, interface, member) = msg.headers();
    let (path, interface, member) = (
        path.unwrap_or_default(),
        interface.unwrap_or_default(),
        member.unwrap_or_default()
    );
    message_metadata.set("interface", interface.as_str())?;
    message_metadata.set("path", path)?;
    message_metadata.set("member", member)?;

    if let Some(sender) = msg.sender() {
        message_metadata.set("sender", sender.to_string())?;
    }

    let bus_typstr = match bus_type {
        BusType::Session => "session",
        BusType::System => "system",
        BusType::Starter => return Ok(None)
    };
    message_metadata.set("bus", bus_typstr)?;

    let lua_message = dbus_to_lua_value(lua, msg.get_items().as_slice())?;

    let dbus_table = lua
        .globals()
        .get::<_, Table>("dbus")
        .expect("Could not get dbus table");
    let signals = dbus_table
        .get(SIGNALS_NAME)
        .expect("Could not get signals table");
    if msg.get_no_reply() {
        signal::emit_signals(lua, signals, &interface, lua_message)?;
        return Ok(None);
    }

    if let Ok(Value::Table(sig)) = signals.get(interface) {
        // There can only be ONE handler to send reply, so get the first one.
        let func: rlua::Function = sig.get(1)?;
        let res: MultiValue = func.call((message_metadata, lua_message))?;
        if res.len() % 2 != 0 {
            warn!(
                "Your D-Bus signal handling method returned \
                 wrong number of arguments (must be even)"
            );
            return Ok(None);
        }

        let types = res.iter().step_by(2);
        let values = res.iter().skip(1).step_by(2);
        let mut reply = msg.method_return();
        for (typ, value) in types.zip(values) {
            match lua_value_to_dbus(lua, typ.clone(), value.clone()) {
                Ok(v) => {
                    reply = reply.append(v);
                },
                Err(err) => {
                    crate::lua::log_error(err);
                    warn!(
                        "Your D-Bus signal handling method returned \
                         bad data. Expected type and value pairs."
                    );
                    return Ok(None);
                }
            }
        }
        Ok(Some(reply))
    } else {
        Ok(None)
    }
}

/// Set up the connections to the session bus and system bus.
///
/// The message handling is all done within the `dbus` module,
/// it's up to the caller of this function to register the DBus
/// file descriptors with glib so that we can awaken and deal
/// with dbus events when necessary.
pub fn connect() -> Result<(RawFd, RawFd), dbus::Error> {
    let session_fds = SESSION_BUS.with(|session_bus| session_bus.watch_fds());
    let system_fds = SYSTEM_BUS.with(|system_bus| system_bus.watch_fds());
    assert_eq!(session_fds.len(), 1, "Only one fd per dbus connection");
    assert_eq!(system_fds.len(), 1, "Only one fd per dbus connection");

    Ok((session_fds[0].fd(), system_fds[0].fd()))
}

/// Set up the DBus object in Lua so that the user libs can interact with
/// Awesome via DBus.
pub fn lua_init(lua: rlua::Context) -> rlua::Result<()> {
    let dbus_table = lua.create_table()?;
    dbus_table.set(SIGNALS_NAME, lua.create_table()?)?;
    dbus_table.set("request_name", lua.create_function(request_name)?)?;
    dbus_table.set("release_name", lua.create_function(release_name)?)?;
    dbus_table.set("add_match", lua.create_function(add_match)?)?;
    dbus_table.set("remove_match", lua.create_function(remove_match)?)?;
    dbus_table.set("connect_signal", lua.create_function(connect_signal)?)?;
    dbus_table
        .set("disconnect_signal", lua.create_function(disconnect_signal)?)?;
    dbus_table.set("emit_signal", lua.create_function(emit_signal)?)?;
    dbus_table.set("__index", lua.create_function(index)?)?;
    dbus_table.set("__newindex", lua.create_function(newindex)?)?;
    lua.globals().set("dbus", dbus_table)?;
    Ok(())
}

fn get_bus_by_name<'bus>(
    bus_name: &str
) -> rlua::Result<&'bus LocalKey<DBusConnection>> {
    match bus_name {
        "session" => Ok(&SESSION_BUS),
        "system" => Ok(&SYSTEM_BUS),
        v => Err(RuntimeError(format!("Unknown bus type {}", v)))
    }
}

fn dbus_to_lua_value<'lua>(
    lua: rlua::Context<'lua>,
    msg_items: &[MessageItem]
) -> rlua::Result<MultiValue<'lua>> {
    let mut res = Vec::with_capacity(msg_items.len());
    for msg_item in msg_items {
        match msg_item {
            MessageItem::Variant(sub_msg_item) => res
                .extend(dbus_to_lua_value(lua, slice::from_ref(sub_msg_item))?),
            MessageItem::DictEntry(key, value) => {
                res.extend(dbus_to_lua_value(lua, slice::from_ref(key))?);
                res.extend(dbus_to_lua_value(lua, slice::from_ref(value))?);
            },
            MessageItem::Struct(fields) => {
                let struct_table = lua.create_table()?;
                let fields = dbus_to_lua_value(lua, fields.as_slice())?;
                for (index, field) in fields.into_iter().enumerate() {
                    struct_table.set(index + 1, field)?;
                }
                res.push(struct_table.to_lua(lua)?);
            },
            MessageItem::Array(array) => {
                let array_table = lua.create_table()?;
                match array.get(0) {
                    None => {},
                    Some(MessageItem::DictEntry(..)) => {
                        assert!(array.len() % 2 == 0);
                        let keys = array.iter().step_by(2);
                        let values = array.iter().skip(1).step_by(2);
                        for (key, value) in keys.zip(values) {
                            let key =
                                dbus_to_lua_value(lua, slice::from_ref(key))?;
                            assert_eq!(key.len(), 1);
                            let key = key.iter().next().unwrap().to_owned();

                            let value =
                                dbus_to_lua_value(lua, slice::from_ref(value))?;
                            assert_eq!(value.len(), 1);
                            let value = value.iter().next().unwrap().to_owned();

                            array_table.set(key, value)?;
                        }
                    },
                    Some(_) => {
                        for (index, value) in array.into_iter().enumerate() {
                            let value =
                                dbus_to_lua_value(lua, slice::from_ref(value))?;
                            assert_eq!(value.len(), 1);
                            let value = value.iter().next().unwrap();

                            array_table.set(index + 1, value.to_owned())?
                        }
                    }
                }
                res.push(array_table.to_lua(lua)?);
            },
            MessageItem::Bool(v) => res.push(v.to_lua(lua)?),
            MessageItem::Byte(v) => res.push(v.to_lua(lua)?),
            MessageItem::Int16(v) => res.push(v.to_lua(lua)?),
            MessageItem::UInt16(v) => res.push(v.to_lua(lua)?),
            MessageItem::Int32(v) => res.push(v.to_lua(lua)?),
            MessageItem::UInt32(v) => res.push(v.to_lua(lua)?),
            MessageItem::Int64(v) => res.push(v.to_lua(lua)?),
            MessageItem::UInt64(v) => res.push(v.to_lua(lua)?),
            MessageItem::Double(v) => res.push(v.to_lua(lua)?),
            MessageItem::Str(v) => res.push(v.to_owned().to_lua(lua)?),
            _ => res.push(Value::Nil)
        }
    }
    Ok(MultiValue::from_vec(res))
}

/// Converts an `rlua::Value` into a `dbus::MessageItem`.
fn lua_value_to_dbus(
    lua: rlua::Context,
    typ: Value,
    value: Value
) -> rlua::Result<MessageItem> {
    let typ = match typ {
        Value::String(s) => s.to_str()?.to_string(),
        _ => {
            return Err(RuntimeError("D-Bus type name was not a string".into()))
        },
    };
    let is_ascii: bool = typ.chars().all(|c| char::is_ascii(&c));
    if typ.len() != 1 || !is_ascii {
        return Err(RuntimeError(format!("{} is an invalid type name", typ)));
    }
    let typ = ArgType::from_i32(typ.chars().next().unwrap() as i32).map_err(
        |_| RuntimeError(format!("{} is an invalid type name", typ))
    )?;

    match (typ, value) {
        (ArgType::Array, Value::Table(value)) => {
            let size = value.len()?;
            if size % 2 != 0 {
                return Err(RuntimeError(
                    "your D-Bus signal handling method returned \
                     wrong number of arguments"
                        .into()
                ));
            }
            let types = value.clone().sequence_values().step_by(2);
            let values = value.clone().sequence_values().skip(1).step_by(2);
            let mut list = Vec::with_capacity(size as usize);
            for (typ, value) in types.zip(values) {
                list.push(lua_value_to_dbus(lua, typ?, value?)?)
            }
            MessageItem::new_array(list)
                .map_err(|_| RuntimeError("Empty list is invalid".into()))
        },
        (ArgType::Boolean, Value::Boolean(value)) => Ok(value.into()),
        (ArgType::String, Value::String(value)) => Ok(value.to_str()?.into()),
        (ArgType::Byte, Value::Integer(value)) |
        (ArgType::Int16, Value::Integer(value)) |
        (ArgType::UInt16, Value::Integer(value)) |
        (ArgType::Int32, Value::Integer(value)) |
        (ArgType::UInt32, Value::Integer(value)) |
        (ArgType::Int64, Value::Integer(value)) |
        (ArgType::UInt64, Value::Integer(value)) |
        (ArgType::Double, Value::Integer(value)) => Ok(value.into()),
        (ArgType::Byte, Value::Number(value)) |
        (ArgType::Int16, Value::Number(value)) |
        (ArgType::UInt16, Value::Number(value)) |
        (ArgType::Int32, Value::Number(value)) |
        (ArgType::UInt32, Value::Number(value)) |
        (ArgType::Int64, Value::Number(value)) |
        (ArgType::UInt64, Value::Number(value)) |
        (ArgType::Double, Value::Number(value)) => Ok(value.into()),
        (typ, value) => Err(RuntimeError(format!(
            "Invalid type {:?} or value {:?}",
            typ, value
        )))
    }
}

fn request_name(
    _: rlua::Context,
    (bus, name): (String, String)
) -> rlua::Result<bool> {
    let bus = get_bus_by_name(bus.as_str())?;
    bus.with(|bus| {
        bus.register_name(name.as_str(), 0)
            .expect(&format!("Could not register name {}", name.as_str()));
    });
    Ok(true)
}

fn release_name(
    _: rlua::Context,
    (bus, name): (String, String)
) -> rlua::Result<bool> {
    let bus = get_bus_by_name(bus.as_str())?;
    bus.with(|bus| {
        bus.release_name(name.as_str())
            .expect(&format!("Could not release name {}", name.as_str()));
    });
    Ok(true)
}

fn add_match(
    _: rlua::Context,
    (bus, name): (String, String)
) -> rlua::Result<()> {
    let bus = get_bus_by_name(bus.as_str())?;
    bus.with(|bus| {
        bus.add_match(name.as_str())
            .map_err(|err| RuntimeError(format!("{}", err)))
    })?;
    Ok(())
}

fn remove_match(
    _: rlua::Context,
    (bus, name): (String, String)
) -> rlua::Result<()> {
    let bus = get_bus_by_name(bus.as_str())?;
    bus.with(|bus| {
        bus.remove_match(name.as_str())
            .map_err(|err| RuntimeError(format!("{}", err)))
    })?;
    Ok(())
}

fn connect_signal<'lua>(
    lua: rlua::Context<'lua>,
    (name, func): (String, rlua::Function<'lua>)
) -> rlua::Result<MultiValue<'lua>> {
    let signals = lua
        .globals()
        .get::<_, Table>("dbus")
        .unwrap()
        .get::<_, Table>(SIGNALS_NAME)
        .unwrap();
    if signals.get(name.as_str())? {
        let error_msg = format!(
            "Cannot add signal {} on D-Bus, already exists",
            name.as_str()
        );
        warn!("{}", error_msg);
        (rlua::Nil, error_msg).to_lua_multi(lua)
    } else {
        signal::connect_signals(lua, signals, &name, &[func])?;
        (true.to_lua_multi(lua))
    }
}

fn disconnect_signal(
    lua: rlua::Context,
    (name, _func): (String, rlua::Function)
) -> rlua::Result<()> {
    let signals: Table = lua
        .globals()
        .get::<_, Table>("dbus")
        .unwrap()
        .get(SIGNALS_NAME)
        .unwrap();
    signal::disconnect_signals(lua, signals, &name)
}

fn emit_signal<'lua>(
    lua: rlua::Context<'lua>,
    (bus, path, interface, name, args): (
        String,
        String,
        String,
        String,
        MultiValue
    )
) -> rlua::Result<Value<'lua>> {
    if args.len() % 2 != 0 {
        let error_msg =
            "your D-Bus signal emitting metod has wrong number of arguments";
        warn!("{}", error_msg);
        return false.to_lua(lua);
    }
    let types = args.iter().step_by(2);
    let values = args.iter().skip(1).step_by(2);
    let args = types
        .zip(values)
        .map(|(typ, val)| {
            lua_value_to_dbus(lua, typ.to_owned(), val.to_owned())
        })
        .collect::<rlua::Result<Vec<MessageItem>>>()?;

    let bus = get_bus_by_name(bus.as_str())?;
    bus.with::<_, rlua::Result<_>>(|bus| {
        let mut msg =
            Message::new_signal(path, interface, name).map_err(|err| {
                RuntimeError(format!(
                    "Your D-Bus signal emitting \
                     method had a bad argument type. {}",
                    err
                ))
            })?;
        msg.append_items(&args);
        bus.send(msg)
            .map_err(|_| RuntimeError("Could not send D-Bus message".into()))
    })?;
    true.to_lua(lua)
}

// TODO This is the default class index/newindex, move there

fn index<'lua>(
    lua: rlua::Context<'lua>,
    args: Value<'lua>
) -> rlua::Result<()> {
    signal::global_emit_signal(lua, ("debug::index::miss".into(), args))
}

fn newindex<'lua>(
    lua: rlua::Context<'lua>,
    args: Value<'lua>
) -> rlua::Result<()> {
    signal::global_emit_signal(lua, ("debug::newindex::miss".into(), args))
}
