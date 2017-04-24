//! Way Cooler exists in one of several different "Modes"
//! The current mode defines what Way Cooler does in each callback.
//!
//! The central use of this is to define different commands that the user can
//! run.
//!
//! For example, when the lock screen mode is active the user can't do anything
//! other than send input to the lock screen program.
//!
//! We also allow users to define their own custom modes, which allows them to
//! hook into the callbacks from Lua. In the `CustomLua` mode, callbacks do
//! the same thing as they do in the `Default` mode, but at the end of will
//! always execute some custom Lua code.

use std::ops::Deref;
use rustwlc::*;

mod default;
mod custom_lua;
pub use self::default::Default;
pub use self::custom_lua::CustomLua;

/// The different modes that Way Cooler can be in
/// * `Default`: The default mode for Way Cooler, this is the standard mode
/// that it starts out in
/// * `CustomLua`: Same as `Default`, except it calls any custom defined
/// callbacks in the Lua configuration file at the end of the call back.

pub enum Modes {
    Default(Default),
    CustomLua(CustomLua)
}

/// If the event is handled by way-cooler
const EVENT_BLOCKED: bool = true;
/// If the event should be passed through to clients
const EVENT_PASS_THROUGH: bool = false;
const LEFT_CLICK: u32 = 0x110;
const RIGHT_CLICK: u32 = 0x111;

pub trait Mode {
    fn output_created(&mut self, output: WlcOutput) -> bool {
        Default.output_created(output)
    }
    fn output_destroyed(&mut self, output: WlcOutput) {
        Default.output_destroyed(output)
    }
    fn output_focused(&mut self, output: WlcOutput, focused: bool) {
        Default.output_focused(output, focused)
    }
    fn output_resolution(&mut self, output: WlcOutput,
                         old_size: Size, new_size: Size) {
        Default.output_resolution(output, old_size, new_size)
    }
    fn output_render_post(&mut self, output: WlcOutput) {
        Default.output_render_post(output)
    }
    fn view_created(&mut self, view: WlcView) -> bool {
        Default.view_created(view)
    }
    fn view_destroyed(&mut self, view: WlcView) {
        Default.view_destroyed(view)
    }
    fn view_focused(&mut self, view: WlcView, focused: bool) {
        Default.view_focused(view, focused)
    }
    fn view_props_changed(&mut self, view: WlcView, prop: ViewPropertyType) {
        Default.view_props_changed(view, prop)
    }
    fn view_request_state(&mut self,
                          view: WlcView,
                          state: ViewState,
                          toggle: bool) {
        Default.view_request_state(view, state, toggle)
    }
    fn view_request_geometry(&mut self, view: WlcView, geometry: Geometry) {
        Default.view_request_geometry(view, geometry)
    }
    fn view_request_move(&mut self, view: WlcView, dest: Point) {
        Default.view_request_move(view, dest)
    }
    fn view_request_resize(&mut self,
                           view: WlcView,
                           edge: ResizeEdge,
                           point: Point) {
        Default.view_request_resize(view, edge, point)
    }
    fn view_pre_render(&mut self, view: WlcView) {
        Default.view_pre_render(view)
    }
    fn on_keyboard_key(&mut self,
                       view: WlcView,
                       time: u32,
                       mods: KeyboardModifiers,
                       key: u32,
                       state: KeyState) -> bool {
        Default.on_keyboard_key(view, time, mods, key, state)
    }
    fn on_pointer_button(&mut self,
                         view: WlcView,
                         time: u32,
                         mods: KeyboardModifiers,
                         button: u32,
                         state: ButtonState,
                         point: Point) -> bool {
        Default.on_pointer_button(view, time, mods, button, state, point)
    }
    fn on_pointer_scroll(&mut self,
                         view: WlcView,
                         time: u32,
                         mods: KeyboardModifiers,
                         axis: ScrollAxis,
                         heights: [f64; 2]) -> bool {
        Default.on_pointer_scroll(view, time, mods, axis, heights)
    }
    fn on_pointer_motion(&mut self,
                         view: WlcView,
                         time: u32,
                         point: Point) -> bool {
        Default.on_pointer_motion(view, time, point)
    }
}

impl Deref for Modes {
    type Target = Mode;

    fn deref(&self) -> &(Mode + 'static) {
        match *self {
            Modes::Default(ref mode) => mode as &Mode,
            Modes::CustomLua(ref mode) => mode as &Mode
        }
    }
}
