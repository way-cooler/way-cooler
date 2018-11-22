use wlroots::{pointer_events::*, wlroots_dehandle::wlroots_dehandle, Capability, CompositorHandle,
              PointerHandle, PointerHandler, WLR_BUTTON_RELEASED};

#[derive(Debug, Default)]
pub struct Pointer;

impl PointerHandler for Pointer {
    #[wlroots_dehandle(compositor, cursor)]
    fn on_motion_absolute(&mut self,
                          compositor_handle: CompositorHandle,
                          _: PointerHandle,
                          event: &AbsoluteMotionEvent)
    {
        use compositor_handle as compositor;
        let server: &mut ::Server = compositor.data.downcast_mut().unwrap();
        let ::Server { ref cursor_handle,
                       ref mut xcursor_manager,
                       ref mut seat,
                       ref mut views,
                       .. } = *server;
        use cursor_handle as cursor;
        let (x, y) = event.pos();
        cursor.warp_absolute(event.device(), x, y);
        seat.update_cursor_position(cursor, xcursor_manager, views, Some(event.time_msec()))
    }

    #[wlroots_dehandle(compositor, cursor)]
    fn on_motion(&mut self,
                 compositor_handle: CompositorHandle,
                 _: PointerHandle,
                 event: &MotionEvent)
    {
        use compositor_handle as compositor;
        let server: &mut ::Server = compositor.into();
        let ::Server { ref cursor_handle,
                       ref mut xcursor_manager,
                       ref mut seat,
                       ref mut views,
                       .. } = *server;
        use cursor_handle as cursor;
        let (x, y) = event.delta();
        cursor.move_to(event.device(), x, y);
        seat.update_cursor_position(cursor, xcursor_manager, views, Some(event.time_msec()));
    }

    #[wlroots_dehandle(compositor, cursor)]
    fn on_button(&mut self,
                 compositor_handle: CompositorHandle,
                 _: PointerHandle,
                 event: &ButtonEvent)
    {
        use compositor_handle as compositor;
        let server: &mut ::Server = compositor.into();
        let ::Server { ref cursor_handle, ref mut views, ref mut seat, .. } = *server;
        use cursor_handle as cursor;
        if event.state() == WLR_BUTTON_RELEASED {
            seat.action = None;
            seat.send_button(event);
            return
        }

        if let (Some(view), ..) = ::Seat::view_at_pointer(views, cursor) {
            seat.focus_view(view.clone(), views);

            let meta_held_down = seat.meta;
            if meta_held_down && event.button() == BTN_LEFT {
                seat.move_view(cursor, &view, None);
            }
            seat.send_button(event);
        } else {
            seat.clear_focus();
        }
    }

    #[wlroots_dehandle(compositor, pointer, seat, cursor)]
    fn destroyed(&mut self, compositor_handle: CompositorHandle, pointer_handle: PointerHandle) {
        use compositor_handle as compositor;
        use pointer_handle as pointer;
        let server: &mut ::Server = compositor.into();
        let weak_reference = pointer.weak_reference();
        server.pointers.retain(|p| *p != weak_reference);
        if server.pointers.is_empty() {
            let seat_handle = &server.seat.seat;
            use seat_handle as seat;
            let mut capabilities = seat.capabilities();
            capabilities.remove(Capability::Pointer);
            seat.set_capabilities(capabilities);
        }
        // TODO Double check this isn't a safety hole actually,
        // because if it isn't then we may not have to do this here...
        let cursor_handle = &server.cursor_handle;
        use cursor_handle as cursor;
        cursor.deattach_input_device(pointer.input_device());
    }
}
