use std::time::Duration;

use wlroots::types::Cursor;
use wlroots::{pointer_events::*, CompositorHandle, Origin, PointerHandle, PointerHandler,
              SurfaceHandle, XCursorManager, WLR_BUTTON_RELEASED};

use compositor::{self, Action, Seat, Server, Shell, View};
use std::rc::Rc;

#[derive(Debug, Default)]
pub struct Pointer;

impl PointerHandler for Pointer {
    fn on_motion_absolute(&mut self,
                          compositor: CompositorHandle,
                          _: PointerHandle,
                          event: &AbsoluteMotionEvent) {
        with_handles!([(compositor: {compositor})] => {
            let server: &mut Server = compositor.data.downcast_mut().unwrap();
            let Server { ref mut cursor,
                         ref mut xcursor_manager,
                         ref mut seat,
                         ref mut views,
                         .. } = *server;
            with_handles!([(cursor: {&mut *cursor})] => {
                let (x, y) = event.pos();
                cursor.warp_absolute(event.device(), x, y);
                update_view_position(cursor, xcursor_manager, seat, views, event.time_msec());
            }).expect("Cursor was destroyed");
        }).unwrap();
    }

    fn on_motion(&mut self, compositor: CompositorHandle, _: PointerHandle, event: &MotionEvent) {
        with_handles!([(compositor: {compositor})] => {
            let server: &mut Server = compositor.into();
            let Server { ref mut cursor,
                         ref mut xcursor_manager,
                         ref mut seat,
                         ref mut views,
                         .. } = *server;
            with_handles!([(cursor: {&mut *cursor})] => {
                let (x, y) = event.delta();
                cursor.move_to(event.device(), x, y);
                update_view_position(cursor, xcursor_manager, seat, views, event.time_msec());
            }).expect("Cursor was destroyed");
        }).unwrap();
    }

    fn on_button(&mut self, compositor: CompositorHandle, _: PointerHandle, event: &ButtonEvent) {
        with_handles!([(compositor: {compositor})] => {
            let server: &mut Server = compositor.into();
            let Server { ref mut cursor,
                         ref mut views,
                         ref mut seat,
                         .. } = *server;
            with_handles!([(cursor: {&mut *cursor})] => {
                if event.state() == WLR_BUTTON_RELEASED {
                    seat.action = None;
                    seat.send_button(event);
                    return
                }

                if let (Some(view), _, _, _) = view_at_pointer(views, cursor) {
                    seat.focus_view(view.clone(), views);

                    let meta_held_down = seat.meta;
                    if meta_held_down && event.button() == BTN_LEFT {
                        move_view(seat, cursor, &view, None);
                    }
                    seat.send_button(event);
                } else {
                    seat.clear_focus();
                }
            }).unwrap()
        }).unwrap()
    }
}

/// After the cursor has been warped, send pointer motion events to the view
/// under the pointer or update the position of a view that might have been
/// affected by an ongoing interactive move/resize operation
fn update_view_position(cursor: &mut Cursor,
                        xcursor_manager: &mut XCursorManager,
                        seat: &mut Seat,
                        views: &mut [Rc<View>],
                        time_msec: u32) {
    match seat.action {
        Some(Action::Moving { start }) => {
            seat.focused = seat.focused.take().map(|f| {
                                                       move_view(seat, cursor, &f, start);
                                                       f
                                                   });
        }
        _ => {
            let (_view, surface, sx, sy) = view_at_pointer(views, cursor);
            match surface {
                Some(surface) => {
                    with_handles!([(surface: {surface}), (seat: {&mut seat.seat})] => {
                        seat.pointer_notify_enter(surface, sx, sy);
                        seat.pointer_notify_motion(Duration::from_millis(time_msec as u64), sx, sy);
                    }).unwrap();
                }
                None => {
                    if seat.has_client_cursor {
                        xcursor_manager.set_cursor_image("left_ptr".to_string(), cursor);
                        seat.has_client_cursor = false;
                    }
                    with_handles!([(seat: {&mut seat.seat})] => {
                        seat.pointer_clear_focus();
                    }).unwrap();
                }
            }
        }
    }
}

fn view_at_pointer(views: &mut [Rc<View>],
                   cursor: &mut Cursor)
                   -> (Option<Rc<View>>, Option<SurfaceHandle>, f64, f64) {
    for view in views {
        match view.shell {
            Shell::XdgV6(ref shell) => {
                let (mut sx, mut sy) = (0.0, 0.0);
                let surface = with_handles!([(shell: {shell})] => {
                    let (lx, ly) = cursor.coords();
                    let Origin {x: shell_x, y: shell_y} = view.origin.get();
                    let (view_sx, view_sy) = (lx - shell_x as f64, ly - shell_y as f64);
                    shell.surface_at(view_sx, view_sy, &mut sx, &mut sy)
                }).unwrap();
                if surface.is_some() {
                    return (Some(view.clone()), surface, sx, sy)
                }
            }
        }
    }
    (None, None, 0.0, 0.0)
}

/// Start moving a view if you haven't already by passing `start: None`.
///
/// Otherwise, update the view position relative to where the move started,
/// which is provided by Action::Moving.
fn move_view<O>(seat: &mut compositor::Seat, cursor: &mut Cursor, view: &View, start: O)
    where O: Into<Option<Origin>>
{
    let Origin { x: shell_x,
                 y: shell_y } = view.origin.get();
    let (lx, ly) = cursor.coords();
    match start.into() {
        None => {
            let (view_sx, view_sy) = (lx - shell_x as f64, ly - shell_y as f64);
            let start = Origin::new(view_sx as _, view_sy as _);
            seat.action = Some(Action::Moving { start });
        }
        Some(start) => {
            let pos = Origin::new(lx as i32 - start.x, ly as i32 - start.y);
            view.origin.replace(pos);
        }
    };
}
