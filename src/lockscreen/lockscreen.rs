use rustwlc::WlcView;
use nix::libc::pid_t;

/// A lock screen program, that has been spawned by Way Cooler.
///
/// The PID is what is returned from spawning the subprocess.
///
/// The view is optional, because after spawning it we must find it in the
/// `view_created` callback.
pub struct LockScreen {
    /// Pid of the lock screen program.
    pub pid: pid_t,
    /// The view that is associated with the lock screen program, if it has been located.
    pub view: Option<WlcView>
}

impl LockScreen {
    /// Creates a new Lock Screen from the PID
    ///
    /// The view is set to `None`, because we haven't found it yet in the
    /// `view_created` callback. This should be updated once it is found,
    /// so that we can start locking the screen.
    pub fn new(pid: pid_t) -> Self {
        LockScreen {
            pid: pid,
            view: None
        }
    }
}
