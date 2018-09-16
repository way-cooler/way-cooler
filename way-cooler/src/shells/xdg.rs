use std::rc::Rc;

use wlroots::{CompositorHandle, Origin, SurfaceHandle, SurfaceHandler,
              XdgShellSurfaceHandle, XdgShellHandler, XdgShellManagerHandler,
              XdgShellState};
use wlroots::xdg_shell_events::{MoveEvent, ResizeEvent};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Xdg {
    shell_surface: XdgShellSurfaceHandle
}

impl Xdg {
    pub fn new() -> Self {
        Xdg { .. Xdg::default()}
    }
}

impl XdgShellHandler for Xdg {
    fn resize_request(&mut self,
                      compositor: CompositorHandle,
                      _: SurfaceHandle,
                      shell_surface: XdgShellSurfaceHandle,
                      event: &ResizeEvent) {
        with_handles!([(compositor: {compositor})] => {
            let server: &mut ::Server = compositor.into();
            let ::Server { ref mut seat,
                         ref mut views,
                         ref mut cursor,
                         .. } = *server;
            let resizing_shell = shell_surface.into();

            if let Some(view) = views.iter().find(|view| view.shell == resizing_shell).cloned() {
                seat.begin_resize(cursor, view.clone(), views, event.edges())
            }
        }).unwrap();
    }

    fn move_request(&mut self,
                    compositor: CompositorHandle,
                    _: SurfaceHandle,
                    shell_surface: XdgShellSurfaceHandle,
                    _: &MoveEvent) {
        with_handles!([(compositor: {compositor})] => {
            let server: &mut ::Server = compositor.into();
            let ref mut seat = server.seat;
            let ref mut cursor = server.cursor;

            if let Some(ref mut view) = seat.focused {
                let shell: ::Shell = shell_surface.into();
                let action = &mut seat.action;
                if view.shell == shell {
                    with_handles!([(cursor: {cursor})] => {
                        let (lx, ly) = cursor.coords();
                        let Origin { x: shell_x, y: shell_y } = view.origin.get();
                        let (view_sx, view_sy) = (lx - shell_x as f64, ly - shell_y as f64);
                        let start = Origin::new(view_sx as _, view_sy as _);
                        *action = Some(::Action::Moving { start: start });
                    }).unwrap();
                }
            }
        }).unwrap();
    }

    fn on_commit(&mut self,
                 compositor: CompositorHandle,
                 _: SurfaceHandle,
                 shell_surface: XdgShellSurfaceHandle) {
        let configure_serial = {
            with_handles!([(shell_surface: {shell_surface.clone()})] => {
                shell_surface.configure_serial()
            }).unwrap()
        };

        let surface = shell_surface.into();
        with_handles!([(compositor: {compositor})] => {
            let server: &mut ::Server = compositor.into();
            let ::Server { ref mut views, .. } = *server;

            if let Some(view) = views.iter().find(|view| view.shell == surface).cloned() {
                if let Some(move_resize) = view.pending_move_resize.get() {
                    if move_resize.serial >= configure_serial {
                        let Origin {mut x, mut y} = view.origin.get();
                        if move_resize.update_x {
                            x  = move_resize.area.origin.x + move_resize.area.size.width -
                                 view.get_size().width;
                        }
                        if move_resize.update_y {
                            y  = move_resize.area.origin.y + move_resize.area.size.height -
                                 view.get_size().height;
                        }

                        view.origin.set(Origin { x, y });

                        if move_resize.serial == configure_serial {
                            view.pending_move_resize.set(None);
                        }
                    }
                }
            }
        }).unwrap();
    }

    fn destroyed(&mut self,
                 compositor: CompositorHandle,
                 shell_surface: XdgShellSurfaceHandle) {
        let surface = shell_surface.into();
        dehandle!(
            @compositor = {compositor};
            let server: &mut ::Server = compositor.into();
            let ::Server { ref mut views, .. } = *server;
            if let Some(index) = views.iter().position(|view| view.shell == surface) {
                views.remove(index);
            }
        );
    }
}

pub struct XdgShellManager;

impl XdgShellManagerHandler for XdgShellManager {
    fn new_surface(&mut self,
                   compositor: CompositorHandle,
                   xdg_surface: XdgShellSurfaceHandle)
                   -> (Option<Box<XdgShellHandler>>, Option<Box<SurfaceHandler>>) {
        dehandle!(
            @compositor = {compositor};
            let server: &mut ::Server = compositor.into();
            let ::Server { ref mut views, .. } = *server;
            views.push(Rc::new(::View::new(::Shell::Xdg(xdg_surface.clone().into()))));
            @xdg_surface = {xdg_surface};
            match xdg_surface.state().unwrap() {
                XdgShellState::TopLevel(toplevel) => {
                    error!("CLIENT PENDING: {:#?}", toplevel.client_pending_state());
                    warn!("SERVER PENDING: {:#?}", toplevel.server_pending_state());
                },
                _ => unimplemented!()
            }
        );
        error!("Added this mother fucker");
        (Some(Box::new(::Xdg::new())), None)
    }
}
