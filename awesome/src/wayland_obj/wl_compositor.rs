//! Wrapper around a wl_compositor.

use std::cell::RefCell;

use wayland_client::{Proxy, NewProxy};
use wayland_client::protocol::wl_compositor::{WlCompositor,
                                              RequestsTrait as WlCompositorTrait};
use wayland_client::protocol::wl_surface::WlSurface;

thread_local! {
    static WL_COMPOSITOR: RefCell<Option<Proxy<WlCompositor>>> =
        RefCell::new(None)
}

pub fn wl_compositor_init(new_proxy: Result<NewProxy<WlCompositor>, u32>, _: ()) {
    let new_proxy = new_proxy.expect("Could not create wl_compositor");
    let proxy = new_proxy.implement(|_event, _proxy| {});
    WL_COMPOSITOR.with(|wl_compositor| {
        *wl_compositor.borrow_mut() = Some(proxy);
    })
}

pub fn create_surface() -> Result<Proxy<WlSurface>, ()> {
    WL_COMPOSITOR.with(|wl_compositor| {
        let wl_compositor = wl_compositor.borrow();
        let wl_compositor = wl_compositor.as_ref()
            .expect("WL_COMPOSITOR was not initilized");
        wl_compositor.create_surface()
            .map(|compositor| compositor.implement(|_event, _proxy| {}))
    })
}
