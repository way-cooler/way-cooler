//! This handles the XWayland server and any XWayland clients that connect to
//! Way Cooler.

use std::panic;
use wlroots::{CompositorHandle, SurfaceHandler, XWaylandManagerHandler, XWaylandSurfaceHandle,
              XWaylandSurfaceHandler};

pub struct XWaylandManager;

impl XWaylandManager {
    pub fn new() -> Self {
        XWaylandManager
    }
}

impl XWaylandManagerHandler for XWaylandManager {
    fn on_ready(&mut self, compositor: CompositorHandle) {}

    // TODO
    fn new_surface(&mut self,
                   _: CompositorHandle,
                   _: XWaylandSurfaceHandle)
                   -> (Option<Box<XWaylandSurfaceHandler>>, Option<Box<SurfaceHandler>>) {
        (None, None)
    }
}
