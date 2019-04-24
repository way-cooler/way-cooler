//! Wrappers around a xdg_surface and xdg_shell setup code.

use std::{cell::RefCell, fmt, os::unix::io::AsRawFd};

use crate::wayland_protocols::xdg_shell::{
    xdg_surface::{self, XdgSurface},
    xdg_toplevel,
    xdg_wm_base::{self, XdgWmBase}
};
use wayland_client::{
    protocol::{wl_buffer::WlBuffer, wl_surface::WlSurface},
    NewProxy, Proxy
};
use wlroots::{Area, Origin, Size};

use crate::wayland_obj;

/// The minimum version of the xdg_wm_base global to bind to.
pub const XDG_WM_BASE_VERSION: u32 = 2;

thread_local! {
    /// The XDG surface creator.
    ///
    /// This should remain local to just this module.
    static XDG_SHELL_CREATOR: RefCell<Option<XdgWmBase>> = RefCell::new(None);
}

/// A wrapper around `xdg_toplevel::Xdgtoplevel` that keeps track of the changes to the
/// internal state of the `xdg_toplevel`, `xdg_surface, and `wl_surface`.
///
/// This should be used instead of using a xdg_toplevel directly as
/// otherwise you need to roll your own caching scheme.
#[derive(Clone, Eq, PartialEq)]
pub struct XdgToplevel {
    proxy: xdg_toplevel::XdgToplevel
}

/// The cached state for the 'XdgToplevel. Cached state about the `xdg_surface`
/// and the `wl_surface` is also stored in here.
///
/// This needs to be stored as the user data in the `XdgToplevel` so that it
/// can be accessed anywhere.
#[derive(Eq, PartialEq)]
struct XdgToplevelState {
    // TODO These should have wrappers around the proxies as well,
    // so that their cached state is transparent to this cached state.
    wl_surface: WlSurface,
    xdg_surface: XdgSurface,
    buffer: Option<WlBuffer>,
    size: Size
}

struct XdgToplevelHandler {}

impl XdgToplevelHandler {}

impl xdg_toplevel::EventHandler for XdgToplevelHandler {
    #[allow(unused)]
    fn configure(&mut self, object: xdg_toplevel::XdgToplevel, width: i32, height: i32, states: Vec<u8>) {
        // TODO Fill in
        let state = unwrap_state(object.as_ref()).borrow_mut();
        state
            .xdg_surface
            .set_window_geometry(0, 0, state.size.width, state.size.height);
    }

    fn close(&mut self, _object: xdg_toplevel::XdgToplevel) {
        // TODO We should probably do what the compositor wants here.
    }
}

impl XdgToplevel {
    pub fn set_size(&self, size: Size) {
        let Size { width, height, .. } = size;
        // TODO Is there ever a need for us to set the x and y?
        unwrap_state(self.as_ref())
            .borrow()
            .xdg_surface
            .set_window_geometry(0, 0, width, height);
    }

    /// Set the backing storage of the xdg shell surface.
    ///
    /// The contents will not be sent until a wl_surface commit, due to
    /// Wayland surfaces being double buffered.
    pub fn set_surface<FD>(&mut self, fd: &FD, size: Size) -> Result<(), ()>
    where
        FD: AsRawFd
    {
        let buffer = wayland_obj::create_buffer(fd.as_raw_fd(), size)?;
        let mut state = unwrap_state(self.as_ref()).borrow_mut();
        state.wl_surface.attach(Some(&buffer), 0, 0);
        state.size = size;
        state.buffer = Some(buffer);

        Ok(())
    }

    #[allow(dead_code)]
    pub fn commit(&self) {
        unwrap_state(self.as_ref()).borrow().wl_surface.commit();
    }
}

impl Drop for XdgToplevel {
    fn drop(&mut self) {
        self.proxy.destroy();
    }
}

pub fn xdg_shell_init(new_proxy: NewProxy<XdgWmBase>) -> XdgWmBase {
    let xwm_base = new_proxy.implement(XdgWmBaseHandler {}, ());

    XDG_SHELL_CREATOR.with(|shell_creator| {
        *shell_creator.borrow_mut() = Some(xwm_base.clone());
    });

    xwm_base
}

pub struct XdgWmBaseHandler {}

impl xdg_wm_base::EventHandler for XdgWmBaseHandler {
    fn ping(&mut self, object: XdgWmBase, serial: u32) {
        object.pong(serial);
    }
}

pub fn create_xdg_toplevel<G>(geometry: G) -> Result<XdgToplevel, ()>
where
    G: Into<Option<Area>>
{
    let wl_surface = wayland_obj::create_surface()?;
    let xdg_surface = create_xdg_surface(&wl_surface)?;
    if let Some(geometry) = geometry.into() {
        let Origin { x, y } = geometry.origin;
        let Size { width, height } = geometry.size;
        xdg_surface.set_window_geometry(x, y, width, height);
    }

    xdg_surface
        .get_toplevel(|new_toplevel| {
            let state = XdgToplevelState {
                size: Size::default(),
                wl_surface: wl_surface.clone(),
                xdg_surface: xdg_surface.clone(),
                buffer: None
            };

            let res = new_toplevel.implement(XdgToplevelHandler {}, RefCell::new(state));

            wl_surface.commit();

            res
        })
        .map(|toplevel| XdgToplevel { proxy: toplevel })
}

struct XdgSurfaceHandler {
    wlsurface: WlSurface
}

impl xdg_surface::EventHandler for XdgSurfaceHandler {
    fn configure(&mut self, object: XdgSurface, serial: u32) {
        object.ack_configure(serial);
        self.wlsurface.commit();
    }
}

fn create_xdg_surface(surface: &WlSurface) -> Result<XdgSurface, ()> {
    XDG_SHELL_CREATOR.with(|shell_creator| {
        let shell_creator = shell_creator.borrow();
        let shell_creator = shell_creator.as_ref().expect("XDG Shell creator not initilized");
        shell_creator.get_xdg_surface(surface, |new_proxy| {
            new_proxy.implement(
                XdgSurfaceHandler {
                    wlsurface: surface.clone()
                },
                ()
            )
        })
    })
}

impl fmt::Debug for XdgToplevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.proxy.as_ref().c_ptr())
    }
}

impl AsRef<Proxy<xdg_toplevel::XdgToplevel>> for XdgToplevel {
    fn as_ref(&self) -> &Proxy<xdg_toplevel::XdgToplevel> {
        &self.proxy.as_ref()
    }
}

fn unwrap_state(proxy: &Proxy<xdg_toplevel::XdgToplevel>) -> &RefCell<XdgToplevelState> {
    proxy
        .user_data::<RefCell<XdgToplevelState>>()
        .expect("User data has not been set yet")
}
