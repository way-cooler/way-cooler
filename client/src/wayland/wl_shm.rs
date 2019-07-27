//! Wrapper around a wl_shm.

use std::{cell::RefCell, fmt, fs::File};

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

pub struct Buffer {
    pub buffer: WlBuffer,
    pub shared_memory: File
}

impl fmt::Debug for Buffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Wayland Buffer with backing file  {:?}",
            self.shared_memory
        )
    }
}

impl PartialEq for Buffer {
    fn eq(&self, other: &Buffer) -> bool {
        self.buffer == other.buffer
    }
}

impl Eq for Buffer {}

pub struct WlShmManager;

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
pub fn create_buffer(size: Size) -> Result<Buffer, ()> {
    let Size { width, height } = size;
    let width = width as i32;
    let height = height as i32;

    let shared_memory =
        tempfile::tempfile().expect("Could not make new temp file");
    shared_memory
        .set_len(size.width as u64 * size.height as u64 * 4)
        .expect("Could not set file length");
    let fd = std::os::unix::io::AsRawFd::as_raw_fd(&shared_memory);

    let buffer = WL_SHM.with(|wl_shm| {
        let wl_shm = wl_shm.borrow();
        let wl_shm = wl_shm.as_ref().expect("WL_SHM was not initilized");
        let pool = wl_shm.create_pool(
            fd,
            width * height * 4,
            NewProxy::implement_dummy
        )?;
        // TODO ARb32 instead
        pool.create_buffer(
            0,
            width,
            height,
            width * 4,
            wl_shm::Format::Argb8888,
            NewProxy::implement_dummy
        )
    })?;

    Ok(Buffer {
        buffer,
        shared_memory
    })
}
