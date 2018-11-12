//! Wrappers around a xdg_surface and xdg_shell setup code.

use std::os::unix::io::AsRawFd;
use std::cell::RefCell;
use std::fmt;

use wayland_client::{Proxy, NewProxy};
use wayland_client::protocol::wl_surface::{WlSurface,
                                           RequestsTrait as WlSurfaceTrait};
use wayland_protocols::xdg_shell::{xdg_wm_base::{XdgWmBase,
                                                 RequestsTrait as XdgWmBaseTrait},
                                   xdg_toplevel::{self,
                                                  RequestsTrait as XdgToplevelTrait},
                                   xdg_surface::{self,
                                                 XdgSurface,
                                                 RequestsTrait as XdgSurfaceTrait}};
use wayland_client::protocol::wl_buffer::WlBuffer;
use wlroots::{Area, Origin, Size};

use wayland_obj;

/// The minimum version of the xdg_wm_base global to bind to.
pub const XDG_WM_BASE_VERSION: u32 = 2;

thread_local! {
    /// The XDG surface creator.
    ///
    /// This should remain local to just this module.
    static XDG_SHELL_CREATOR: RefCell<Option<Proxy<XdgWmBase>>> = RefCell::new(None);
}

/// A wrapper around `xdg_toplevel::Xdgtoplevel` that keeps track of the changes to the
/// internal state of the `xdg_toplevel`, `xdg_surface, and `wl_surface`.
///
/// This should be used instead of using a xdg_toplevel directly as
/// otherwise you need to roll your own caching scheme.
#[derive(Clone, Eq, PartialEq)]
pub struct XdgToplevel {
    proxy: Proxy<xdg_toplevel::XdgToplevel>
}

/// The cached state for the 'XdgToplevel. Cached state about the `xdg_surface`
/// and the `wl_surface` is also stored in here.
///
/// This needs to be stored as the user data in the `XdgToplevel` so that it
/// can be accessed anywhere.
#[derive(Clone, Eq, PartialEq)]
struct XdgToplevelState {
    // TODO These should have wrappers around the proxies as well,
    // so that their cached state is transparent to this cached state.
    wl_surface: Proxy<WlSurface>,
    xdg_surface: Proxy<XdgSurface>,
    buffer: Option<Proxy<WlBuffer>>,
    size: Size
}

impl XdgToplevel {
    fn new(wl_surface: Proxy<WlSurface>,
           xdg_surface: Proxy<XdgSurface>,
           xdg_proxy: NewProxy<xdg_toplevel::XdgToplevel>) -> Self {
        let proxy = xdg_proxy.implement(|event, proxy: Proxy<xdg_toplevel::XdgToplevel>| {
            use self::xdg_toplevel::Event;
            match event {
                Event::Configure { .. } => {
                    // TODO Fill in
                    let state = unwrap_state(&proxy);
                    state.xdg_surface.set_window_geometry(0, 0, state.size.width, state.size.height);
                },
                Event::Close => {
                    // TODO We should probably do what the compositor wants here.
                }
            }
        });
        wl_surface.commit();
        let cached_state = Box::new(XdgToplevelState { wl_surface,
                                                       xdg_surface,
                                                       buffer: None,
                                                       size: Size::default() });
        proxy.set_user_data(Box::into_raw(cached_state) as _);
        XdgToplevel { proxy }
    }

    pub fn set_size(&self, size: Size) {
        let Size { width, height } = size;
        // TODO Is there ever a need for us to set the x and y?
        unwrap_state(&self.proxy).xdg_surface.set_window_geometry(0, 0, width, height);
    }

    /// Set the backing storage of the xdg shell surface.
    ///
    /// The contents will not be sent until a wl_surface commit, due to
    /// Wayland surfaces being double buffered.
    pub fn set_surface<FD>(&mut self, fd: &FD, size: Size) -> Result<(), ()>
    where FD: AsRawFd {
        let buffer = wayland_obj::create_buffer(fd.as_raw_fd(), size)?;
        let state = unwrap_state_mut(&mut self.proxy);
        state.wl_surface.attach(Some(&buffer), 0, 0);
        state.size = size;
        state.buffer = Some(buffer);
        Ok(())
    }

    #[allow(dead_code)]
    pub fn commit(&self) {
        unwrap_state(&self.proxy).wl_surface.commit();
    }
}

impl Drop for XdgToplevel {
    fn drop(&mut self) {
        self.proxy.destroy();
        unsafe {
            let user_data = self.proxy.get_user_data() as *mut XdgToplevelState;
            if !user_data.is_null() {
                Box::from_raw(user_data);
            }
        }
    }
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
                              -> Result<XdgToplevel, ()>
where G: Into<Option<Area>> {
    let surface = wayland_obj::create_surface()?;
    let xdg_surface = create_xdg_surface(&surface)?;
    if let Some(geometry) = geometry.into() {
        let Origin { x, y } = geometry.origin;
        let Size { width, height } = geometry.size;
        xdg_surface.set_window_geometry(x, y, width, height);
    }
    xdg_surface.get_toplevel()
        .map(|toplevel| XdgToplevel::new(surface, xdg_surface, toplevel))
}

fn create_xdg_surface(surface: &Proxy<WlSurface>)
                      -> Result<Proxy<XdgSurface>, ()> {
    XDG_SHELL_CREATOR.with(|shell_creator| {
        let shell_creator = shell_creator.borrow();
        let shell_creator = shell_creator.as_ref()
            .expect("XDG Shell creator not initilized");
        let surface_ = surface.clone();
        shell_creator.get_xdg_surface(surface)
            .map(|new_proxy| new_proxy.implement(move |event, proxy: Proxy<XdgSurface>| {
                use self::xdg_surface::Event;
                match event {
                    Event::Configure { serial } => {
                        proxy.ack_configure(serial);
                        surface_.commit();
                    }
                }
            }))
    })
}

impl fmt::Debug for XdgToplevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.proxy.c_ptr())
    }
}

fn unwrap_state_mut<'this>(proxy: &'this mut Proxy<xdg_toplevel::XdgToplevel>)
                           -> &'this mut XdgToplevelState {
    unsafe {
        let user_data = proxy.get_user_data() as *mut XdgToplevelState;
        if user_data.is_null() {
            panic!("User data has not been set yet");
        }
        &mut *user_data
    }
}

fn unwrap_state<'this>(proxy: &'this Proxy<xdg_toplevel::XdgToplevel>)
                           -> &'this XdgToplevelState {
    unsafe {
        let user_data = proxy.get_user_data() as *const XdgToplevelState;
        if user_data.is_null() {
            panic!("User data has not been set yet");
        }
        &*user_data
    }
}
