//! IPC for way-cooler

use std::thread;
use std::env;
use std::path::Path;
use std::fs;

use unix_socket::UnixListener;

mod channel;

/// Versions are incremented.
pub const VERSION: u64 = 0u64; // Increment to 1 on release.

/// Very much not cross-platform!
/// Submit an issue when Wayland is ported to Windoze.
pub const TEMP_FOLDER: &'static str = "/tmp/way-cooler";
/// Socket over which synchronous communication is made with clients.
pub const COMMAND_SOCKET: &'static str = "command";
/// Socket over which events are sent to clients.
pub const EVENT_SOCKET: &'static str = "event";

/// We need random folder names to place sockets in, but they don't need
/// to be _that_ random.
pub fn unique_ish_id() -> u32 {
    use std::hash::{Hash, Hasher, SipHasher};
    use std::time::Instant;

    // If you shift a u64 hash right by this you get a "checksum",
    // a number which retains some of the entropy of the hash but
    // is small enough to fit a more comfortable file name.
    const MAGIC_SHIFT_NUMBER: u64 = 0b110000;

    // Instant doesn't implement hash, and it's supposed to be an opaque
    // struct, but it does implement debug...
    let now = Instant::now();
    let mut hasher = SipHasher::new();
    format!("{:?}", now).hash(&mut hasher);
    hasher.finish();
}

/// Initialize the Lua server.
pub fn init() {
    trace!("Initializing way-cooler IPC...");
    let id = unique_ish_id();
    info!("Starting IPC with unique ID {}", id);

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
