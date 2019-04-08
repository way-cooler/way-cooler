use std::cell::Cell;
use wlroots::{wlroots_dehandle, Area, Origin, Size, SurfaceHandle, XdgShellState, XdgV6ShellState};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PendingMoveResize {
    pub update_x: bool,
    pub update_y: bool,
    pub serial: u32,
    pub area: Area
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct View {
    pub shell: ::Shell,
    pub origin: Cell<Origin>,
    pub pending_move_resize: Cell<Option<PendingMoveResize>>
}

impl View {
    pub fn new(shell: ::Shell) -> View {
        View {
            shell,
            origin: Cell::new(Origin::default()),
            pending_move_resize: Cell::new(None)
        }
    }

    pub fn surface(&self) -> SurfaceHandle {
        self.shell.surface()
    }

    #[wlroots_dehandle(xdg_surface)]
    pub fn activate(&self, activate: bool) {
        match self.shell.clone() {
            ::Shell::XdgV6(xdg_surface_handle) => {
                use xdg_surface_handle as xdg_surface;
                match xdg_surface.state() {
                    Some(&mut XdgV6ShellState::TopLevel(ref mut toplevel)) => {
                        toplevel.set_activated(activate);
                    },
                    _ => unimplemented!()
                }
            },
            ::Shell::Xdg(xdg_surface_handle) => {
                use xdg_surface_handle as xdg_surface;
                match xdg_surface.state() {
                    Some(&mut XdgShellState::TopLevel(ref mut toplevel)) => {
                        toplevel.set_activated(activate);
                    },
                    _ => unimplemented!()
                }
            }
        }
    }

    #[wlroots_dehandle(xdg_surface)]
    pub fn get_size(&self) -> Size {
        match self.shell.clone() {
            ::Shell::XdgV6(xdg_surface_handle) => {
                use xdg_surface_handle as xdg_surface;
                let Area { origin: _, size } = xdg_surface.geometry();
                size
            },
            ::Shell::Xdg(xdg_surface_handle) => {
                use xdg_surface_handle as xdg_surface;
                let Area { origin: _, size } = xdg_surface.geometry();
                size
            }
        }
    }

    #[wlroots_dehandle(xdg_surface)]
    pub fn move_resize(&self, area: Area) {
        let Area {
            origin: Origin { x, y },
            size: Size { width, height }
        } = area;
        let width = width as u32;
        let height = height as u32;

        let Origin { x: view_x, y: view_y } = self.origin.get();

        let update_x = x != view_x;
        let update_y = y != view_y;
        let mut serial = 0;

        match self.shell.clone() {
            ::Shell::XdgV6(xdg_surface_handle) => {
                use xdg_surface_handle as xdg_surface;
                match xdg_surface.state() {
                    Some(&mut XdgV6ShellState::TopLevel(ref mut toplevel)) => {
                        // TODO apply size constraints
                        serial = toplevel.set_size(width, height);
                    },
                    _ => unimplemented!()
                }
            },
            ::Shell::Xdg(xdg_surface_handle) => {
                use xdg_surface_handle as xdg_surface;
                match xdg_surface.state() {
                    Some(&mut XdgShellState::TopLevel(ref mut toplevel)) => {
                        // TODO apply size constraints
                        serial = toplevel.set_size(width, height);
                    },
                    _ => unimplemented!()
                }
            }
        }

        if serial == 0 {
            // size didn't change
            self.origin.set(Origin { x, y });
        } else {
            self.pending_move_resize.set(Some(PendingMoveResize {
                update_x,
                update_y,
                area,
                serial
            }));
        }
    }

    #[wlroots_dehandle(xdg_surface)]
    pub fn for_each_surface(&self, f: &mut FnMut(SurfaceHandle, i32, i32)) {
        match self.shell.clone() {
            ::Shell::XdgV6(xdg_surface_handle) => {
                use xdg_surface_handle as xdg_surface;
                xdg_surface.for_each_surface(f);
            },
            ::Shell::Xdg(xdg_surface_handle) => {
                use xdg_surface_handle as xdg_surface;
                xdg_surface.for_each_surface(f);
            }
        }
    }
}
