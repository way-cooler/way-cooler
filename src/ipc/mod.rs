//! Main module of way-cooler ipc.

use std::sync::Mutex;
use std::sync::mpsc::{self, Sender};
use std::thread;

use dbus::tree::{Factory, Interface, ObjectPath, Tree, MTFn, MethodErr};

mod utils;
mod keybindings;
mod dbus_message;
pub use self::dbus_message::DBusMessage;
mod session;
pub use self::session::DBusSession;

mod layout;
mod pixels;

pub const VERSION: u32 = 1;

type DBusResult<T> = Result<T, MethodErr>;
type DBusObjPath = ObjectPath<MTFn<()>, ()>;
type DBusInterface = Interface<MTFn<()>, ()>;
type DBusFactory = Factory<MTFn<()>>;
type DBusTree = Tree<MTFn<()>, ()>;

lazy_static! {
    static ref SENDER: Mutex<Option<Sender<DBusMessage>>> = Mutex::new(None);
}


pub fn init() {
    let (send, recv) = mpsc::channel();
    let _join = thread::spawn( move || {
        let mut session = DBusSession::create(recv);

        *SENDER.lock().expect("Unable to unlock") = Some(send);

        session.run_thread();
    });
}
