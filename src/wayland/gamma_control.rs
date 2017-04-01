//pub use self::generated::server::gamma_control_manager as gamma_control;
pub use wayland::gamma_control::generated
    ::server::gamma_control::{GammaControl, Handler as GammaControlHandler};
pub use wayland::gamma_control::generated
    ::server::gamma_control_manager::{GammaControlManager, Handler as ManagerHandler};

use wayland_server::{EventLoopHandle, Client};
use wayland_server::protocol::{wl_output};


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

pub struct GammaHandler {}
pub struct GammaManagerHandler {}

impl GammaManagerHandler {
    pub fn new() -> Self {
        GammaManagerHandler {}
    }
}

impl ManagerHandler for GammaManagerHandler {
    fn get_gamma_control(&mut self,
                         evqh: &mut EventLoopHandle,
                         client: &Client,
                         resource: &GammaControlManager,
                         id: super::gamma_control::GammaControl,
                         output: &wl_output::WlOutput) {
        error!("Trying to get gamma control");
        println!("Trying to get gamma control");
        println!("HEEEREEE");
        //panic!()
    }

    fn destroy(&mut self,
               evqh: &mut EventLoopHandle,
               client: &Client,
               resource: &GammaControlManager) {
        error!("Destroyed the manager!");
        //panic!()
    }
}

impl GammaHandler {
    pub fn new() -> Self {
        GammaHandler {}
    }
}

impl GammaControlHandler for GammaHandler {
    fn set_gamma(&mut self,
                 evqh: &mut EventLoopHandle,
                 client: &Client,
                 resource: &GammaControl,
                 red: Vec<u8>,
                 green: Vec<u8>,
                 blue: Vec<u8>) {
        error!("Hey look you set the gamma to {:?} {:?} {:?}", red, green, blue);
        panic!()
    }
}

use wayland_sys::server::*;
use wayland_server::Resource;
use std::os::raw::c_void;
use super::{GAMMA_CONTROL, GAMMA_CONTROL_MANAGER};
use nix::libc::c_int;

/// Binds the handler to a new Wayland resource, created by the client.
/// See https://github.com/vberger/wayland-rs/blob/451ccab330b3d0ec18eaaf72ae17ac35cf432370/wayland-server/src/event_loop.rs#L617
/// to see where I got this particular bit of magic from.
pub unsafe extern "C" fn bind(client: *mut wl_client,
                              data: *mut c_void,
                              version: u32,
                              id: u32) {
    // Initialize the gamma control manager, which manages the gamma control.
    error!("BINDING HERE!");
    println!("BINDING HERE");
    let cur_version = GammaControlManager::supported_version();
    if version > cur_version {
        warn!("Unsupported gamma control protocol version {}!", version);
        warn!("We only support version {}", cur_version);
        return
    }
    error!("Making resource");
    let resource = ffi_dispatch!(
        WAYLAND_SERVER_HANDLE,
        wl_resource_create,
        client,
        GammaControlManager::interface_ptr(),
        version as c_int,
        id
    );
    if resource.is_null() {
        ffi_dispatch!(
            WAYLAND_SERVER_HANDLE,
            wl_client_post_no_memory,
            client
        );
    }
    error!("Resource made");
    println!("Resource made");
    let mut manager = GAMMA_CONTROL_MANAGER.try_lock().unwrap();
    let global_manager_ptr = &mut *manager as *mut _ as *mut c_void;
    error!("Setting impl");
    ffi_dispatch!(
        WAYLAND_SERVER_HANDLE,
        wl_resource_set_implementation,
        resource,
        // This is safe because our lazy static won't move anywhere
        global_manager_ptr,
        ::std::ptr::null_mut(),
        None
    );
    error!("Impl set");
    println!("Impl set");
}
