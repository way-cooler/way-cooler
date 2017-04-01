pub mod gamma_control;

use std::ptr;
use wayland_server::Resource;
use wayland_server::sys::WAYLAND_SERVER_HANDLE;
use wayland_sys::server::*;
use wayland_sys::common::*;
use self::gamma_control::{GammaControl, GammaHandler, GammaManagerHandler};
use rustwlc::handle::{wlc_handle_from_wl_output_resource, WlcOutput, WlcView};
use rustwlc::wayland;

use std::os::raw::c_void;
use std::ptr::null_mut;

use std::sync::{Mutex};

// TODO Is this the best way really? Not use anything from the generated file?
// TODO Move this to gamma_control.rs
unsafe extern "C" fn gamma_control_send_gamma_size(resource: *mut wl_resource,
                                                   size: u32) {
    ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                  wl_resource_post_event,
                  resource,
                  0,
                  size);
}

#[repr(C)]
pub struct GammaControlManagerInterface {
    destroy: unsafe extern "C" fn (client: *mut wl_client,
                                   resource: *mut wl_resource),
    get_gamma_control: unsafe extern "C" fn (client: *mut wl_client,
                                             resource: *mut wl_resource,
                                             id: u32,
                                             output: *mut wl_resource)
}

#[repr(C)]
pub struct GammaControlInterface {
    destroy: unsafe extern "C" fn (client: *mut wl_client,
                                   resource: *mut wl_resource),
    set_gamma: unsafe extern "C" fn (client: *mut wl_client,
                                     resource: *mut wl_resource,
                                     red: *mut wl_array,
                                     green: *mut wl_array,
                                     blue: *mut wl_array),
    reset_gamma: unsafe extern "C" fn (client: *mut wl_client,
                                       resource: *mut wl_resource)
}

unsafe extern "C" fn set_gamma(client: *mut wl_client,
                               resource: *mut wl_resource,
                               red: *mut wl_array,
                               green: *mut wl_array,
                               blue: *mut wl_array) {
    if (*red).size != (*green).size || (*red).size != (*blue).size {
        // TODO wl_resource_post_error
        return
    }
    let r = (*red).data as *mut u16;
    let g = (*green).data as *mut u16;
    let b = (*blue).data as *mut u16;
    let user_data = ffi_dispatch!(
        WAYLAND_SERVER_HANDLE,
        wl_resource_get_user_data,
        resource) as *const _;
    let output = WlcOutput(wlc_handle_from_wl_output_resource(user_data));
    // TODO Make this less stupid to check if it's a null index
    if output.as_view().is_root() {
        return;
    }
    debug!("Setting gamma for output {:?}", output);
    warn!("Setting gamma for output {:?}", output);
    error!("Setting gamma for output {:?}", output);
    // TODO Uncomment when implemented in wlc
    //wlc_output_set_gamma(output, (*red).size / 16, r, g b)

}
unsafe extern "C" fn reset_gamma(client: *mut wl_client,
                                 resource: *mut wl_resource) {
    // Do nothing
}

unsafe extern "C" fn destroy(wl_client: *mut wl_client,
                resource: *mut wl_resource) {
    // TODO wl_destroy_resource
    unimplemented!()
}

unsafe extern "C" fn get_gamma_control(client: *mut wl_client,
                                       resource: *mut wl_resource,
                                       id: u32,
                                       output: *mut wl_resource) {
    let manager_resource = ffi_dispatch!(
        WAYLAND_SERVER_HANDLE,
        wl_resource_create,
        client,
        GammaControl::interface_ptr(),
        GammaControl::supported_version() as i32,
        id);
    // check that the output is something we control from wlc
    warn!("In the setup or whatever");
    {
        let output = WlcOutput(wlc_handle_from_wl_output_resource(resource as *const _));
        // TODO Make this less stupid to check if it's a null index
        if output.as_view().is_root() {
            warn!("This is triggering, dis bad?");
            // TODO ORRRR maybe note whatever
            //return;
        }
    }
    let mut gamma_control = GAMMA_CONTROL.try_lock().unwrap();
    let gamma_control_ptr = &mut *gamma_control as *mut _ as *mut c_void;
    ffi_dispatch!(
        WAYLAND_SERVER_HANDLE,
        wl_resource_set_implementation,
        manager_resource,
        gamma_control_ptr,
        output as *mut c_void,
        None
    );
    // TODO Uncomment when I add that function to wlc
    //gamma_control_send_gamma_size(manager_resource, wlc_output_get_gamma_size(output))
}


impl GammaControlManagerInterface {
    pub fn new() -> Self {
        GammaControlManagerInterface {
            destroy: destroy,
            get_gamma_control: get_gamma_control
        }
    }
}

impl GammaControlInterface {
    pub fn new() -> Self {
        GammaControlInterface {
            destroy: destroy,
            set_gamma: set_gamma,
            reset_gamma: reset_gamma
        }
    }
}

lazy_static!(
    static ref GAMMA_CONTROL_MANAGER: Mutex<GammaControlManagerInterface> =
        Mutex::new(GammaControlManagerInterface::new());
    static ref GAMMA_CONTROL: Mutex<GammaControlInterface> =
        Mutex::new(GammaControlInterface::new());
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
