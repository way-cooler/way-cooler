pub mod gamma_control;

use std::ptr;
use wayland_server::Resource;
use wayland_server::sys::WAYLAND_SERVER_HANDLE;
use wayland_sys::server::*;
use self::gamma_control::{GammaHandler, GammaManagerHandler};
use rustwlc::wayland;

use std::os::raw::c_void;
use std::ptr::null_mut;

use std::sync::{Mutex};

// TODO Is this the best way really? Not use anything from the generated file?

#[repr(C)]
pub struct GammaControlManagerInterface {
    destroy: extern "C" fn (wl_client: *mut wl_client,
                            resource: *mut wl_resource),
    get_gamma_control: extern "C" fn (wl_client: *mut wl_client,
                                      resource: *mut wl_resource)
}
extern "C" fn destroy(wl_client: *mut wl_client,
                resource: *mut wl_resource) {
    warn!("GOT HERE");
    warn!("GOT HERE");
    warn!("GOT HERE");

}

extern "C" fn get_gamma_control(wl_client: *mut wl_client,
                                resource: *mut wl_resource) {
    warn!("GOT HERE2");
    warn!("GOT HERE2");
    warn!("GOT HERE2");
}


impl GammaControlManagerInterface {
    pub fn new() -> Self {
        GammaControlManagerInterface {
            destroy: destroy,
            get_gamma_control: get_gamma_control
        }
    }
}

lazy_static!(
    static ref GAMMA_CONTROL_MANAGER: Mutex<GammaControlManagerInterface> =
        Mutex::new(GammaControlManagerInterface::new());
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
