mod xdg;
mod xdg_v6;

pub use self::{xdg::*, xdg_v6::*};

use wlroots::{Area, HandleResult, SurfaceHandle, XdgShellSurfaceHandle, XdgV6ShellSurfaceHandle};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Shell {
    XdgV6(XdgV6ShellSurfaceHandle),
    Xdg(XdgShellSurfaceHandle)
}

impl Shell {
    /// Get a wlr surface from the shell.
    pub fn surface(&self) -> SurfaceHandle {
        match self.clone() {
            Shell::XdgV6(shell) => shell.run(|shell| shell.surface())
                                        .expect("An xdg v6 client did not provide us a surface"),
            Shell::Xdg(shell) => shell.run(|shell| shell.surface())
                                      .expect("An xdg client did not provide us a surface")
        }
    }

    /// Get the geometry of a shell.
    pub fn geometry(&mut self) -> HandleResult<Area> {
        match *self {
            Shell::XdgV6(ref mut shell) => shell.run(|shell| shell.geometry()),
            Shell::Xdg(ref mut shell) => shell.run(|shell| shell.geometry())
        }
    }
}

impl Into<Shell> for XdgV6ShellSurfaceHandle {
    fn into(self) -> Shell { Shell::XdgV6(self) }
}

impl Into<Shell> for XdgShellSurfaceHandle {
    fn into(self) -> Shell { Shell::Xdg(self) }
}
