use compositor::Shell;
use wlroots::XdgV6ShellState::*;
use wlroots::{Origin, SurfaceHandle};
use std::cell::Cell;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct View {
    pub shell: Shell,
    pub origin: Cell<Origin>
}

impl View {
    pub fn new(shell: Shell) -> View {
        View { shell: shell,
               origin: Cell::new(Origin::default()) }
    }

    pub fn surface(&self) -> SurfaceHandle {
        match self.shell {
            Shell::XdgV6(ref xdg_surface) => {
                with_handles!([(xdg_surface: {xdg_surface})] => {
                    xdg_surface.surface()
                }).unwrap()
            }
        }
    }

    pub fn activate(&self, activate: bool) {
        match self.shell {
            Shell::XdgV6(ref xdg_surface) => {
                with_handles!([(xdg_surface: {xdg_surface})] => {
                    match xdg_surface.state() {
                        Some(&mut TopLevel(ref mut toplevel)) => {
                            toplevel.set_activated(activate);
                        },
                        _ => unimplemented!()
                    }
                }).unwrap();
            }
        }
    }

    pub fn for_each_surface(&self, f: &mut FnMut(SurfaceHandle, i32, i32)) {
        match self.shell {
            Shell::XdgV6(ref xdg_surface) => {
                with_handles!([(xdg_surface: {xdg_surface})] => {
                    xdg_surface.for_each_surface(f);
                }).unwrap();
            }
        }
    }
}
