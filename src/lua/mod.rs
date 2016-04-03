//! Lua functionality

use hlua;
use hlua::Lua;
use std::thread;
use std::time::Duration;
use std::fs::{File};
use std::io::Write;
use std::sync::Mutex;
use std::sync::mpsc::{channel, Sender, Receiver};

lazy_static! {
    static ref SENDER: Mutex<Sender<LuaQuery>> = {
        let (tx, rx) = channel::<LuaQuery>();
        Mutex::new(tx)
    };
    static ref RECEIVER: Mutex<Receiver<LuaResponse>> = {
        let (tx, rx) = channel::<LuaResponse>();
        Mutex::new(rx)
    };
}


/// Messages sent to the lua thread
pub enum LuaQuery {
    CallFunction(String),
    GetVariable(String)
}

/// Messages received from lua thread
pub enum LuaResponse {
    Error(hlua::LuaError),
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
        let mut LUA = Lua::new();
        print!("[lua] Loading libs...");
        LUA.openlibs();
        println!(" done!");
        println!("[lua] Creating init file");
        let mut file = File::create("/tmp/init.lua").unwrap();
        file.write(b"print('Hello world!')").unwrap();
        println!("[lua] Created hello world file!");
        println!("[lua] Executing init.lua...");
        LUA.execute_from_reader::<(), File>(File::open("/tmp/init.lua").unwrap());
        println!("[lua] Done!");
        thread::sleep(Duration::from_secs(10));
        println!("[lua] Exiting thread...");
    });
    println!(" created thread.")
}
