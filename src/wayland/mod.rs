pub mod gamma_control;

use std::ptr;
use wayland_server::Resource;
use wayland_server::sys::WAYLAND_SERVER_HANDLE;
use self::gamma_control::{GammaHandler, GammaManagerHandler};
use rustwlc::wayland;

use std::sync::{Mutex};

lazy_static!(
    static ref GAMMA_CONTROL_MANAGER: Mutex<GammaManagerHandler> =
        Mutex::new(GammaManagerHandler::new());
    static ref GAMMA_CONTROL: Mutex<GammaHandler> =
        Mutex::new(GammaHandler::new());
);

/// Initializes the appropriate handlers for each wayland protocol
/// that Way Cooler supports.
pub fn init_wayland_protocols() {
    error!("Initializing wayland protocols");
    let w_display = wayland::get_display();
    //declare_handler!(GammaHandler,
    //                 gamma_control::Handler,
    //                 gamma_control::GammaControl);
    unsafe {
        // Initalize the gamma control manager
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_global_create,
                      w_display as *mut _,
                      gamma_control::GammaControlManager::interface_ptr(),
                      gamma_control::GammaControlManager::supported_version() as i32,
                      ptr::null_mut(),
                      gamma_control::bind
        );
    }
    error!("Created global!");
    // TODO Doesn't handler get dropped?
}
