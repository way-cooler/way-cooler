use compositor::{Action, Server, Shell, View};
use wlroots::{CompositorHandle, Origin, SurfaceHandle, XdgV6ShellHandler,
              XdgV6ShellManagerHandler, XdgV6ShellState::*, XdgV6ShellSurfaceHandle};

use wlroots::xdg_shell_v6_events::MoveEvent;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct XdgV6 {
    shell_surface: XdgV6ShellSurfaceHandle
}

impl XdgV6 {
    pub fn new() -> Self {
        XdgV6 { ..XdgV6::default() }
    }
}

impl XdgV6ShellHandler for XdgV6 {
    fn move_request(&mut self,
                    compositor: CompositorHandle,
                    _surface: SurfaceHandle,
                    shell_surface: XdgV6ShellSurfaceHandle,
                    _event: &MoveEvent) {
        with_handles!([(compositor: {compositor})] => {
            let server: &mut Server = compositor.into();
            let ref mut seat = server.seat;
            let ref mut cursor = server.cursor;

            if let Some(ref mut view) = seat.focused {
                let shell: Shell = shell_surface.into();
                let action = &mut seat.action;
                if view.shell == shell {
                    with_handles!([(cursor: {cursor})] => {
                        let (lx, ly) = cursor.coords();
                        let Origin { x: shell_x, y: shell_y } = view.origin;
                        let (view_sx, view_sy) = (lx - shell_x as f64, ly - shell_y as f64);
                        let start = Origin::new(view_sx as _, view_sy as _);
                        *action = Some(Action::Moving { start: start });
                    }).unwrap();
                }
            }
        }).unwrap();
    }
}

pub struct XdgV6ShellManager;

impl XdgV6ShellManagerHandler for XdgV6ShellManager {
    fn new_surface(&mut self,
                   compositor: CompositorHandle,
                   mut shell_surface: XdgV6ShellSurfaceHandle)
                   -> Option<Box<XdgV6ShellHandler>> {
        let is_toplevel = with_handles!([(shell_surface: {&mut shell_surface})] => {
            match shell_surface.state().unwrap() {
                TopLevel(_) => true,
                _ => false
            }
        }).unwrap();

        if is_toplevel {
            with_handles!([(compositor: {compositor})] => {
                let server: &mut Server = compositor.into();
                let Server { ref mut seat,
                             ref mut views,
                             .. } = *server;
                views.insert(0, View::new(Shell::XdgV6(shell_surface.into())));

                with_handles!([(seat: {&mut seat.seat})] => {
                    if let Some(keyboard) = seat.get_keyboard() {
                        with_handles!([(keyboard: {keyboard}),
                                       (surface: {views[0].surface()})] => {
                                           seat.keyboard_notify_enter(surface,
                                                                      &mut keyboard.keycodes(),
                                                                      &mut keyboard.get_modifier_masks());
                        }).unwrap();
                    }
                    views[0].activate(true);
                }).unwrap();
            }).unwrap();
        }

        Some(Box::new(XdgV6::new()))
    }

    fn surface_destroyed(&mut self,
                         compositor: CompositorHandle,
                         shell_surface: XdgV6ShellSurfaceHandle) {
        with_handles!([(compositor: {compositor})] => {
            let server: &mut Server = compositor.into();
            let Server { ref mut seat,
                         ref mut views,
                         .. } = *server;
            let destroyed_shell = shell_surface.into();
            if let Some(pos) = views.iter().position(|view| view.shell == destroyed_shell) {
                views.remove(pos);
            }

            seat.focused = seat.focused.take().and_then(|mut focused| {
                if focused.shell == destroyed_shell {
                    with_handles!([(seat: {&mut seat.seat})] => {
                        if views.len() > 0 {
                            if let Some(keyboard) = seat.get_keyboard() {
                                with_handles!([(keyboard: {keyboard}),
                                    (surface: {focused.surface()})] => {
                                    seat.keyboard_notify_enter(surface,
                                                               &mut keyboard.keycodes(),
                                                               &mut keyboard.get_modifier_masks());
                                }).unwrap();
                            }
                            views[0].activate(true);
                            Some(views[0].clone())
                        } else {
                            seat.keyboard_clear_focus();
                            None
                        }
                    }).unwrap()
                } else {
                    Some(focused)
                }
            });

        }).unwrap();
    }
}
