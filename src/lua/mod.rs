//! Lua functionality

use hlua::{Lua, LuaError};

use std::fmt::{Debug, Formatter};
use std::fmt::Result as FmtResult;

use std::thread;
use std::fs::{File};
use std::path::Path;
use std::io::Write;

use std::sync::{Mutex, RwLock};
use std::sync::mpsc::{channel, Sender, Receiver};

#[macro_use]
mod funcs;
#[cfg(test)]
mod tests;

mod types;
pub use self::types::{LuaQuery, LuaFunc, LuaResponse};

lazy_static! {
    /// Sends requests to the lua thread
    static ref SENDER: Mutex<Option<Sender<LuaMessage>>> = Mutex::new(None);

    /// Whether the lua thread is currently running
    pub static ref RUNNING: RwLock<bool> = RwLock::new(false);
}


/// Struct sent to the lua query
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
pub fn send(query: LuaQuery) -> Result<Receiver<LuaResponse>, LuaSendError> {
    if !thread_running() {
        return Err(LuaSendError::ThreadClosed);
    }
    let thread_sender: Sender<LuaMessage>;
    {
        let maybe_sender = SENDER.lock().unwrap();
        match *maybe_sender {
            Some(ref real_sender) => {
                thread_sender = real_sender.clone();
            },
            // If the sender doesn't exist yet, the thread doesn't either
            None => {
                return Err(LuaSendError::ThreadUninitialized);
            }
        }
    }
    let (tx, rx) = channel();
    let message = LuaMessage { reply: tx, query: query };
    match thread_sender.send(message) {
        Ok(_) => Ok(rx),
        Err(e) => Err(LuaSendError::Sender(e.0.query))
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
        LuaQuery::ExecRust(func) => {
            let result = func(lua);
            thread_send(request.reply, LuaResponse::Variable(Some(result)));
        },
        LuaQuery::Ping => {
            thread_send(request.reply, LuaResponse::Pong);
        },
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
