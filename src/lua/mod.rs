//! Lua functionality

use hlua;
use hlua::{Lua, LuaError, LuaTable, PushGuard};
use hlua::any::AnyLuaValue;

use rustc_serialize::json::Json;

use std::thread;
use std::fs::{File};
use std::path::Path;
use std::io::Write;
use std::fmt::{Debug, Formatter};
use std::fmt::Result as FmtResult;
use std::borrow::Borrow;

use std::sync::{Mutex, RwLock};
use std::sync::mpsc::{channel, Sender, Receiver};

#[macro_use]
mod funcs;
#[cfg(test)]
mod tests;

lazy_static! {
    /// Sends requests to the lua thread
    static ref SENDER: Mutex<Option<Sender<LuaMessage>>> = Mutex::new(None);

    /// Whether the lua thread is currently running
    pub static ref RUNNING: RwLock<bool> = RwLock::new(false);
}

/// Represents an identifier for dealing with nested tables.
///
/// To access foo.bar.baz, use vec!["foo", "bar", "baz"].
///
/// To access foo[2], use vec!["foo", 2].
pub type LuaIdent = Vec<String>;

/// Messages sent to the lua thread
#[derive(Debug)]
pub enum LuaQuery {
    /// Pings the lua thread
    Ping,
    /// Halt the lua thread
    Terminate,
    // Restart the lua thread
    Restart,

    /// Execute a string
    Execute(String),
    /// Execute a file
    ExecFile(String),

    /// Get a variable, expecting an AnyLuaValue
    GetValue(LuaIdent),
    /// Invoke a function found at the position,
    /// with the specified arguments.
    Invoke(LuaIdent, Vec<AnyLuaValue>),
    /// Set a value
    SetValue(LuaIdent, Json),
    /// Create a new table
    NewTable(LuaIdent),
}

/// Messages received from lua thread
pub enum LuaResponse {
    /// If the identifier had length 0
    InvalidName,
    /// Lua variable obtained
    Variable(Option<AnyLuaValue>),
    /// Lua error
    Error(hlua::LuaError),
    /// A function is returned
    Function(hlua::functions_read::LuaFunction<String>),
    /// Pong response from lua ping
    Pong,
}

/// Struct sent to the lua query
struct LuaMessage {
    reply: Sender<LuaResponse>,
    query: LuaQuery
}

unsafe impl Send for LuaQuery { }
unsafe impl Sync for LuaQuery { }
unsafe impl Send for LuaResponse { }
unsafe impl Sync for LuaResponse { }
unsafe impl Send for LuaMessage { }
unsafe impl Sync for LuaMessage { }

impl Debug for LuaResponse {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            LuaResponse::InvalidName =>
                write!(f, "LuaResponse::InvalidName"),
            LuaResponse::Variable(ref var) =>
                write!(f, "LuaResponse::Variable({:?})", var),
            LuaResponse::Error(ref err) =>
                write!(f, "LuaResponse::Error({:?})", err),
            LuaResponse::Function(_) =>
                write!(f, "LuaResponse::Function"),
            LuaResponse::Pong =>
                write!(f, "LuaResponse::Pong")
        }
    }
}

impl Debug for LuaMessage {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "LuaMessage({:?})", self.query)
    }
}

/// Whether the lua thread is currently available
pub fn thread_running() -> bool {
    *RUNNING.read().unwrap()
}

/// Errors which may arise from attempting
/// to sending a message to the lua thread.
#[derive(Debug)]
pub enum LuaSendError {
    /// The thread crashed, was shut down, or rebooted.
    ThreadClosed,
    /// The thread has not been initialized yet (maybe not used)
    ThreadUninitialized,
    /// The sender had an issue, most likey because the thread panicked.
    /// Following the `Sender` API, the original value sent is returned.
    Sender(LuaQuery)
}

/// Attemps to send a LuaQuery to the lua thread.
pub fn try_send(query: LuaQuery) -> Result<Receiver<LuaResponse>,LuaSendError> {
    if !thread_running() {
        Err(LuaSendError::ThreadClosed)
    }
    else if let Some(sender) = SENDER.lock().unwrap().clone() {
        let (tx, rx) = channel();
        let message = LuaMessage { reply: tx, query: query };
        match sender.send(message) {
            Ok(_) => Ok(rx),
            Err(e) => Err(LuaSendError::Sender(e.0.query))
        }
    }
    else {
        Err(LuaSendError::ThreadUninitialized)
    }
}

/// Initialize the lua thread
pub fn init() {
    trace!("Initializing...");
    let (query_tx, query_rx) = channel::<LuaMessage>();
    {
        let mut sender = SENDER.lock().unwrap();
        *sender = Some(query_tx);
    }

    thread::spawn(move || {
        thread_init(query_rx);
    });
    trace!("Created thread. Init finished.");
}

fn thread_init(receiver: Receiver<LuaMessage>) {
    trace!("thread: initializing.");
    let mut lua = Lua::new();
    //unsafe {
    //    hlua_ffi::lua_atpanic(&mut lua.as_mut_lua().0, thread_on_panic);
    //}
    debug!("thread: Loading Lua libraries...");
    lua.openlibs();
    trace!("thread: Loading way-cooler lua extensions...");
    // We should have some good file handling, read files from /usr by default,
    // but for now we're reading directly from the source.
    lua.execute_from_reader::<(), File>(
        File::open("lib/lua/init.lua").unwrap()
    ).unwrap();
    trace!("thread: loading way-cooler libraries...");
    funcs::register_libraries(&mut lua);
    // Only ready after loading libs
    *RUNNING.write().unwrap() = true;
    debug!("thread: entering main loop...");
    thread_main_loop(receiver, &mut lua);
}

fn thread_main_loop(receiver: Receiver<LuaMessage>, lua: &mut Lua) {
    loop {
        let request = receiver.recv();
        match request {
            Err(e) => {
                error!("Lua thread: unable to receive message: {}", e);
                error!("Lua thread: now panicking!");
                *RUNNING.write().unwrap() = false;

                panic!("Lua thread: lost contact with host, exiting!");
            }
            Ok(message) => {
                trace!("Handling a request");
                thread_handle_message(message, lua);
            }
        }
    }
}

fn thread_handle_message(request: LuaMessage, lua: &mut Lua) {
    match request.query {
        LuaQuery::Terminate => {
            trace!("thread: Received terminate signal");
            *RUNNING.write().unwrap() = false;

            info!("thread: Lua thread terminating!");
            thread_send(request.reply, LuaResponse::Pong);
            return;
        },

        LuaQuery::Restart => {
            trace!("thread: Received restart signal!");
            error!("thread: Lua thread restart not supported!");

            *RUNNING.write().unwrap() = false;
            thread_send(request.reply, LuaResponse::Pong);

            panic!("Lua thread: Restart not supported!");
        },

        LuaQuery::Execute(code) => {
            trace!("thread: Received request to execute code");
            trace!("thread: Executing {}", code);

            match lua.execute::<()>(&code) {
                Err(error) => {
                    warn!("thread: Error executing code: {:?}", error);
                    thread_send(request.reply, LuaResponse::Error(error));
                }
                Ok(_) => {
                    trace!("thread: Code executed okay.");
                    thread_send(request.reply, LuaResponse::Pong);
                }
            }
        },

        LuaQuery::ExecFile(name) => {
            info!("thread: Executing {}", name);

            let path = Path::new(&name);
            let try_file = File::open(path);

            if let Ok(file) = try_file {
                let result = lua.execute_from_reader::<(), File>(file);
                if let Err(err) = result {
                    warn!("thread: Error executing {}!", name);

                    thread_send(request.reply, LuaResponse::Error(err));
                }
                else {
                    trace!("thread: Execution of {} successful.", name);
                    thread_send(request.reply, LuaResponse::Pong);
                }
            }
            else { // Could not open file
                // Unwrap_err is used because we're in the else of let
                let read_error =
                    LuaError::ReadError(try_file.unwrap_err());

                thread_send(request.reply, LuaResponse::Error(read_error));
            }
        },

        LuaQuery::GetValue(varname) => {
            trace!("thread: Received request to get variable {:?}", varname);

            if varname.len() == 0 {
                thread_send(request.reply, LuaResponse::InvalidName);
                return;
            }
            // Table[0] String had to be cloned, it'd be nice if Rust let us
            // borrow out parts of memory
            match lua.get::<AnyLuaValue, _>(varname[0].clone()) {
                Some(table) => {
                    let full_table = walk_table(table, &varname[1..]);
                    thread_send(request.reply,
                                LuaResponse::Variable(full_table));
                },
                None => thread_send(request.reply, LuaResponse::Variable(None))
            }
        },

        LuaQuery::SetValue(name, val) => {
            trace!("thread: SetValue: Setting {:?} to {:?}", name, val);
            /*
            let maybe_table: Option<LuaTable<_>> = lua.get::<_, _>(name[0]);

            let table: LuaTable<_>;

            if name.len() == 0 {
                thread_send(request.reply, LuaResponse::InvalidName);
            }
            else if name.len() == 1 {
                let lua_val = 
                table.set(name[0], val);
            }
            else {
                match maybe_table {
                    Some(table) => {
                        
                    }
                }
            }*/
            unimplemented!();
        },

        LuaQuery::NewTable(name_list) => {
            panic!("thread: unimplemented LuaQuery::NewTable!");
        },

        LuaQuery::Ping => {
            thread_send(request.reply, LuaResponse::Pong);
        },
        _ => {
            panic!("Unimplemented send type for lua thread!");
        }
    }
}

fn thread_send(sender: Sender<LuaResponse>, response: LuaResponse) {
    match sender.send(response) {
        Err(err) => {
            match err.0 {
                LuaResponse::Pong => {}, // Those are boring
                _ => {
                    warn!("thread: Someone dropped an important Lua response!");
                }
            }
        }
        Ok(_) => {}
    }
}

fn walk_table(table: AnyLuaValue, names: &[String]) -> Option<AnyLuaValue> {
    if let Some(name) = names.first() {
        if let AnyLuaValue::LuaArray(arr) = table {
            for (key, val) in arr {
                if let AnyLuaValue::LuaString(key_str) = key {
                    if *key_str == *name {
                        return walk_table(val, &names[1..]);
                    }
                }
            }
            return None;
        }
        else {
            return None;
        }
    }
    else {
        return Some(table); // ???
    }
}

/*
fn set_value<'l>(lua: &'l mut Lua, mut table: PushGuard<LuaTable<Lua>>,
             names: &[String], val: AnyLuaValue) {
    // Should not be reached!
    if names.len() == 0 {
        return;
    }
    // The last name is the name of the value
    else if names.len() == 1 {
        table.set(names[0].clone(), val);
    }
    else {
        let maybe_table = table.get::<_, _>(names[0]);
        match maybe_table {
            Some(new_table) => {
                set_value(&mut lua, new_table, &names[1..], val);
            }
            None => { return; }
        }
    }
}*/

/// Converts a Json map into an AnyLuaValue
pub fn json_to_lua(json: Json) -> AnyLuaValue {
    match json {
        Json::String(val)  => AnyLuaValue::LuaString(val),
        Json::Boolean(val) => AnyLuaValue::LuaBoolean(val),
        Json::F64(val)     => AnyLuaValue::LuaNumber(val),
        Json::I64(val)     => AnyLuaValue::LuaNumber((val as i32) as f64),
        Json::U64(val)     => AnyLuaValue::LuaNumber((val as u32) as f64),
        Json::Null         => AnyLuaValue::LuaNil,
        Json::Array(vals)  => {
            let mut lua_arr = Vec::with_capacity(vals.len());
            for (ix, val) in vals.into_iter().enumerate() {
                lua_arr.push((AnyLuaValue::LuaNumber(ix as f64 + 1.0),
                              json_to_lua(val)));
            }
            AnyLuaValue::LuaArray(lua_arr)
        },
        Json::Object(vals) => {
            let mut lua_table = Vec::with_capacity(vals.len());
            for (key, val) in vals.into_iter() {
                lua_table.push((AnyLuaValue::LuaString(key),
                                json_to_lua(val)));
            }
            AnyLuaValue::LuaArray(lua_table)
        }
    }
}

