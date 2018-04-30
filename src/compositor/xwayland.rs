//! This handles the XWayland server and any XWayland clients that connect to
//! Way Cooler.

use std::panic;
use wlroots::{Compositor, XWaylandManagerHandler};

use awesome;
use compositor::Server;

pub struct XWaylandManager;

impl XWaylandManager {
    pub fn new() -> Self {
        XWaylandManager
    }
}

impl XWaylandManagerHandler for XWaylandManager {
    fn on_ready(&mut self, compositor: &mut Compositor) {
        // TODO Do this properly with Results!
        let server: &mut Server = compositor.into();
        match panic::catch_unwind(panic::AssertUnwindSafe(|| ::awesome::lua::setup_lua(server))) {
            Ok(_) => {}
            Err(err) => {
                awesome::lua::terminate();
                panic::resume_unwind(err)
            }
        }
    }
}
