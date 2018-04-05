use compositor::{self, Server, Shell, View};
use wlroots::{self, Area, Compositor, HandleResult, Origin, PointerHandler, Size,
              pointer_events::*};

#[derive(Debug, Default)]
pub struct Pointer;

impl PointerHandler for Pointer {
    fn on_motion(&mut self,
                 compositor: &mut Compositor,
                 _: &mut wlroots::Pointer,
                 event: &MotionEvent) {
        let server: &mut Server = compositor.into();
        let cursor = &mut server.cursor;
        run_handles!([(cursor: {cursor})] => {
            let (x, y) = event.delta();
            cursor.move_to(event.device(), x, y);
        }).expect("Cursor was destroyed");
    }

    fn on_button(&mut self,
                 compositor: &mut Compositor,
                 _: &mut wlroots::Pointer,
                 _: &ButtonEvent) {
        let server: &mut Server = compositor.into();
        if let Some(view) = view_at_pointer(server) {
            focus_under_pointer(server, view).expect("Could not focus view");
        }
    }
}

fn view_at_pointer(server: &mut Server) -> Option<View> {
    let Server { ref mut cursor,
                 ref mut views,
                 .. } = *server;
    for view in views {
        match view.shell.clone() {
            Shell::XdgV6(ref mut shell) => {
                let (mut sx, mut sy) = (0.0, 0.0);
                let seen = run_handles!([(shell: {&mut *shell}),
                                         (cursor: {&mut *cursor})] => {
                    let (lx, ly) = cursor.coords();
                    let Origin {x: shell_x, y: shell_y} = shell.geometry().origin;
                    let (view_sx, view_sy) = (lx - shell_x as f64, ly - shell_y as f64);
                    shell.surface_at(view_sx, view_sy, &mut sx, &mut sy).is_some()
                }).ok()?.ok()?;
                if seen {
                    return Some(view.clone())
                }
            }
        }
    }
    None
}

fn focus_under_pointer(server: &mut Server, mut view: View) -> HandleResult<()> {
    let Server { ref mut seat,
                 ref mut keyboards,
                 .. } = *server;
    // TODO Use those surface level coordinates to send events and shit
    for keyboard in { &mut *keyboards } {
        match &mut view.shell {
            &mut Shell::XdgV6(ref mut shell) => {
                run_handles!([(seat: {&mut seat.seat}),
                              (shell: {&mut *shell}),
                              (surface: {shell.surface()}),
                              (keyboard: {keyboard})] => {
                    use wlroots::XdgV6ShellState::*;
                    match shell.state() {
                        Some(&mut TopLevel(ref mut toplevel)) => {
                            // TODO Don't send this for each keyboard!
                            toplevel.set_activated(true);
                        },
                        _ => unimplemented!()
                    }
                    seat.keyboard_notify_enter(surface,
                                               &mut keyboard.keycodes(),
                                               &mut keyboard.get_modifier_masks())
                })????;
            }
        }
    }
    Ok(())
}
