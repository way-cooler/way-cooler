use std::{
    cell::RefCell,
    fmt,
    io::{self, Write as _}
};

pub use {
    wayland_client::{
        protocol::{
            wl_buffer::WlBuffer, wl_output::WlOutput, wl_surface::WlSurface
        },
        NewProxy, Proxy
    },
    wayland_protocols::wlr::unstable::layer_shell::v1::client::{
        zwlr_layer_shell_v1::{
            self as layer_shell, Layer, ZwlrLayerShellV1 as LayerShell
        },
        zwlr_layer_surface_v1::{
            self as layer_surface, Anchor,
            ZwlrLayerSurfaceV1 as WlrLayerSurface
        }
    }
};

use crate::{
    area::{Margin, Size},
    wayland::{self, Buffer}
};

/// The minimum version of the wl_layer_shell protocol to bind to.
pub const LAYER_SHELL_VERSION: u32 = 1;

thread_local! {
    /// The layer shell surface creator.
    ///
    /// This should remain local to just this module.
    static LAYER_SHELL_CREATOR: RefCell<Option<LayerShell>> =
        RefCell::new(None);
}

#[derive(Eq, PartialEq)]
pub struct LayerSurfaceState {
    pub wl_surface: WlSurface,
    pub buffer: Option<Buffer>,
    pub configuration_serial: Option<u32>
}

pub struct LayerSurface {
    proxy: WlrLayerSurface
}

impl fmt::Debug for LayerSurface {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.proxy.as_ref().c_ptr())
    }
}

impl LayerSurface {
    pub fn set_size(&self, size: Size) {
        self.proxy.set_size(size.width, size.height);
    }

    pub fn set_margin(&self, margin: Margin) {
        warn!("Margin: {:#?}", margin);
        self.proxy.set_margin(
            margin.top,
            margin.right,
            margin.bottom,
            margin.left
        );
    }

    pub fn clear_surface(&mut self) {
        let mut state = unwrap_state(self.proxy.as_ref()).borrow_mut();
        state.wl_surface.attach(None, 0, 0);
        state.buffer = None;
    }

    pub fn set_surface(&mut self, size: Size) -> Result<(), ()> {
        let buffer = wayland::create_buffer(size)?;
        let mut state = unwrap_state(self.proxy.as_ref()).borrow_mut();
        state.wl_surface.attach(Some(&buffer.buffer), 0, 0);
        state.buffer = Some(buffer);
        Ok(())
    }

    pub fn write_to_buffer(&mut self, data: &[u8]) -> Result<(), io::Error> {
        let mut state = unwrap_state(self.proxy.as_ref()).borrow_mut();
        let buffer = state.buffer.as_mut().expect("buffer was none");

        buffer.shared_memory.write(data)?;
        buffer.shared_memory.flush()?;
        Ok(())
    }

    pub fn commit(&self) {
        let state = unwrap_state(self.proxy.as_ref()).borrow_mut();
        state.wl_surface.commit();
    }
}

struct LayerShellHandler;

struct LayerSurfaceHandler {
    wl_surface: WlSurface,
    buffer: Option<WlBuffer>,
    configuration_serial: Option<u32>
}

impl LayerSurfaceHandler {
    fn new(wl_surface: WlSurface) -> Self {
        LayerSurfaceHandler {
            wl_surface,
            buffer: None,
            configuration_serial: None
        }
    }
}

impl layer_shell::EventHandler for LayerShellHandler {}

impl layer_surface::EventHandler for LayerSurfaceHandler {
    fn configure(
        &mut self,
        layer_surface: WlrLayerSurface,
        serial: u32,
        _width: u32,
        _height: u32
    ) {
        if self.buffer.is_some() {
            self.wl_surface.attach(self.buffer.as_ref(), 0, 0);
            self.wl_surface.commit();
        }

        self.configuration_serial = Some(serial);
        layer_surface.ack_configure(serial);

        let mut state = unwrap_state(layer_surface.as_ref()).borrow_mut();
        state.configuration_serial = Some(serial);

        if let Some(buffer) = state.buffer.as_ref() {
            state.wl_surface.attach(Some(&buffer.buffer), 0, 0);
            state.wl_surface.commit();
        }
    }

    fn closed(&mut self, _: WlrLayerSurface) {}
}

pub fn layer_shell_init(new_proxy: NewProxy<LayerShell>) -> LayerShell {
    let layer_shell = new_proxy.implement(LayerShellHandler, ());

    LAYER_SHELL_CREATOR.with(|shell_creator| {
        *shell_creator.borrow_mut() = Some(layer_shell.clone());
    });

    layer_shell
}

pub fn create_layer_surface(
    output: Option<&WlOutput>,
    layer: Layer,
    namespace: String
) -> Result<LayerSurface, ()> {
    let surface = crate::wayland::create_surface()?;

    let layer_surface = LAYER_SHELL_CREATOR.with(|creator| {
        let creator = creator.borrow();
        let creator = creator
            .as_ref()
            .expect("Layer shell creator not initilized");
        creator
            .get_layer_surface(
                &surface,
                output,
                layer,
                namespace,
                |new_proxy| {
                    let state = LayerSurfaceState {
                        wl_surface: surface.clone(),
                        buffer: None,
                        configuration_serial: None
                    };
                    let proxy = new_proxy.implement(
                        LayerSurfaceHandler::new(surface.clone()),
                        RefCell::new(state)
                    );

                    proxy.set_anchor(Anchor::Top | Anchor::Left);
                    proxy
                }
            )
            .map(|proxy| LayerSurface { proxy })
    })?;

    layer_surface.commit();

    Ok(layer_surface)
}

fn unwrap_state(proxy: &Proxy<WlrLayerSurface>) -> &RefCell<LayerSurfaceState> {
    proxy
        .user_data::<RefCell<LayerSurfaceState>>()
        .expect("User data has not been set yet")
}
