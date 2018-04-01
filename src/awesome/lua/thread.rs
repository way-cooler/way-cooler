//! Code for the internal Lua thread which handles all Lua requests.

use std::collections::btree_map::BTreeMap;
use std::thread;
use std::fs::{File};
use std::path::Path;
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::cell::{Cell, RefCell};
use std::sync::{RwLock, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::io::Read;

use glib::MainLoop;
use glib::source::{idle_add, Continue};

use awesome::convert::lua_to_json;

use rustc_serialize::json::Json;
use uuid::Uuid;
use rlua;

use super::types::*;
use super::rust_interop;
use super::init_path;
use awesome::signal;

thread_local! {
    // NOTE The debug library does some powerful reflection that can do crazy things,
    // which is why it's unsafe to load.
    static LUA: RefCell<rlua::Lua> = RefCell::new(unsafe { rlua::Lua::new_with_debug() });
    static MAIN_LOOP: RefCell<MainLoop> = RefCell::new(MainLoop::new(None, false));
}

lazy_static! {
    /// Sends requests to the Lua thread
    static ref CHANNEL: ChannelToLua = ChannelToLua::default();
}


const INIT_LUA_FUNC: &'static str = "way_cooler.on_init()";
const LUA_TERMINATE_CODE: &'static str = "way_cooler.on_terminate()";
const LUA_RESTART_CODE: &'static str = "way_cooler.on_restart()";

/// Struct sent to the Lua query
struct LuaMessage {
    reply: Sender<LuaResponse>,
    query: LuaQuery
}

unsafe impl Send for LuaMessage { }
unsafe impl Sync for LuaMessage { }


impl Debug for LuaMessage {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "LuaMessage({:?})", self.query)
    }
}

/// Struct used to communicate with the Lua thread.
struct ChannelToLua {
    sender: Mutex<Sender<LuaMessage>>,
    receiver: Mutex<Option<Receiver<LuaMessage>>>
}

impl Default for ChannelToLua {
    fn default() -> Self {
        let (sender, receiver) = channel();
        ChannelToLua {
            sender: Mutex::new(sender),
            receiver: Mutex::new(Some(receiver))
        }
    }
}

// Reexported in lua/mod.rs:11
/// Errors which may arise from attempting
/// to sending a message to the Lua thread.
#[derive(Debug)]
pub enum LuaSendError {
    /// The sender had an issue, most likely because the Lua thread panicked.
    /// Following the `Sender` API, the original value sent is returned.
    Sender(LuaQuery)
}

impl Into<rlua::Error> for LuaSendError {
    fn into(self) -> rlua::Error {
        rlua::Error::RuntimeError(match self {
            LuaSendError::Sender(query) =>
                format!("Could not send query to Lua thread: {:?}", query)
        })
    }
}

// Reexported in lua/mod.rs
/// Run a closure with the Lua state. The closure will execute in the Lua thread.
pub fn run_with_lua<F>(func: F) -> rlua::Result<()>
    where F: 'static + FnMut(&rlua::Lua) -> rlua::Result<()>
{
    let send_result = send(LuaQuery::ExecWithLua(Box::new(func))).map_err(|err|err.into())?
        .recv().map_err(|err| {
            rlua::Error::RuntimeError(format!("Could not receive from mspc: {:?}", err))
        })?;
    match send_result {
        LuaResponse::Error(err) => Err(err),
        LuaResponse::Pong => Ok(()),
        _ => Err(rlua::Error::CoroutineInactive)
    }
}

fn emit_refresh(lua: &rlua::Lua) {
    if let Err(err) = signal::global_emit_signal(lua, ("refresh".to_owned(), rlua::Value::Nil)) {
        error!("Internal error while emitting 'refresh' signal: {}", err);
    }
}

fn idle_add_once<F>(func: F)
    where F: Send + 'static + FnOnce() -> ()
{
    let mut cell = Cell::new(Some(func));
    idle_add(move || {
        (&mut cell).get_mut().take().unwrap()();
        Continue(false)
    });
}

// Reexported in lua/mod.rs:11
/// Attempts to send a LuaQuery to the Lua thread.
pub fn send(query: LuaQuery) -> Result<Receiver<LuaResponse>, LuaSendError> {
    // Create a response channel
    let (response_tx, response_rx) = channel();
    match CHANNEL.sender.lock() {
        Err(_) => Err(LuaSendError::Sender(query)),
        Ok(sender) => {
            let message = LuaMessage { reply: response_tx, query: query };
            sender.send(message)
                .map_err(|e| LuaSendError::Sender(e.0.query))
        }
    }?;
    idle_add_once(|| {
        let receiver = CHANNEL.receiver.lock().unwrap();
        if let Some(ref receiver) = *receiver {
            LUA.with(|lua| {
                let lua = &mut *lua.borrow_mut();
                for message in receiver.try_iter() {
                    trace!("Handling a request");
                    if !handle_message(message, lua) {
                        MAIN_LOOP.with(|main_loop| main_loop.borrow().quit())
                    }
                }
                emit_refresh(lua);
            });
        }
    });
    Ok(response_rx)
}

/// Initialize the Lua thread.
pub fn init() {
    info!("Starting Lua thread...");
    let _lua_handle = thread::Builder::new()
        .name("Lua thread".to_string())
        .spawn(|| main_loop());
}

pub fn on_compositor_ready() {
    info!("Running lua on_init()");
    // Call the special init hook function that we read from the init file
    init();
    send(LuaQuery::Execute(INIT_LUA_FUNC.to_owned())).err()
        .map(|error| warn!("Lua init callback returned an error: {:?}", error));
}

fn lua_init() {
    info!("Initializing lua...");
    LUA.with(|lua| {
        load_config(&mut *lua.borrow_mut());
    });
}

fn load_config(mut lua: &mut rlua::Lua) {
    info!("Loading way-cooler libraries...");

    let maybe_init_file = init_path::get_config();
    match maybe_init_file {
        Ok((init_dir, mut init_file)) => {
            if init_dir.components().next().is_some() {
                // Add the config directory to the package path.
                let globals = lua.globals();
                let package: rlua::Table = globals.get("package")
                    .expect("package not defined in Lua");
                let paths: String = package.get("path")
                    .expect("package.path not defined in Lua");
                package.set("path", paths + ";" + init_dir.join("?.lua").to_str()
                            .expect("init_dir not a valid UTF-8 string"))
                    .expect("Failed to set package.path");
            }
            let mut init_contents = String::new();
            init_file.read_to_string(&mut init_contents)
                .expect("Could not read contents");
            lua.exec(init_contents.as_str(), Some("init.lua".into()))
                .map(|_:()| info!("Read init.lua successfully"))
                .or_else(|err| {
                    fn recursive_callback_print(error: ::std::sync::Arc<rlua::Error>) {
                        match *error {
                            rlua::Error::CallbackError {traceback: ref err, ref cause } => {
                                error!("{}", err);
                                recursive_callback_print(cause.clone())
                            },
                            ref err => error!("{:?}", err)
                        }
                    }
                    match err {
                        rlua::Error::RuntimeError(ref err) => {
                            error!("{}", err);
                        }
                        rlua::Error::CallbackError{traceback: ref err, ref cause } => {
                            error!("traceback: {}", err);
                            recursive_callback_print(cause.clone());
                        },
                        err => {
                            error!("init file error: {:?}", err);
                        }
                    }
                    // Keeping this an error, so that it is visible
                    // in release builds.
                    info!("Defaulting to pre-compiled init.lua");
                    unsafe { *lua = rlua::Lua::new_with_debug(); }
                    rust_interop::register_libraries(&mut lua)?;
                    lua.exec(init_path::DEFAULT_CONFIG,
                             Some("init.lua <DEFAULT>".into()))
                })
                .expect("Unable to load pre-compiled init file");
        }
        Err(_) => {
            warn!("Defaulting to pre-compiled init.lua");
            let _: () = lua.exec(init_path::DEFAULT_CONFIG,
                                 Some("init.lua <DEFAULT>".into()))
                .expect("Unable to load pre-compiled init file");
        }
    }
    emit_refresh(lua);
}

struct DropReceiver;

impl Drop for DropReceiver {
    fn drop(&mut self) {
        // Drop the receiver, if possible
        match CHANNEL.receiver.lock() {
            Ok(mut receiver) => {
                let _ = receiver.take();
            },
            _ => {}
        }
    }
}

/// Main loop of the Lua thread:
///
/// * Initialise the Lua state
/// * Run a GMainLoop
fn main_loop() {
    let _guard = DropReceiver;
    LUA.with(|lua| rust_interop::register_libraries(&*lua.borrow()))
        .expect("Could not register lua libraries");
    lua_init();
    MAIN_LOOP.with(|main_loop| main_loop.borrow().run());
}

/// Handle each LuaQuery option sent to the thread
fn handle_message(request: LuaMessage, lua: &mut rlua::Lua) -> bool {
    match request.query {
        LuaQuery::Terminate => {
            trace!("Received terminate signal");
            if let Err(error) = lua.exec::<()>(LUA_TERMINATE_CODE,
                                               Some("custom terminate code".into())) {
                warn!("Lua termination callback returned an error: {:?}", error);
                warn!("However, termination will continue");
            }
            thread_send(request.reply, LuaResponse::Pong);

            info!("Lua thread terminating!");
            return false
        },
        LuaQuery::Restart => {
            trace!("Received restart signal!");
            if let Err(error) = lua.exec::<()>(LUA_RESTART_CODE,
                                         Some("custom restart code".into())) {
                warn!("Lua restart callback returned an error: {:?}", error);
                warn!("However, Lua will be restarted");
            }
            thread_send(request.reply, LuaResponse::Pong);

            info!("Lua thread restarting");
            unsafe { *lua = rlua::Lua::new_with_debug(); }
            rust_interop::register_libraries(lua)
                .expect("Could not register libraries");
            load_config(lua);
            return true;
        },
        LuaQuery::Execute(code) => {
            trace!("Received request to execute {}", code);

            match lua.exec::<()>(&code, None) {
                Err(error) => {
                    warn!("Error executing code: {:?}", error);
                    thread_send(request.reply, LuaResponse::Error(error));
                }
                Ok(_) => {
                    trace!("Code executed okay.");
                    thread_send(request.reply, LuaResponse::Pong);
                }
            }
        },
        LuaQuery::ExecFile(name) => {
            info!("Executing {}", name);

            let path = Path::new(&name);
            let try_file = File::open(path);

            if let Ok(mut file) = try_file {
                let mut file_contents = String::new();
                file.read_to_string(&mut file_contents)
                    .expect("Could not read file contents");
                let result = lua.exec::<rlua::Value>(file_contents.as_str(),
                                      Some(name.as_str()));
                if let Err(err) = result {
                    warn!("Error executing {}!", name);
                    thread_send(request.reply, LuaResponse::Error(err));
                }
                else {
                    trace!("Execution of {} successful.", name);
                    thread_send(request.reply, LuaResponse::Pong);
                }
            }
            else { // Could not open file
                // Unwrap_err is used because we're in the else of let Ok
                let read_error =
                    rlua::Error::RuntimeError(format!("{:#?}", try_file.unwrap_err()));
                thread_send(request.reply, LuaResponse::Error(read_error));
            }
        },
        LuaQuery::ExecRust(func) => {
            let result = func(lua);
            thread_send(request.reply, LuaResponse::Variable(Some(result)));
        },
        LuaQuery::ExecWithLua(mut func) => {
            match func(lua) {
                Ok(()) => thread_send(request.reply, LuaResponse::Pong),
                Err(e) => thread_send(request.reply, LuaResponse::Error(e))
            };
        },
        LuaQuery::Ping => {
            thread_send(request.reply, LuaResponse::Pong);
        },
    }
    return true
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
