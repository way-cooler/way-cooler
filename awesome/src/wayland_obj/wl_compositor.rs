//! Wrapper around a wl_compositor.

use std::os::unix::io::RawFd;
use std::cell::RefCell;

use wayland_client::{Proxy, NewProxy};
use wayland_client::protocol::wl_compositor::{WlCompositor,
                                              RequestsTrait as WlCompositorTrait};
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::protocol::wl_shm::{self,
                                       WlShm,
                                       RequestsTrait as WlShmTrait};
use wayland_client::protocol::wl_shm_pool::{WlShmPool,
                                            RequestsTrait as WlShmPoolTrait};
use wayland_client::protocol::wl_buffer::WlBuffer;

use wlroots::{Area, Size};

/// The minimum version of the wl_compositor global to bind to.
pub const WL_COMPOSITOR_VERSION: u32 = 3;

/// The minimum version of the wl_shm global to bind to.
pub const WL_SHM_VERSION: u32 = 1;

thread_local! {
    static WL_COMPOSITOR: RefCell<Option<Proxy<WlCompositor>>> =
        RefCell::new(None);
    static WL_SHM: RefCell<Option<Proxy<WlShm>>> = RefCell::new(None);
}

pub fn wl_compositor_init(new_proxy: Result<NewProxy<WlCompositor>, u32>, _: ()) {
    let new_proxy = new_proxy.expect("Could not create wl_compositor");
    let proxy = new_proxy.implement(|_event, _proxy| {});
    WL_COMPOSITOR.with(|wl_compositor| {
        *wl_compositor.borrow_mut() = Some(proxy);
    })
}
pub fn wl_shm_init(new_proxy: Result<NewProxy<WlShm>, u32>, _: ()) {
    let new_proxy = new_proxy.expect("Could not create wl_shm");
    let proxy = new_proxy.implement(|_event, _proxy| {});
    WL_SHM.with(|wl_shm| {
        *wl_shm.borrow_mut() = Some(proxy);
    });
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

pub fn create_buffer(fd: RawFd, size: Size) -> Result<Proxy<WlBuffer>, ()> {
    let Size { width, height } = size;
    WL_SHM.with(|wl_shm| {
        let wl_shm = wl_shm.borrow();
        let wl_shm = wl_shm.as_ref()
            .expect("WL_SHM was not initilized");
        let pool = wl_shm.create_pool(fd, width * height * 4)?.implement(|_,_|{});
        // TODO ARb32 instead
        Ok(pool.create_buffer(0, width, height, (width * 4) as i32, wl_shm::Format::Argb8888)?
            .implement(|_,_| {}))
    })
}
