//! Lua socket: IPC for way-cooler

use std::thread;

use std::path::Path;
use std::fs;

use unix_socket::{UnixStream, UnixListener};

mod channel;

const SERVER_SOCKET_PATH_NAME: &'static str = "/tmp/way-cooler/server";
const  EVENT_SOCKET_PATH_NAME: &'static str = "/tmp/way-cooler/events";
/// Versions are incremented.
pub const VERSION: u64 = 0u64; // Increment to 1 on release.

/// Initialize the Lua server.
pub fn init() {
    trace!("Initializing way-cooler IPC...");

    debug!("Creating server socket...");
    let server_socket_path = Path::new(SERVER_SOCKET_PATH_NAME);
    let event_socket_path  = Path::new(EVENT_SOCKET_PATH_NAME);
    // Ensure /tmp folder exists - should we do this elsewhere?
    // What else are we going to put in /tmp/way-cooler?
    fs::DirBuilder::new().create("/tmp/way-cooler").ok();
    // Remove the socket if it already exists
    fs::remove_file(server_socket_path).ok();
    fs::remove_file(event_socket_path).ok();

    let server_socket = UnixListener::bind(server_socket_path)
        .expect("Unable to open server socket!");

    let event_socket = UnixListener::bind(event_socket_path)
        .expect("Unable to open event socket!");

    debug!("IPC initialized, now listening for clients.");

    let _server_handle = thread::Builder::new()
        .name("Server socket listener".to_string())
        .spawn(move || { server_thread(server_socket) });

    let _event_handle = thread::Builder::new()
        .name("Event socket listener".to_string())
        .spawn(move || { event_thread(event_socket) });

    trace!("IPC initialized.");
}

fn server_thread(socket: UnixListener) {
    for stream in socket.incoming() {
        trace!("Sever: new connection: {:?}", stream);
        match stream {
            Ok(stream) => {
                info!("Server: connected to {:?}", stream);
                let _handle = thread::Builder::new()
                    .name("IPC server helper".to_string())
                    .spawn(move || channel::handle_command(stream));
            },
            Err(err) => {
                info!("Error receiving a stream: {}", err);
            }
        }
    }
}

fn event_thread(socket: UnixListener) {
    for stream in socket.incoming() {
        trace!("Event: new connection: {:?}", stream);
        match stream {
            Ok(stream) => {
                info!("Event: connected to {:?}", stream);
                let _handle = thread::Builder::new()
                    .name("IPC evemt helper".to_string())
                    .spawn(move || channel::handle_event(stream));
            },
            Err(err) => {
                info!("Error receiving a stream: {}", err);
            }
        }
    }
}
