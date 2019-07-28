//! Wrapper around a wl_output

use std::{cell::RefCell, fmt};

use wayland_client::{
    protocol::wl_output::{self, WlOutput},
    GlobalImplementor, NewProxy, Proxy
};

use crate::{
    area::{Area, Origin, Size},
    lua::LUA,
    objects::screen::{self, Screen}
};

/// The minimum version of the wl_output global to bind to.
pub const WL_OUTPUT_VERSION: u32 = 2;

/// Wrapper around WlOutput.
#[derive(Clone, Eq, PartialEq)]
pub struct Output {
    output: WlOutput
}

// Provides new WlOutputs with an implementation.
pub struct WlOutputManager;

// Handle incoming events for WlOutput.
pub struct WlOutputEventHandler;

/// The cached state for the WlOutput.
#[derive(Debug, Default, Clone, Eq, PartialEq)]
struct OutputState {
    name: String,
    geometry: Area
}

impl Output {
    pub fn resolution(&self) -> Size {
        unwrap_state(self.as_ref()).borrow().geometry.size
    }

    pub fn name(&self) -> String {
        unwrap_state(self.as_ref()).borrow().name.clone()
    }
}

impl GlobalImplementor<WlOutput> for WlOutputManager {
    fn new_global(&mut self, new_proxy: NewProxy<WlOutput>) -> WlOutput {
        let res = new_proxy.implement(
            WlOutputEventHandler,
            RefCell::new(OutputState::default())
        );

        LUA.with(|lua| {
            lua.borrow().context(|ctx| {
                let output = Output {
                    output: res.clone()
                };
                let mut screen =
                    Screen::new(ctx).expect("Could not allocate new screen");
                screen
                    .init_screens(output.clone(), vec![output])
                    .expect("Could not initilize new output with a screen");
                screen::add_screen(ctx, screen)
                    .expect("Could not add screen to the list of screens");
            })
        });

        res
    }
}

impl wl_output::EventHandler for WlOutputEventHandler {
    #[allow(unused)]
    fn geometry(
        &mut self,
        object: WlOutput,
        x: i32,
        y: i32,
        physical_width: i32,
        physical_height: i32,
        subpixel: wl_output::Subpixel,
        make: String,
        model: String,
        transform: wl_output::Transform
    ) {
        let mut state = unwrap_state(object.as_ref()).borrow_mut();
        state.name = format!("{} ({})", make, model);
        state.geometry.origin = Origin { x, y };
    }

    #[allow(unused)]
    fn mode(
        &mut self,
        object: WlOutput,
        flags: wl_output::Mode,
        width: i32,
        height: i32,
        refresh: i32
    ) {
        unwrap_state(object.as_ref()).borrow_mut().geometry.size = Size {
            width: width as _,
            height: height as _
        };
        let geometry = Area {
            origin: Origin { x: 0, y: 0 },
            size: Size {
                width: width as u32,
                height: height as u32
            }
        };
        LUA.with(|lua| {
            lua.borrow().context(|ctx| {
                if let Ok(Some(mut screen)) =
                    screen::get_screen(ctx, &Output { output: object })
                {
                    screen
                        .set_geometry(ctx, geometry)
                        .expect("could not set geometry");
                    screen
                        .set_workarea(ctx, geometry)
                        .expect("could not set workarea ");
                }
            });
        });
    }

    #[allow(unused)]
    fn done(&mut self, object: WlOutput) {
        // TODO We may not always want to add a new screen
        // see how awesome does it and fix this.
        LUA.with(|lua| {
            lua.borrow().context(|ctx| {
                let geometry = unwrap_state(object.as_ref()).borrow().geometry;
                let output = Output { output: object };
                match screen::get_screen(ctx, &output) {
                    Ok(Some(mut screen)) => {
                        screen
                            .set_geometry(ctx, geometry)
                            .expect("could not set geometry");
                        screen
                            .set_workarea(ctx, geometry)
                            .expect("could not set workarea ");
                    },
                    Ok(None) => {
                        let mut screen = Screen::new(ctx)
                            .expect("Could not allocate new screen");
                        screen
                            .init_screens(output.clone(), vec![output])
                            .expect(
                                "Could not initilize new output with a screen"
                            );
                        screen::add_screen(ctx, screen).expect(
                            "Could not add screen to the list of screens"
                        );
                    },
                    Err(err) => warn!("Could not add screen: {:?}", err)
                }
            });
        });
    }

    #[allow(unused)]
    fn scale(&mut self, object: WlOutput, factor: i32) {
        // TODO
    }
}

impl fmt::Debug for Output {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.output.as_ref().c_ptr())
    }
}

impl AsRef<Proxy<WlOutput>> for Output {
    fn as_ref(&self) -> &Proxy<WlOutput> {
        &self.output.as_ref()
    }
}

fn unwrap_state(proxy: &Proxy<WlOutput>) -> &RefCell<OutputState> {
    proxy
        .user_data::<RefCell<OutputState>>()
        .expect("User data has not been set yet")
}
