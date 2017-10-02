//! Implementations of the default callbacks exposed by wlc.
//! These functions are the main entry points into Way Cooler from user action.
//! This is the default mode that Way Cooler is in at initilization
use std::process::{Command};
use std::path::PathBuf;
use std::io;
use rustwlc::*;
use rustwlc::input::pointer;
use uuid::Uuid;

use super::{Mode, Modes, Default, write_current_mode};
use super::{EVENT_BLOCKED, EVENT_PASS_THROUGH};
use ::render::screen_scrape::{read_screen_scrape_lock, scraped_pixels_lock,
                              sync_scrape};
use ::layout::try_lock_tree;
use ::registry;

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
    view: Option<WlcView>,
    /// Flag that is set when the view is destroyed, either because the user
    /// closed it OR if the program failed to spawn.
    removed: bool
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
            view: None,
            removed: false
        }
    }

    fn view(&self) -> Option<WlcView> {
        self.view
    }

    /// Adds the view to the `LockScreen` if it's PID matches the stored one.
    ///
    /// If there is already here, false is always returned.
    ///
    /// If the view's PID matches this PID, then it's automatically focused,
    /// and the geometry of it is set to the size of it's `WlcOutput`'s size.
    fn add_view_if_match(&mut self, view: WlcView) -> bool {
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
    fn remove_if_match(&mut self, view: WlcView) -> Option<WlcView> {
        if self.view == Some(view) {
            self.removed = true;
            // invalidate this view so we can't do anything to it.
            self.view.take()
        } else {
            None
        }
    }
}

#[allow(unused)]
impl Mode for LockScreen {
    fn output_render_post(&mut self, output: WlcOutput) {
        // Don't allow screen scraping during lock screen
        let need_to_fetch = read_screen_scrape_lock();
        if *need_to_fetch {
            if let Ok(mut scraped_pixels) = scraped_pixels_lock() {
                let resolution = output.get_resolution()
                    .expect("Output had no resolution");
                // Give them zero-ed out pixels
                *scraped_pixels = vec![0; resolution.w as usize * resolution.h as usize * 4];
                sync_scrape();
            }
        }
    }

    fn view_created(&mut self, view: WlcView) -> bool {
        // this will focus and set the size if necessary.
        if self.add_view_if_match(view) {
            trace!("Adding lockscreen");
            update_mode((*self).into());
            true
        } else {
            false
        }
    }

    fn view_destroyed(&mut self, view: WlcView) {
        // Always remove the destroyed view from the tree.
        Default.view_destroyed(view);
        // View is automatically removed from the internal lock screen struct.
        if self.remove_if_match(view).is_some() {
            trace!("Removing lock screen");
            update_mode(Default.into());
            // Update the active container in the tree to the active path end.
            if let Ok(mut tree) = try_lock_tree() {
                tree.reset_focus()
                    .unwrap_or_else(|_| {
                        warn!("Could not reset focus\
                              after removing lock screen");
                    })
            }
        }
    }

    fn view_focused(&mut self, current: WlcView, focused: bool) {
        self.view().map(|v| v.set_state(VIEW_ACTIVATED, focused));
    }

    fn view_request_state(&mut self, view: WlcView, state: ViewState, toggle: bool) {
        // Do nothing
    }

    fn view_request_move(&mut self, view: WlcView, _dest: Point) {
        // Do nothing
    }

    fn view_request_resize(&mut self, view: WlcView, edge: ResizeEdge, point: Point) {
        // Do nothing
    }

    fn on_keyboard_key(&mut self, _view: WlcView, _time: u32, mods: KeyboardModifiers,
                            key: u32, state: KeyState) -> bool {
        self.view().map(WlcView::focus).is_none()
    }

    fn view_request_geometry(&mut self, view: WlcView, geometry: Geometry) {
        // Do nothing
    }

    fn on_pointer_button(&mut self, view: WlcView, _time: u32,
                            mods: KeyboardModifiers, button: u32,
                                state: ButtonState, point: Point) -> bool {
        self.view().map(WlcView::focus).is_none()
    }

    fn on_pointer_scroll(&mut self, _view: WlcView, _time: u32,
                            _mods_ptr: KeyboardModifiers, _axis: ScrollAxis,
                            _heights: [f64; 2]) -> bool {
        self.view().map(WlcView::focus).is_none()
    }

    fn on_pointer_motion(&mut self, view: WlcView, _time: u32, point: Point) -> bool {
        if self.view().map(WlcView::focus).is_some() {
            pointer::set_position(point);
            EVENT_PASS_THROUGH
        } else {
            EVENT_BLOCKED
        }
    }
}

/// Updates the global mode to the this mode.
fn update_mode(mode: Modes) {
    *write_current_mode() = mode
}


#[derive(Debug)]
pub enum LockScreenErr {
    /// IO error while trying to lock screen.
    /// e.g: Couldn't open lock screen program.
    IO(io::Error),
    /// Lock screen was already locked.
    AlreadyLocked
}

/// Locks the screen, with the stored path from the registry.
///
/// If it fails to lock the screen, then a warning is raised in
/// the log and it other wise continues.
///
/// If you want to handle the error yourself, please use
/// `lock_screen_with_path`.
pub fn spawn_lock_screen() {
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
pub fn lock_screen_with_path(path: PathBuf) -> Result<(), LockScreenErr> {
    let mut mode = write_current_mode();
    match *mode {
        Modes::LockScreen(_) =>
            return Err(LockScreenErr::AlreadyLocked),
        _ => {}
    };
    // TODO This can fail badly, and we should use child_try_wait.
    // It's possible the program can fail to spawn a window, which
    // means we'll be waiting forever.
    // HOWEVER that's behind a nightly flag, so we must wait.
    let child = Command::new(path).spawn()
        .map_err(|io_er| LockScreenErr::IO(io_er))?;
    let id = child.id();
    *mode = Modes::LockScreen(LockScreen::new(id as pid_t));
    Ok(())
}
