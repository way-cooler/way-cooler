use std::rc::Rc;

use wlroots::{CompositorHandle, Origin, SurfaceHandle, SurfaceHandler,
              XdgShellSurfaceHandle, XdgShellHandler, XdgShellManagerHandler};
use wlroots::xdg_shell_events::{MoveEvent, ResizeEvent};
use wlroots::wlroots_dehandle;

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
    #[wlroots_dehandle(compositor)]
    fn resize_request(&mut self,
                      compositor: CompositorHandle,
                      _: SurfaceHandle,
                      shell_surface: XdgShellSurfaceHandle,
                      event: &ResizeEvent) {
        use compositor as compositor;
        let server: &mut ::Server = compositor.into();
        let ::Server { ref mut seat,
                        ref mut views,
                        ref mut cursor,
                        .. } = *server;
        let resizing_shell = shell_surface.into();

        if let Some(view) = views.iter().find(|view| view.shell == resizing_shell).cloned() {
            seat.begin_resize(cursor, view.clone(), views, event.edges())
        }
    }

    #[wlroots_dehandle(compositor, cursor)]
    fn move_request(&mut self,
                    compositor: CompositorHandle,
                    _: SurfaceHandle,
                    shell_surface: XdgShellSurfaceHandle,
                    _: &MoveEvent) {
        use compositor as compositor;
        let server: &mut ::Server = compositor.into();
        let ref mut seat = server.seat;
        let ref mut cursor = server.cursor;

        if let Some(ref mut view) = seat.focused {
            let shell: ::Shell = shell_surface.into();
            let action = &mut seat.action;
            if view.shell == shell {
                use cursor as cursor;
                let (lx, ly) = cursor.coords();
                let Origin { x: shell_x, y: shell_y } = view.origin.get();
                let (view_sx, view_sy) = (lx - shell_x as f64, ly - shell_y as f64);
                let start = Origin::new(view_sx as _, view_sy as _);
                *action = Some(::Action::Moving { start: start });
            }
        }
    }

    #[wlroots_dehandle(compositor, shell_surface)]
    fn on_commit(&mut self,
                 compositor: CompositorHandle,
                 _: SurfaceHandle,
                 shell_surface: XdgShellSurfaceHandle) {
        let configure_serial = {
            use shell_surface as shell_surface;
            shell_surface.configure_serial()
        };

        let surface = shell_surface.into();
        use compositor as compositor;
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
    }

    #[wlroots_dehandle(shell_surface, compositor, cursor)]
    fn map_request(&mut self,
                   compositor: CompositorHandle,
                   _: SurfaceHandle,
                   shell_surface: XdgShellSurfaceHandle) {
        use wlroots::XdgShellState::*;

        let is_toplevel = {
            // we can't combine this with the next block because it needs to use the handle
            use shell_surface as shell_surface;
            match shell_surface.state().unwrap() {
                TopLevel(_) => true,
                _ => false
            }
        };

        use compositor as compositor;
        let server: &mut ::Server = compositor.into();
        let ::Server { ref mut seat,
                       ref mut views,
                       ref cursor,
                       ref mut xcursor_manager,
                       .. } = *server;
        if is_toplevel {
            let view = Rc::new(::View::new(::Shell::Xdg(shell_surface.into())));
            views.push(view.clone());
            seat.focus_view(view, views);
        };
        use cursor as cursor;
        seat.update_cursor_position(cursor, xcursor_manager, views, None)
    }

    #[wlroots_dehandle(compositor, cursor)]
    fn unmap_request(&mut self,
                     compositor: CompositorHandle,
                     _: SurfaceHandle,
                     shell_surface: XdgShellSurfaceHandle) {
        use compositor as compositor;
        let server: &mut ::Server = compositor.into();
        let ::Server { ref mut seat,
                       ref mut views,
                       ref cursor,
                       ref mut xcursor_manager,
                       .. } = *server;
        let destroyed_shell = shell_surface.into();
        views.retain(|view| view.shell != destroyed_shell);

        if let Some(view) = views.get(0).cloned() {
            seat.focus_view(view, views);
        } else {
            seat.clear_focus();
        };
        use cursor as cursor;
        seat.update_cursor_position(cursor, xcursor_manager, views, None)
    }

    #[wlroots_dehandle(compositor)]
    fn destroyed(&mut self,
                 compositor: CompositorHandle,
                 shell_surface: XdgShellSurfaceHandle) {
        let surface = shell_surface.into();
        use compositor as compositor;
        let server: &mut ::Server = compositor.into();
        let ::Server { ref mut views, .. } = *server;
        if let Some(index) = views.iter().position(|view| view.shell == surface) {
            views.remove(index);
        }
    }
}

pub struct XdgShellManager;

impl XdgShellManagerHandler for XdgShellManager {
    #[wlroots_dehandle(compositor)]
    fn new_surface(&mut self,
                   compositor: CompositorHandle,
                   xdg_surface: XdgShellSurfaceHandle)
                   -> (Option<Box<XdgShellHandler>>, Option<Box<SurfaceHandler>>) {
        {
            use compositor as compositor;
            let server: &mut ::Server = compositor.into();
            let ::Server { ref mut views, .. } = *server;
            // TODO We should only push it once it's mapped.
            views.push(Rc::new(::View::new(::Shell::Xdg(xdg_surface.clone().into()))));
        }
        (Some(Box::new(::Xdg::new())), None)
    }
}
