pub mod gamma_control;

use wayland_server::Resource;
use wayland_server::sys::WAYLAND_SERVER_HANDLE;
use rustwlc::wayland;
use std::ptr;

/// Initializes the appropriate handlers for each wayland protocol
/// that Way Cooler supports.
pub fn init_wayland_protocols() {
    debug!("Initializing wayland protocols");
    let w_display = wayland::get_display();
    unsafe {
        debug!("Initializing gamma control manager");
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_global_create,
                      w_display as *mut _,
                      gamma_control::GammaControlManager::interface_ptr(),
                      gamma_control::GammaControlManager::supported_version() as i32,
                      ptr::null_mut(),
                      gamma_control::bind
        );
    }
}
