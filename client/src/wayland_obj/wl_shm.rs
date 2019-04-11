//! Wrapper around a wl_shm.

use std::{cell::RefCell, os::unix::io::RawFd};

use wayland_client::{
    self,
    protocol::{
        wl_buffer::WlBuffer,
        wl_shm::{self, WlShm}
    },
    NewProxy
};

use crate::area::Size;

/// The minimum version of the wl_shm global to bind to.
pub const WL_SHM_VERSION: u32 = 1;

thread_local! {
    static WL_SHM: RefCell<Option<WlShm>> = RefCell::new(None);
}

pub struct WlShmManager {}

impl wayland_client::GlobalImplementor<WlShm> for WlShmManager {
    fn new_global(&mut self, new_proxy: NewProxy<WlShm>) -> WlShm {
        let res = new_proxy.implement_dummy();

        WL_SHM.with(|wl_shm| {
            *wl_shm.borrow_mut() = Some(res.clone());
        });

        res
    }
}

/// Create a buffer from the raw file descriptor in the given size.
///
/// This should be called from a shell and generally should not be used
/// directly by the Awesome objects.
pub fn create_buffer(fd: RawFd, size: Size) -> Result<WlBuffer, ()> {
    let Size { width, height } = size;
    let width = width as i32;
    let height = height as i32;
    WL_SHM.with(|wl_shm| {
        let wl_shm = wl_shm.borrow();
        let wl_shm = wl_shm.as_ref().expect("WL_SHM was not initilized");
        let pool = wl_shm.create_pool(fd, width * height * 4, NewProxy::implement_dummy)?;
        // TODO ARb32 instead
        pool.create_buffer(
            0,
            width,
            height,
            width * 4,
            wl_shm::Format::Argb8888,
            NewProxy::implement_dummy
        )
    })
}
