//! This handles the XWayland server and any XWayland clients that connect to
//! Way Cooler.

use wlroots::{Compositor, XWaylandManagerHandler};
pub struct XWaylandManager;

impl XWaylandManager {
    pub fn new() -> Self {
        XWaylandManager
    }
}

impl XWaylandManagerHandler for XWaylandManager {
    fn on_ready(&mut self, _: &mut Compositor) {
        ::awesome::lua::setup_lua();
    }
}
