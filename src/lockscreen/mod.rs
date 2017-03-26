mod lockscreen;

pub use self::lockscreen::LockScreen;

use std::sync::{Mutex, MutexGuard, TryLockError, PoisonError};

type LockScreenGuard = MutexGuard<'static, Option<LockScreen>>;
type LockScreenErr = TryLockError<LockScreenGuard>;

lazy_static! {
    static ref LOCK_SCREEN: Mutex<Option<LockScreen>> = {
        Mutex::new(None)
    };
}

pub fn try_lock_lock_screen() -> Result<LockScreenGuard, LockScreenErr> {
    LOCK_SCREEN.try_lock()
}

pub fn lock_lock_screen() -> Result<LockScreenGuard,
                                    PoisonError<LockScreenGuard>> {
    LOCK_SCREEN.lock()
}
