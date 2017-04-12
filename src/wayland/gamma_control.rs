//! Module that defines the bindings for the patched redshift protocol.
//! See https://github.com/giucam/redshift/blob/master/src/gamma-control.xml
//! and https://github.com/jonls/redshift/issues/55
//! for more information about the spec and its status in regards to upstream.
use wayland::gamma_control::generated
    ::server::gamma_control::GammaControl;
use wayland::gamma_control::generated
    ::server::gamma_control_manager::GammaControlManager;
use rustwlc::wayland;
use rustwlc::handle::{wlc_handle_from_wl_output_resource, WlcOutput};
use rustwlc::render::{wlc_output_set_gamma, wlc_output_get_gamma_size};
use wayland_server::Resource;
use wayland_sys::common::{wl_array};
use wayland_sys::server::{WAYLAND_SERVER_HANDLE, wl_client, wl_resource};
use std::os::raw::c_void;
use nix::libc::{c_int, c_uint, uint32_t, uint16_t};

static SET_GAMMA_ERROR: &'static str = "The gamma ramps don't have the same size!";
static INVALID_GAMMA_CODE: u32 = 0;

static mut GAMMA_CONTROL_MANAGER: GammaControlManagerInterface =
    GammaControlManagerInterface {
        destroy: destroy,
        get_gamma_control: get_gamma_control
    };

static mut GAMMA_CONTROL: GammaControlInterface =
    GammaControlInterface {
        destroy: destroy,
        set_gamma: set_gamma,
        reset_gamma: reset_gamma
    };

/// Generated modules from the XML protocol spec.
mod generated {
    // Generated code generally doesn't follow standards
    #![allow(dead_code,non_camel_case_types,unused_unsafe,unused_variables)]
    #![allow(non_upper_case_globals,non_snake_case,unused_imports)]

    pub mod interfaces {
        #[doc(hidden)]
        pub use wayland_server::protocol_interfaces::{wl_output_interface};
        include!(concat!(env!("OUT_DIR"), "/gamma-control_interface.rs"));
    }

    pub mod server {
        #[doc(hidden)]
        pub use wayland_server::{Resource, Handler,
                                 Client, Liveness,
                                 EventLoopHandle, EventResult};
        #[doc(hidden)]
        pub use wayland_server::protocol::{wl_output};
        #[doc(hidden)]
        pub use super::interfaces;
        include!(concat!(env!("OUT_DIR"), "/gamma-control_api.rs"));
    }
}

#[repr(C)]
/// Controls access to the gamma control interface.
/// Right now we let anyone through that wants to access it, but this
/// will allow us to limit who access to it in the future.
struct GammaControlManagerInterface {
    destroy: unsafe extern "C" fn (client: *mut wl_client,
                                   resource: *mut wl_resource),
    get_gamma_control: unsafe extern "C" fn (client: *mut wl_client,
                                             resource: *mut wl_resource,
                                             id: u32,
                                             output: *mut wl_resource)
}

#[repr(C)]
/// The interface that allows a client to control the gamma ramps.
struct GammaControlInterface {
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

/// Sets the gamma to the provided values.
/// If the sizes of the arrays are not all the same,
/// nothing is done and we post an error to the client
/// through the Wayland IPC.
unsafe extern "C" fn set_gamma(_client: *mut wl_client,
                               resource: *mut wl_resource,
                               red: *mut wl_array,
                               green: *mut wl_array,
                               blue: *mut wl_array) {
    debug!("Setting gamma");
    if (*red).size != (*green).size || (*red).size != (*blue).size {
        ffi_dispatch!(
            WAYLAND_SERVER_HANDLE,
            wl_resource_post_error,
            resource,
            INVALID_GAMMA_CODE,
            SET_GAMMA_ERROR.as_bytes().as_ptr() as *const i8);
        warn!("Color size error, can't continue");
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
    if output.is_null() {
        warn!("wl_resource didn't correspond to a wlc output");
        return;
    }
    wlc_output_set_gamma(output.0, ((*red).size / 2) as u16, r, g, b)

}

/// Resets the gamma to the original value set by the hardware.
/// This action is performed properly by `set_gamma`, so method only
/// logs that the action occured.
unsafe extern "C" fn reset_gamma(_client: *mut wl_client,
                                 _resource: *mut wl_resource) {
    debug!("Resetting gamma");
}

/// Destroys the wl_resource.
unsafe extern "C" fn destroy(_client: *mut wl_client,
                             resource: *mut wl_resource) {
    ffi_dispatch!(
        WAYLAND_SERVER_HANDLE,
        wl_resource_destroy,
        resource
    );
}

/// The method that is called when a client attempts to access the gamma
/// control interface. If successful, this method grants access by
/// setting the implementation of the passed in wl_resource to the gamma
/// control singleton and responding to the client with the size of the gamma
/// ramps for the specified output.
unsafe extern "C" fn get_gamma_control(client: *mut wl_client,
                                       _resource: *mut wl_resource,
                                       id: uint32_t,
                                       output: *mut wl_resource) {
    debug!("Request received for control of the gamma ramps");
    let manager_resource = ffi_dispatch!(
        WAYLAND_SERVER_HANDLE,
        wl_resource_create,
        client,
        GammaControl::interface_ptr(),
        GammaControl::supported_version() as i32,
        id);
    let wlc_output = WlcOutput(wlc_handle_from_wl_output_resource(output as *const _));
    if wlc_output.is_null() {
        warn!("This is triggering, dis bad?");
        return;
    }
    debug!("Client requested control of the gamma ramps for {:?}", wlc_output);
    let gamma_control_ptr = &mut GAMMA_CONTROL as *mut _ as *mut c_void;
    ffi_dispatch!(
        WAYLAND_SERVER_HANDLE,
        wl_resource_set_implementation,
        manager_resource,
        gamma_control_ptr,
        output as *mut c_void,
        None
    );
    debug!("Request granted for gamma ramp control of {:?}", wlc_output);
    gamma_control_send_gamma_size(manager_resource,
                                  wlc_output_get_gamma_size(wlc_output.0))
}

/// Binds the handler to a new Wayland resource, created by the client.
/// See https://github.com/vberger/wayland-rs/blob/451ccab330b3d0ec18eaaf72ae17ac35cf432370/wayland-server/src/event_loop.rs#L617
/// to see where I got this particular bit of magic from.
unsafe extern "C" fn bind(client: *mut wl_client,
                              _data: *mut c_void,
                              version: u32,
                              id: u32) {
    debug!("Binding Gamma Control resource");
    let cur_version = GammaControlManager::supported_version();
    if version > cur_version {
        warn!("Unsupported gamma control protocol version {}!", version);
        warn!("We only support version {}", cur_version);
        return
    }
    let resource = ffi_dispatch!(
        WAYLAND_SERVER_HANDLE,
        wl_resource_create,
        client,
        GammaControlManager::interface_ptr(),
        version as c_int,
        id
    );
    if resource.is_null() {
        warn!("Out of memory, could not make a new wl_resource \
               for gamma control");
        ffi_dispatch!(
            WAYLAND_SERVER_HANDLE,
            wl_client_post_no_memory,
            client
        );
    }
    let global_manager_ptr = &mut GAMMA_CONTROL_MANAGER as *mut _ as *mut c_void;
    //let global_manager_ptr = &mut *manager as *mut _ as *mut c_void;
    ffi_dispatch!(
        WAYLAND_SERVER_HANDLE,
        wl_resource_set_implementation,
        resource,
        global_manager_ptr,
        ::std::ptr::null_mut(),
        None
    );
}

/// Send the size of the gamma ramps of the output specified by the wl_resource
/// to the client.
unsafe extern "C" fn gamma_control_send_gamma_size(resource: *mut wl_resource,
                                                   size: uint16_t) {
    ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                  wl_resource_post_event,
                  resource,
                  0,
                  size as c_uint);
}

/// Sets up Way Cooler to announce it uses the gamma control interface.
pub fn init() {
    let w_display = wayland::get_display();
    unsafe {
        debug!("Initializing gamma control manager");
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_global_create,
                      w_display as *mut _,
                      GammaControlManager::interface_ptr(),
                      GammaControlManager::supported_version() as i32,
                      ::std::ptr::null_mut(),
                      bind
        );
    }
}

