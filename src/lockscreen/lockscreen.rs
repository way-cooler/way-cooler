use rustwlc::{ResizeEdge, WlcView, Geometry, Point};
use rustwlc::types::VIEW_ACTIVATED;
use nix::libc::pid_t;

/// A lock screen program, that has been spawned by Way Cooler.
///
/// The PID is what is returned from spawning the subprocess.
///
/// The view is optional, because after spawning it we must find it in the
/// `view_created` callback.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LockScreen {
    /// Pid of the lock screen program.
    pid: pid_t,
    /// The view that is associated with the lock screen program, if it has been located.
    view: Option<WlcView>
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

    pub fn view(&self) -> Option<WlcView> {
        self.view
    }

    /// Adds the view to the `LockScreen` if it's PID matches the stored one.
    ///
    /// If there is already here, false is always returned.
    ///
    /// If the view's PID matches this PID, then it's automatically focused,
    /// and the geometry of it is set to the size of it's `WlcOutput`'s size.
    pub fn add_view_if_match(&mut self, view: WlcView) -> bool {
        if self.view.is_some() {
            return false
        }
        if view.get_pid() == self.pid {
            let output = view.get_output();
            let resolution = output.get_resolution()
                .expect("Output was invalid: had no resolution");
            let geo = Geometry {
                origin: Point { x: 0, y: 0 },
                size: resolution
            };
            view.set_geometry(ResizeEdge::empty(), geo);
            view.set_state(VIEW_ACTIVATED, true);
            view.focus();
            view.set_mask(1);
            view.bring_to_front();
            self.view = Some(view);
            true
        } else {
            false
        }
    }

    /// Removes the lockscreen if provided view matches the stored view.
    ///
    /// Does not compare the PID, because the passed in `WlcView` should be
    /// considered invalidated because this function is probably being called
    /// from `view_destroyed`.
    ///
    /// To protect against failing programs, the child handler should be
    /// checked to see if the program is still running.
    /// HOWEVER, this is blocked because this is still a nightly feature.
    /// See https://github.com/rust-lang/rust/issues/38903
    pub fn remove_if_match(self, view: WlcView) -> Option<Self> {
        if self.view == Some(view) {
            None
        } else {
            Some(self)
        }
    }
}
