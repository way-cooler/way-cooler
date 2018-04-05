use compositor::{self, Server};
use wlroots::{self, Compositor, PointerHandler, pointer_events::*};

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
}
