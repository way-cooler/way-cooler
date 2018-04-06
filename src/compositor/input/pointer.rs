use compositor::{self, Action, Server, Shell, View};
use std::time::Duration;
use wlroots::{self, Area, Compositor, CursorHandle, HandleResult, KeyboardHandle, Origin,
              PointerHandler, Size, XdgV6ShellState::*, pointer_events::*, WLR_BUTTON_RELEASED};

#[derive(Debug, Default)]
pub struct Pointer;

impl PointerHandler for Pointer {
    fn on_motion(&mut self,
                 compositor: &mut Compositor,
                 _: &mut wlroots::Pointer,
                 event: &MotionEvent) {
        let server: &mut Server = compositor.into();
        let Server { ref mut cursor,
                     ref mut seat,
                     ref mut views,
                     .. } = *server;
        run_handles!([(cursor: {&mut *cursor})] => {
            let (x, y) = event.delta();
            cursor.move_to(event.device(), x, y);
        }).expect("Cursor was destroyed");
        match seat.action {
            Some(Action::Moving { start }) => {
                if let Some(mut view) = view_at_pointer(views, cursor) {
                    let meta_held_down = seat.meta;
                    if meta_held_down {
                        move_view(seat, cursor, &mut view, start).expect("Could not move view");
                    }
                }
            }
            _ => { /* TODO */ }
        }
    }

    fn on_button(&mut self,
                 compositor: &mut Compositor,
                 _: &mut wlroots::Pointer,
                 event: &ButtonEvent) {
        let server: &mut Server = compositor.into();
        let Server { ref mut cursor,
                     ref mut views,
                     ref mut seat,
                     ref mut keyboards,
                     .. } = *server;
        if event.state() == WLR_BUTTON_RELEASED {
            seat.action = None;
            send_pointer_button(seat, event).expect("Could not send pointer button");
            return
        }
        if let Some(view) = view_at_pointer(views, cursor) {
            focus_under_pointer(seat, &mut **keyboards, { &mut *view }).expect("Could not focus \
                                                                                view");
            let meta_held_down = seat.meta;
            if meta_held_down && event.button() == BTN_LEFT {
                move_view(seat, cursor, view, None).expect("Could not move view");
            }
            send_pointer_button(seat, event).expect("Could not send pointer button");
        } else {
            focus_under_pointer(seat, &mut **keyboards, None).expect("Could not focus view");
        }
    }
}

fn view_at_pointer<'view>(views: &'view mut [View],
                          cursor: &mut CursorHandle)
                          -> Option<&'view mut View> {
    for view in views {
        match view.shell.clone() {
            Shell::XdgV6(ref mut shell) => {
                let (mut sx, mut sy) = (0.0, 0.0);
                let seen = run_handles!([(shell: {&mut *shell}),
                                         (cursor: {&mut *cursor})] => {
                    let (lx, ly) = cursor.coords();
                    let Origin {x: shell_x, y: shell_y} = view.origin;
                    let (view_sx, view_sy) = (lx - shell_x as f64, ly - shell_y as f64);
                    shell.surface_at(view_sx, view_sy, &mut sx, &mut sy).is_some()
                }).ok()?.ok()?;
                if seen {
                    return Some(view)
                }
            }
        }
    }
    None
}

/// Focus the view under the pointer.
fn focus_under_pointer<'view, V>(seat: &mut compositor::Seat,
                                 keyboards: &mut [KeyboardHandle],
                                 view: V)
                                 -> HandleResult<()>
    where V: Into<Option<&'view mut View>>
{
    // TODO Use those surface level coordinates to send events and shit
    match view.into() {
        None => {
            if let Some(mut focused_view) = seat.focused.take() {
                match focused_view.shell {
                    Shell::XdgV6(ref mut shell) => {
                        run_handles!([(shell: {shell})] => {
                            match shell.state() {
                                Some(&mut TopLevel(ref mut toplevel)) => {
                                    toplevel.set_activated(false)
                                },
                                _ => unimplemented!()
                            }
                        })?;
                    }
                }
            }
            run_handles!([(seat: {&mut seat.seat})] => {
                seat.keyboard_clear_focus()
            })
        }
        Some(view) => {
            seat.focused = Some(view.clone());
            for keyboard in { &mut *keyboards } {
                match &mut view.shell {
                    &mut Shell::XdgV6(ref mut shell) => {
                        run_handles!([(seat: {&mut seat.seat}),
                                      (shell: {&mut *shell}),
                                      (surface: {shell.surface()}),
                                      (keyboard: {keyboard})] => {
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
    }
}

/// Start moving a view if you haven't already by passing `start: None`.
///
/// Otherwise, update the view position relative to where the move started,
/// which is provided by Action::Moving.
fn move_view<O>(seat: &mut compositor::Seat,
                cursor: &mut CursorHandle,
                view: &mut View,
                start: O)
                -> HandleResult<()>
    where O: Into<Option<Origin>>
{
    let Origin { x: shell_x,
                 y: shell_y } = view.origin;
    run_handles!([(cursor: {cursor})] => {
        let (lx, ly) = cursor.coords();
        match start.into() {
            None => {
                let (view_sx, view_sy) = (lx - shell_x as f64, ly - shell_y as f64);
                let start = Origin::new(view_sx as _, view_sy as _);
                seat.action = Some(Action::Moving{start});
                return
            }
            Some(start) => {
                let pos = Origin::new(lx as i32 - start.x, ly as i32 - start.y);
                view.origin = pos;
            }
        };
    })?;
    Ok(())
}

fn send_pointer_button(seat: &mut compositor::Seat, event: &ButtonEvent) -> HandleResult<()> {
    run_handles!([(seat: {&mut seat.seat})] => {
        seat.pointer_notify_button(Duration::from_millis(event.time_msec() as _),
                                   event.button(),
                                   event.state() as u32);
    })
}
