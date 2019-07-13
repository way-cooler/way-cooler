use std::cell::RefCell;

use wayland_client::NewProxy;

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
use mouse_grabber::ZwayCoolerMousegrabber;

thread_local! {
    static MOUSE_GRABBER: RefCell<Option<ZwayCoolerMousegrabber>> =
        RefCell::new(None)
}

pub const MOUSEGRABBER_VERSION: u32 = 1;

pub struct MousegrabberHandler;

impl mouse_grabber::EventHandler for MousegrabberHandler {
    fn mouse_moved(&mut self, _: ZwayCoolerMousegrabber, x: i32, y: i32) {
        info!("Mouse moved to ({}, {})", x, y);
    }
}

pub fn mousegrabber_init(
    new_proxy: NewProxy<ZwayCoolerMousegrabber>
) -> ZwayCoolerMousegrabber {
    let mouse_grabber = new_proxy.implement(MousegrabberHandler, ());
    mouse_grabber.grab_mouse();
    MOUSE_GRABBER.with(|grabber| {
        *grabber.borrow_mut() = Some(mouse_grabber.clone());
    });
    mouse_grabber
}
