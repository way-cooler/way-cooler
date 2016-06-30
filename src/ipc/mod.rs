//! IPC for way-cooler

use std::thread;
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use std::fs;

use unix_socket::UnixListener;

mod channel;
mod command;
mod event;

#[cfg(test)]
mod tests;

/// Versions are incremented.
pub const VERSION: u64 = 0u64; // Increment to 1 on release.

/// Socket over which synchronous communication is made with clients.
pub const COMMAND_SOCKET: &'static str = "command";
/// Socket over which events are sent to clients.
pub const EVENT_SOCKET: &'static str = "event";
/// Folder in which sockets are created
pub const PATH_VAR: &'static str = "WAY_COOLER_SOCKET_FOLDER";

lazy_static! {
    static ref ID: u32 = unique_ish_id();
    static ref BASE_PATH: PathBuf = {
        let mut path = socket_base_path();
        path.push("way-cooler");
        path.push(format!("{}", *ID));
        path
    };
}

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

/// Gets the base path which the socket uses
fn socket_base_path() -> PathBuf {
    if let Ok(path) = env::var("XDG_RUNTIME_DIR") {
        return PathBuf::from(path)
    }

    let user = env::var("USER")
        .expect("$USER environment variable undefined!");

    let output = Command::new("id")
        .arg("-u").arg(user)
        .output()
        .unwrap_or_else(|e| panic!("Failed to get user ID: {}", e));

    if !output.status.success() {
        panic!("Executing id command failed!")
    }

    match String::from_utf8(output.stdout) {
        Ok(id) => return Path::new("/var/run/user").join(id),
        Err(err) => panic!("Unable to parse user id: {:?}", err)
    }
}

/// Initialize the IPC socket.
pub fn init() {
    info!("Starting IPC with unique ID {}", *ID);

    if let Err(ioerr) = fs::create_dir_all(BASE_PATH.clone()) {
        // How can we handle not having a socket?
        // In the future, we could log and continue.
        // We could have a config option to not create/create-if-possible
        error!("Unable to create temp folder: {:?}", ioerr);
        return;
    }
    let command_socket = UnixListener::bind(BASE_PATH.join(COMMAND_SOCKET))
        .expect("Unable to open command socket!");

    let event_socket = UnixListener::bind(BASE_PATH.join(EVENT_SOCKET))
        .expect("Unable to open event socket!");

    env::set_var(PATH_VAR, BASE_PATH.clone());

    debug!("IPC initialized, now listening for clients.");

    let _server_handle = thread::Builder::new()
        .name("Command socket listener".to_string())
        .spawn(move || { command_thread(command_socket) });

    let _event_handle = thread::Builder::new()
        .name("Event socket listener".to_string())
        .spawn(move || { event_thread(event_socket) });

    trace!("IPC initialized.");
}

pub fn terminate() {
    if let Some(path) = BASE_PATH.to_str() {
        trace!("Removing folder {}", path);
    }
    if let Err(ioerr) = fs::remove_dir_all(BASE_PATH.clone()) {
        error!("Unable to delete path: {}", ioerr);
    }
    info!("Cleaned up IPC folder");
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
