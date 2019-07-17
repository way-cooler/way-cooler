use std::cell::RefCell;

use wayland_client::NewProxy;

use crate::mousegrabber::mousegrabber_handle;

#[allow(
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    dead_code,
    unused_imports,
    unused_variables
)]
mod mouse_grabber {
    use wayland_client::{protocol::wl_surface, *};
    pub(crate) use wayland_commons::{
        map::{Object, ObjectMetadata},
        wire::{Argument, ArgumentType, Message, MessageDesc},
        Interface, MessageGroup
    };
    pub(crate) use wayland_sys as sys;
    use wayland_sys::common::{wl_argument, wl_interface};

    include!(concat!(env!("OUT_DIR"), "/mouse_grabber_api.rs"));

    pub use zway_cooler_mousegrabber::*;
}
use mouse_grabber::{ButtonState, ZwayCoolerMousegrabber};

thread_local! {
    static MOUSE_GRABBER: RefCell<Option<ZwayCoolerMousegrabber>> =
        RefCell::new(None)
}

pub const MOUSEGRABBER_VERSION: u32 = 1;

pub struct MousegrabberHandler;

impl mouse_grabber::EventHandler for MousegrabberHandler {
    fn mouse_moved(&mut self, _: ZwayCoolerMousegrabber, x: i32, y: i32) {
        if let Err(err) = mousegrabber_handle(x, y, None) {
            warn!("mousegrabber returned error {}", err);
        }
    }

    fn mouse_button(
        &mut self,
        _: ZwayCoolerMousegrabber,
        x: i32,
        y: i32,
        pressed: ButtonState,
        button: u32
    ) {
        if let Err(err) =
            mousegrabber_handle(x, y, Some((pressed as u32, button)))
        {
            warn!("mousegrabber returned error {}", err)
        }
    }
}

/// Grabs the mouse. Grabbing the mouse multiple times is a protocol error.
///
/// Only one client at a time can grab the mouse.
pub fn grab_mouse(cursor: String) {
    MOUSE_GRABBER.with(|mouse_grabber| {
        let mouse_grabber = mouse_grabber.borrow();
        let mouse_grabber = mouse_grabber
            .as_ref()
            .expect("Mouse grabber has not been initialized");
        mouse_grabber.grab_mouse(cursor);
    });
}

/// Release the mouse. Releasing the mouse when the client has not grabbed
/// the mouse is a protocol error.
pub fn release_mouse() {
    MOUSE_GRABBER.with(|mouse_grabber| {
        let mouse_grabber = mouse_grabber.borrow();
        let mouse_grabber = mouse_grabber
            .as_ref()
            .expect("Mouse grabber has not been initialized");
        mouse_grabber.release_mouse();
    })
}

pub fn mousegrabber_init(
    new_proxy: NewProxy<ZwayCoolerMousegrabber>
) -> ZwayCoolerMousegrabber {
    let mouse_grabber = new_proxy.implement(MousegrabberHandler, ());
    MOUSE_GRABBER.with(|grabber| {
        *grabber.borrow_mut() = Some(mouse_grabber.clone());
    });
    mouse_grabber
}
