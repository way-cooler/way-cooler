//! This handles the XWayland server and any XWayland clients that connect to
//! Way Cooler.

use std::panic;
use wlroots::{CompositorHandle, XWaylandManagerHandler};

use awesome;

pub struct XWaylandManager;

impl XWaylandManager {
    pub fn new() -> Self {
        XWaylandManager
    }
}

impl XWaylandManagerHandler for XWaylandManager {
    fn on_ready(&mut self, compositor: CompositorHandle) {
        // TODO Do this properly with Results!
        match panic::catch_unwind(panic::AssertUnwindSafe(|| ::awesome::lua::setup_lua(compositor)))
        {
            Ok(_) => {}
            Err(err) => {
                awesome::lua::terminate();
                panic::resume_unwind(err)
            }
        }
    }
}
