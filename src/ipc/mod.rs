//! Lua socket: IPC for way-cooler

use std::thread;

use std::path::Path;
use std::fs;

use unix_socket::{UnixStream, UnixListener};

mod channel;

const SOCKET_PATH_NAME: &'static str = "/tmp/way-cooler/socket";

/// Initialize the Lua server.
pub fn init() {
    trace!("Initializing way-cooler IPC...");
    let _handle = thread::Builder::new()
        .name("Lua socket listener".to_string())
        .spawn(move || { thread_init() });

    trace!("IPC thread created.");
}

fn thread_init() {
    debug!("Creating socket {}", SOCKET_PATH_NAME);

    let socket_path = Path::new(SOCKET_PATH_NAME);
    // Ensure /tmp folder exists - should we do this elsewhere?
    // What else are we going to put in /tmp/way-cooler?
    fs::DirBuilder::new().create("/tmp/way-cooler").ok();
    // Remove the socket if it already exists
    fs::remove_file(socket_path).ok();

    let server_socket = UnixListener::bind(socket_path)
        .expect("Unable to bind to IPC socket!");

    debug!("IPC initialized, now listening for clients.");
    thread_main_loop(server_socket);
}

fn thread_main_loop(socket: UnixListener) {
    for stream in socket.incoming() {
        trace!("New connection: {:?}", stream);
        match stream {
            Ok(stream) => {
                info!("Connected to {:?}", stream);
                let _handle = thread::Builder::new()
                    .name("IPC_helper".to_string())
                    .spawn(move || channel::handle_client(stream));
            },
            Err(err) => {
                info!("Error receiving a stream: {}", err);
            }
        }
    }
}
