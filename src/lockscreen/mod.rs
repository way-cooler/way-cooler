mod lockscreen;

pub use self::lockscreen::LockScreen;

use uuid::Uuid;
use nix::libc::pid_t;
use std::process::{Command};
use std::path::PathBuf;
use std::io;
use std::sync::{Mutex, MutexGuard, TryLockError, PoisonError};

use ::registry;

type LockScreenGuard = MutexGuard<'static, Option<LockScreen>>;
type LockScreenErr = TryLockError<LockScreenGuard>;

lazy_static! {
    static ref LOCK_SCREEN: Mutex<Option<LockScreen>> = {
        Mutex::new(None)
    };
}

/// Locks the screen, with the stored path from the registry.
///
/// If it fails to lock the screen, then a warning is raised in
/// the log and it other wise continues.
///
/// If you want to handle the error yourself, please use
/// `lock_screen_with_path`.
pub fn lock_screen() {
    let lock = registry::clients_read();
    let client = lock.client(Uuid::nil()).unwrap();
    let handle = registry::ReadHandle::new(&client);
    let path = handle.read("programs".into()).ok()
        .and_then(|programs| programs.get("lock_screen"))
        .and_then(|lock_screen| lock_screen.as_string());
    match path {
        None => warn!("No lock screen program set!"),
        Some(path) => {
            let path = path.into();
            lock_screen_with_path(path).unwrap_or_else(|err| {
                warn!("Could not lock screen: {:?}", err)
            })
        }
    }
}

/// Locks the screen by spawning the program at the given path.
///
/// This modifies the global `LOCK_SCREEN` singleton,
/// so that it can be used by wlc callbacks.
///
/// If the program could not be spawned, an `Err` is returned.
pub fn lock_screen_with_path(path: PathBuf) -> Result<(), io::Error> {
    let mut lock_screen = lock_lock_screen()
        .expect("LockScreen lock has been poisoned");
    let child = Command::new(path).spawn()?;
    let id = child.id();
    *lock_screen = Some(LockScreen::new(id as pid_t));
    Ok(())
}

pub fn try_lock_lock_screen() -> Result<LockScreenGuard, LockScreenErr> {
    LOCK_SCREEN.try_lock()
}

pub fn lock_lock_screen() -> Result<LockScreenGuard,
                                    PoisonError<LockScreenGuard>> {
    LOCK_SCREEN.lock()
}
