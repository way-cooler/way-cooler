//! Lua functionality

use hlua;
use hlua::{Lua, LuaError};
use std::thread;
use std::time::Duration;
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
}

/// Messages received from lua thread
pub enum LuaResponse {
    /// Lua error
    Error(hlua::LuaError),
    /// A function is returned
    Function(hlua::functions_read::LuaFunction<String>)
}

unsafe impl Send for LuaQuery { }
unsafe impl Send for LuaResponse { }
unsafe impl Sync for LuaQuery { }
unsafe impl Sync for LuaResponse { }

pub fn init() {
    print!("[lua] Initializing...");
    let (query_tx, query_rx) = channel::<LuaQuery>();
    let (answer_tx, answer_rx) = channel::<LuaResponse>();
    {
        let mut sender = SENDER.lock().unwrap();
        let mut receiver = RECEIVER.lock().unwrap();

        *sender = query_tx;
        *receiver = answer_rx;
    }

    thread::spawn(move || {
        println!("[lua] Inside thread!");
        let receiver = query_rx;
        let sender = answer_tx;
        let mut lua = Lua::new();
        print!("[lua] Loading libs...");
        lua.openlibs();
        println!(" done!");
        println!("[lua] Creating init file");
        let mut file = File::create("/tmp/init.lua").unwrap();
        file.write(b"print('Hello world!')").unwrap();
        println!("[lua] Created hello world file!");
        println!("[lua] Executing init.lua...");
        lua.execute_from_reader::<(), File>(File::open("/tmp/init.lua")
                                            .unwrap()).unwrap();
        println!("[lua] Done!");
        thread::sleep(Duration::from_secs(10));
        println!("[lua] Exiting thread...");
    });
    println!(" created thread.")
}
