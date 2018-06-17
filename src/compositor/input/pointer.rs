use wlroots::{pointer_events::*, Capability, CompositorHandle, PointerHandle, PointerHandler,
              WLR_BUTTON_RELEASED};

use compositor::{Seat, Server};

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
                seat.update_cursor_position(cursor,
                                            xcursor_manager,
                                            views,
                                            Some(event.time_msec()));
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
                seat.update_cursor_position(cursor,
                                            xcursor_manager,
                                            views,
                                            Some(event.time_msec()));
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

                if let (Some(view), _, _, _) = Seat::view_at_pointer(views, cursor) {
                    seat.focus_view(view.clone(), views);

                    let meta_held_down = seat.meta;
                    if meta_held_down && event.button() == BTN_LEFT {
                        seat.move_view(cursor, &view, None);
                    }
                    seat.send_button(event);
                } else {
                    seat.clear_focus();
                }
            }).unwrap()
        }).unwrap()
    }

    fn destroyed(&mut self, compositor: CompositorHandle, pointer: PointerHandle) {
        with_handles!([(compositor: {compositor}), (pointer: {pointer})] => {
            let server: &mut Server = compositor.into();
            let weak_reference = pointer.weak_reference();
            if let Some(index) = server.pointers.iter().position(|p| *p == weak_reference) {
                server.pointers.remove(index);
                if server.pointers.len() == 0 {
                    with_handles!([(seat: {&mut server.seat.seat})] => {
                        let mut capabilities = seat.capabilities();
                        capabilities.remove(Capability::Pointer);
                        seat.set_capabilities(capabilities);
                    }).expect("Seat was destroyed")
                }
            }
            // TODO Double check this isn't a safety hole actually,
            // because if it isn't then we may not have to do this here...
            with_handles!([(cursor: {&mut server.cursor})] => {
                cursor.deattach_input_device(pointer.input_device());
            }).expect("Cursor was destroyed");
        }).unwrap();
    }
}
