//! Lua functionality

use hlua;
use hlua::{Lua, LuaError};
use hlua::any::AnyLuaValue;

use std::thread;

use std::fs::{File};
use std::path::Path;
use std::io::Write;

use std::sync::{Mutex, RwLock};
use std::sync::mpsc::{channel, Sender, Receiver};

lazy_static! {
    /// Sends requests to the lua thread
    static ref SENDER: Mutex<Sender<LuaQuery>> = {
        let (tx, rx) = channel::<LuaQuery>();
        Mutex::new(tx)
    };

    /// Receives data back from the lua thread
    /// This should only be accessed by the lua thread itself.
    static ref RECEIVER: Mutex<Receiver<LuaResponse>> = {
        let (tx, rx) = channel::<LuaResponse>();
        Mutex::new(rx)
    };

    /// Whether the lua thread is currently running
    pub static ref RUNNING: RwLock<bool> = RwLock::new(false);
}

/// Messages sent to the lua thread
pub enum LuaQuery {
    /// Halt the lua thread
    Terminate,
    // Restart the lua thread
    Restart,
    /// Execute a string
    Execute(String),
    /// Execute a file
    ExecuteFile(String),
    /// Get a variable
    GetVariable(String),
    /// Set a value
    SetValue {
        name: Box<::std::borrow::Borrow<str> + Sized>,
        val: Box<hlua::Push<&'static mut Lua<'static>> + Sized>
    },
    /// Create a new array
    EmptyArray(String),
    /// Unused send type
    Unused,
}

/// Messages received from lua thread
pub enum LuaResponse {
    /// Lua variable obtained
    Variable(Option<AnyLuaValue>),
    /// Lua error
    Error(hlua::LuaError),
    /// A function is returned
    Function(hlua::functions_read::LuaFunction<String>)
}

unsafe impl Send for LuaQuery { }
unsafe impl Send for LuaResponse { }
unsafe impl Sync for LuaQuery { }
unsafe impl Sync for LuaResponse { }

/// Whether the lua thread is currently available
pub fn thread_running() -> bool {
    *RUNNING.read().unwrap()
}

/// Errors which may arise from attempting
/// to sending a message to the lua thread.
pub enum LuaSendError {
    ThreadClosed,
    Sender
}

/// Attemps to send a LuaQuery to the lua thread.
pub fn try_send(query: LuaQuery) -> Result<(), LuaSendError> {
    if !thread_running() { Err(LuaSendError::ThreadClosed) }
    else {
        match SENDER.lock().unwrap().send(query) {
            Ok(_) => Ok(()),
            Err(_) => Err(LuaSendError::Sender)
        }
    }
}

/// Sends a value to the lua thread.
///
/// # Panics
/// * If the lua thread is currently not running (check `lua::thread_running()`)
/// * If the sender was unable to send, which should only happen
/// if the lua thread isn't running.
pub fn send(query: LuaQuery) {
    if !thread_running() {
        panic!("lua: Attempted to message the lua thread while it is not active!");
    }
    SENDER.lock().unwrap().send(query).unwrap();
}

/// Initialize the lua thread
pub fn init() {
    trace!("Initializing...");
    let (query_tx, query_rx) = channel::<LuaQuery>();
    let (answer_tx, answer_rx) = channel::<LuaResponse>();
    {
        let mut sender = SENDER.lock().unwrap();
        let mut receiver = RECEIVER.lock().unwrap();

        *sender = query_tx;
        *receiver = answer_rx;
    }

    thread::spawn(move || {
        trace!("thread: Inside thread!");
        let sender = answer_tx;
        let receiver = query_rx;
        let mut lua = Lua::new();
        trace!("thread: Loading libraries...");
        lua.openlibs();
        trace!("thread: Libraries loaded");
        *RUNNING.write().unwrap() = true;
        trace!("thread: Testing hello world...");
        let mut file = File::create("/tmp/init.lua").unwrap();
        file.write(b"print('Hello world!')").unwrap();
        lua.execute_from_reader::<(), File>(File::open("/tmp/init.lua")
                                            .unwrap()).unwrap();
        trace!("thread: Done!");
        trace!("thread: Entering loop...");
        thread_main_loop(sender, receiver, &mut lua);
    });
    trace!("Created thread. Init finished.");
}

fn thread_main_loop(sender: Sender<LuaResponse>, receiver: Receiver<LuaQuery>,
                    lua: &mut Lua) {
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
                thread_handle_message(&sender, message, lua);
            }
        }
    }
}

fn thread_handle_message(sender: &Sender<LuaResponse>, request: LuaQuery, lua: &mut Lua) {
    match request {
        LuaQuery::Terminate => {
            trace!("thread: Received terminate signal");
            *RUNNING.write().unwrap() = false;

            info!("thread: Lua thread terminating!");
            return;
        },

        LuaQuery::Restart => {
            trace!("thread: Received restart signal!");
            error!("thread: Lua thread restart not supported!");

            *RUNNING.write().unwrap() = false;

            panic!("Lua thread: Restart not supported!");
        },

        LuaQuery::Execute(code) => {
            trace!("thread: Received request to execute code");
            trace!("thread: Executing {:?}", code);

            match lua.execute::<()>(&code) {
                Err(error) => {
                    warn!("thread: Error executing code: {:?}", error);
                    let response = LuaResponse::Error(error);

                    thread_send(&sender, response);
                }
                Ok(_) => {
                    // This is gonna be really spammy one day
                    trace!("thread: Code executed okay.");
                }
            }
        },

        LuaQuery::ExecuteFile(name) => {
            trace!("thread: Received request to execute file {}", name);
            info!("thread: Executing {}", name);

            let path = Path::new(&name);
            let try_file = File::open(path);

            if let Ok(file) = try_file {
                let result = lua.execute_from_reader::<(), File>(file);
                if let Err(err) = result {
                    warn!("thread: Error executing {}!", name);

                    thread_send(&sender, LuaResponse::Error(err));
                }
                else {
                    trace!("thread: Execution of {} successful.", name);
                }
            }
            else { // Could not open file
                // Unwrap_err is used because we're in the else of let
                let read_error =
                    LuaError::ReadError(try_file.unwrap_err());

                thread_send(&sender, LuaResponse::Error(read_error));
            }
        },

        LuaQuery::GetVariable(varname) => {
            trace!("thread: Received request to get variable {}", varname);
            let var_result = lua.get(varname.as_str());

            match var_result {
                Some(var) => {
                    thread_send(&sender, LuaResponse::Variable(Some(var)));
                }
                None => {
                    warn!("thread: Unable to get variable {}", varname);

                    thread_send(&sender, LuaResponse::Variable(None));
                }
            }
        },

        LuaQuery::SetValue { name: name, val: val } => {
        },

        LuaQuery::EmptyArray(name) => {
            
        },

        _ => {
            panic!("Unimplemented send type for lua thread!");
        }
    }
}

fn thread_send(sender: &Sender<LuaResponse>, response: LuaResponse) {
    trace!("Called thread_send");
    match sender.send(response) {
        Err(e) => {
            error!("thread: Unable to broadcast response!");
            error!("thread: Shutting down in response to inability \
                    to continue!");
            panic!("Lua thread unable to communicate with main thread, \
                    shutting down!");
        }
        Ok(_) => {}
    }
}
