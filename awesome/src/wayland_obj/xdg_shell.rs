//! Wrappers around a xdg_surface and xdg_shell setup code.

use std::cell::RefCell;

use wayland_client::{Proxy, NewProxy};
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_protocols::xdg_shell::{xdg_wm_base::{XdgWmBase,
                                                 RequestsTrait as XdgwmBaseTrait},
                                   xdg_toplevel::XdgToplevel,
                                   xdg_surface::{self,
                                                 XdgSurface,
                                                 RequestsTrait as XdgSurfaceTrait}};
use wlroots::{Area, Origin, Size};

use wayland_obj;

thread_local! {
    /// The XDG surface creator.
    ///
    /// This should remain local to just this module.
    static XDG_SHELL_CREATOR: RefCell<Option<Proxy<XdgWmBase>>> = RefCell::new(None);
}

pub fn xdg_shell_init(new_proxy: Result<NewProxy<XdgWmBase>, u32>, _: ()) {
    let new_proxy = new_proxy.expect("Could not create Xdgsurface");
    let proxy = new_proxy.implement(move |event, proxy: Proxy<XdgWmBase>| {
        use xdg_wm_base::Event;
        match event {
            Event::Ping { serial } => proxy.pong(serial)
        }
    });
    XDG_SHELL_CREATOR.with(|shell_creator| {
        *shell_creator.borrow_mut() = Some(proxy);
    });
}

pub fn create_xdg_toplevel<G>(geometry: G)
                              -> Result<NewProxy<XdgToplevel>, ()>
where G: Into<Option<Area>> {
    let surface = wayland_obj::create_surface()?;
    let xdg_surface = create_xdg_surface(&surface)?;
    if let Some(geometry) = geometry.into() {
        let Origin { x, y } = geometry.origin;
        let Size { width, height } = geometry.size;
        xdg_surface.set_window_geometry(x, y, width, height);
    }
    xdg_surface.get_toplevel()
}

fn create_xdg_surface(surface: &Proxy<WlSurface>)
                      -> Result<Proxy<XdgSurface>, ()> {
    XDG_SHELL_CREATOR.with(|shell_creator| {
        let shell_creator = shell_creator.borrow();
        let shell_creator = shell_creator.as_ref()
            .expect("XDG Shell creator not initilized");
        shell_creator.get_xdg_surface(surface)
            .map(|new_proxy| new_proxy.implement(move |event, proxy: Proxy<XdgSurface>| {
                use self::xdg_surface::Event;
                match event {
                    Event::Configure { serial } => proxy.ack_configure(serial)
                }
            }))
    })
}
