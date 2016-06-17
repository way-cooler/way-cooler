//! IPC for way-cooler

use std::thread;
use std::env;
use std::path::PathBuf;
use std::fs;

use unix_socket::UnixListener;

mod channel;
mod command;
mod event;

#[cfg(test)]
mod tests;

/// Versions are incremented.
pub const VERSION: u64 = 0u64; // Increment to 1 on release.

/// Very much not cross-platform!
/// Submit an issue when Wayland is ported to Windoze.
pub const TEMP_FOLDER: &'static str = "/run/way-cooler/";
/// Socket over which synchronous communication is made with clients.
pub const COMMAND_SOCKET: &'static str = "command";
/// Socket over which events are sent to clients.
pub const EVENT_SOCKET: &'static str = "event";
/// Folder in which sockets are created
pub const PATH_VAR: &'static str = "WAY_COOLER_SOCKET_FOLDER";
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
    (hasher.finish() >> MAGIC_SHIFT_NUMBER) as u32
}

/// Initialize the Lua server.
pub fn init() {
    trace!("Initializing way-cooler IPC...");
    let id = unique_ish_id();
    info!("Starting IPC with unique ID {}", id);

    let mut path = PathBuf::from(TEMP_FOLDER);
    path.push(id.to_string());

    if let Err(ioerr) = fs::create_dir_all(path.clone()) {
        // How can we handle not having a socket?
        // In the future, we could log and continue.
        // We could have a config option to not create/create-if-possible
        error!("Unable to create temp folder: {:?}", ioerr);
        return;
    }
    else {
        let command_socket = UnixListener::bind(path.join(COMMAND_SOCKET))
            .expect("Unable to open command socket!");

        let event_socket = UnixListener::bind(path.join(EVENT_SOCKET))
            .expect("Unable to open event socket!");

        env::set_var(PATH_VAR, path.clone());

        debug!("IPC initialized, now listening for clients.");

        let _server_handle = thread::Builder::new()
            .name("Command socket listener".to_string())
            .spawn(move || { command_thread(command_socket) });

        let _event_handle = thread::Builder::new()
            .name("Event socket listener".to_string())
            .spawn(move || { event_thread(event_socket) });

        trace!("IPC initialized.");
    }
}

fn command_thread(socket: UnixListener) {
    for stream in socket.incoming() {
        trace!("Sever: new connection: {:?}", stream);
        match stream {
            Ok(mut stream) => {
                info!("Command: connected to {:?}", stream);
                let _handle = thread::Builder::new()
                    .name("IPC server helper".to_string())
                    .spawn(move || command::thread(&mut stream));
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
            Ok(mut stream) => {
                info!("Event: connected to {:?}", stream);
                let _handle = thread::Builder::new()
                    .name("IPC evemt helper".to_string())
                    .spawn(move || event::thread(&mut stream));
            },
            Err(err) => {
                info!("Error receiving a stream: {}", err);
            }
        }
    }
}
