//! Module that defines the bindings for the desktop shell protocol
//! See https://github.com/swaywm/sway/blob/master/protocols/desktop-shell.xml
//! for more information about the spec

use self::generated::server::desktop_shell::DesktopShell;
use rustwlc::wayland::{get_display};
use rustwlc::handle::{wlc_handle_from_wl_output_resource, WlcOutput};
use ::layout::{lock_tree, IncompleteBackground};
use wayland_server::Resource;
use wayland_sys::server::{WAYLAND_SERVER_HANDLE, wl_client, wl_resource};
use std::os::raw::c_void;
use nix::libc::{c_int};

#[repr(C)]
/// Holds pointers to the functions that should be called when a Wayland request
/// is sent to Way Cooler that falls under the desktop shell's responsibility
struct DesktopShellImpl {
    set_background: unsafe extern "C" fn (*mut wl_client,
                                          *mut wl_resource,
                                          *mut wl_resource,
                                          *mut wl_resource),
    set_panel: unsafe extern "C" fn (*mut wl_client,
                                     *mut wl_resource,
                                     *mut wl_resource,
                                     *mut wl_resource),
    set_lock_surface: unsafe extern "C" fn (*mut wl_client,
                                            *mut wl_resource,
                                            *mut wl_resource,
                                            *mut wl_resource),
    unlock: unsafe extern "C" fn (*mut wl_client,
                                  *mut wl_resource),
    set_grab_surface: unsafe extern "C" fn (*mut wl_client,
                                            *mut wl_resource,
                                            *mut wl_resource),
    desktop_ready: unsafe extern "C" fn (*mut wl_client,
                                         *mut wl_resource),
    set_panel_position: unsafe extern "C" fn (*mut wl_client,
                                              *mut wl_resource,
                                              u32)
}

static mut DESKTOP_SHELL_GLOBAL: DesktopShellImpl =
    DesktopShellImpl {
        set_background,
        set_panel,
        set_lock_surface,
        unlock,
        set_grab_surface,
        desktop_ready,
        set_panel_position
    };

mod generated {
    // Generated code generally doesn't follow standards
    #![allow(dead_code,non_camel_case_types,unused_unsafe,unused_variables)]
    #![allow(non_upper_case_globals,non_snake_case,unused_imports)]

    pub mod interfaces {
        #[doc(hidden)]
        use wayland_server::protocol_interfaces::{wl_output_interface, wl_surface_interface};
        include!(concat!(env!("OUT_DIR"), "/desktop-shell_interface.rs"));
    }

    pub mod server {
        #[doc(hidden)]
        use wayland_server::{Resource, Handler,
                                 Client, Liveness,
                                 EventLoopHandle, EventResult};
        #[doc(hidden)]
        use wayland_server::protocol::{wl_output, wl_surface};
        #[doc(hidden)]
        use super::interfaces;
        include!(concat!(env!("OUT_DIR"), "/desktop-shell_api.rs"));
    }
}

/// Binds the handler to a new Wayland resource, created by the client.
/// See https://github.com/vberger/wayland-rs/blob/451ccab330b3d0ec18eaaf72ae17ac35cf432370/wayland-server/src/event_loop.rs#L617
/// to see where I got this particular bit of magic from.
unsafe extern "C" fn bind(client: *mut wl_client,
                          _data: *mut c_void,
                          version: u32,
                          id: u32) {
    info!("Binding Desktop Shell resource");
    let cur_version = DesktopShell::supported_version();
    if version > cur_version {
        warn!("Unsupported desktop shell control protocol version {}!", version);
        warn!("We only support version {}", cur_version);
        return
    }
    let resource = ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                                 wl_resource_create,
                                 client,
                                 DesktopShell::interface_ptr(),
                                 version as c_int,
                                 id);
    if resource.is_null() {
        warn!("Out of memory, could not make a new wl_resource \
               for desktop shell");
        ffi_dispatch!(
            WAYLAND_SERVER_HANDLE,
            wl_client_post_no_memory,
            client
        );
    }
    let global_ptr = &mut DESKTOP_SHELL_GLOBAL as *mut _ as *mut c_void;
    ffi_dispatch!(
        WAYLAND_SERVER_HANDLE,
        wl_resource_set_implementation,
        resource,
        global_ptr,
        ::std::ptr::null_mut(),
        None
    );
}

unsafe extern "C" fn set_background(client: *mut wl_client,
                                    _resource: *mut wl_resource,
                                    output_: *mut wl_resource,
                                    surface: *mut wl_resource) {
    let output = wlc_handle_from_wl_output_resource(output_ as *const _);
    if output == 0 {
        return;
    }
    info!("Setting surface {:?} as background for output {}", surface, output);
    if let Ok(mut tree) = lock_tree() {
        tree.add_incomplete_background(IncompleteBackground::new(client), WlcOutput::dummy(output as _))
            .expect("Could not add incomplete background");
    }
}

unsafe extern "C" fn desktop_ready(_client: *mut wl_client,
                                   _resource: *mut wl_resource) {
    /* Intentionally left blank */
}

// TODO Implement
unsafe extern "C" fn set_lock_surface(_client: *mut wl_client,
                                      _resource: *mut wl_resource,
                                      _output: *mut wl_resource,
                                      _surface: *mut wl_resource) {}
unsafe extern "C" fn unlock(_client: *mut wl_client,
                            _resource: *mut wl_resource) {}
unsafe extern "C" fn set_grab_surface(_client: *mut wl_client,
                                      _resource: *mut wl_resource,
                                      _surface: *mut wl_resource) {}
unsafe extern "C" fn set_panel(_client: *mut wl_client,
                               _resource: *mut wl_resource,
                               _output_: *mut wl_resource,
                               _surface: *mut wl_resource) {}
unsafe extern "C" fn set_panel_position(_client: *mut wl_client,
                                        _resource:*mut wl_resource,
                                        _position: u32) {}

/// Sets up Way Cooler to announce the desktop-shell interface.
pub fn init() {
    let w_display = get_display();
    unsafe {
        info!("Initializing desktop shell");
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                    wl_global_create,
                    w_display as *mut _,
                    DesktopShell::interface_ptr(),
                    DesktopShell::supported_version() as i32,
                    ::std::ptr::null_mut(),
                    bind);
    }
}
