//! The trait that should be implemented for every variant in the `Modes`
//! enum. These define what to do on each wlc callback.
//!
//! If any are left unspecified, they execute the `Default` operation.
use rustwlc::*;
use super::default::Default;


pub trait Mode {
    fn output_created(&self, output: WlcOutput) -> bool {
        Default.output_created(output)
    }
    fn output_destroyed(&self, output: WlcOutput) {
        Default.output_destroyed(output)
    }
    fn output_focused(&self, output: WlcOutput, focused: bool) {
        Default.output_focused(output, focused)
    }
    fn output_resolution(&self, output: WlcOutput,
                         old_size: Size, new_size: Size) {
        Default.output_resolution(output, old_size, new_size)
    }
    fn output_render_post(&self, output: WlcOutput) {
        Default.output_render_post(output)
    }
    fn view_moved_to_output(&self,
                            view: WlcView,
                            o1: WlcOutput,
                            o2: WlcOutput) {
        Default.view_moved_to_output(view, o1, o2)
    }
    fn view_created(&self, view: WlcView) -> bool {
        Default.view_created(view)
    }
    fn view_destroyed(&self, view: WlcView) {
        Default.view_destroyed(view)
    }
    fn view_focused(&self, view: WlcView, focused: bool) {
        Default.view_focused(view, focused)
    }
    fn view_props_changed(&self, view: WlcView, prop: ViewPropertyType) {
        Default.view_props_changed(view, prop)
    }
    fn view_request_state(&self,
                          view: WlcView,
                          state: ViewState,
                          toggle: bool) {
        Default.view_request_state(view, state, toggle)
    }
    fn view_request_geometry(&self, view: WlcView, geometry: Geometry) {
        Default.view_request_geometry(view, geometry)
    }
    fn view_request_move(&self, view: WlcView, dest: Point) {
        Default.view_request_move(view, dest)
    }
    fn view_request_resize(&self,
                           view: WlcView,
                           edge: ResizeEdge,
                           point: Point) {
        Default.view_request_resize(view, edge, point)
    }
    fn view_pre_render(&self, view: WlcView) {
        Default.view_pre_render(view)
    }
    fn on_keyboard_key(&self,
                       view: WlcView,
                       time: u32,
                       mods: KeyboardModifiers,
                       key: u32,
                       state: KeyState) -> bool {
        Default.on_keyboard_key(view, time, mods, key, state)
    }
    fn on_pointer_button(&self,
                         view: WlcView,
                         time: u32,
                         mods: KeyboardModifiers,
                         button: u32,
                         state: ButtonState,
                         point: Point) -> bool {
        Default.on_pointer_button(view, time, mods, button, state, point)
    }
    fn on_pointer_scroll(&self,
                         view: WlcView,
                         time: u32,
                         mods: KeyboardModifiers,
                         axis: ScrollAxis,
                         heights: [f64; 2]) -> bool {
        Default.on_pointer_scroll(view, time, mods, axis, heights)
    }
    fn on_pointer_motion(&self,
                         view: WlcView,
                         time: u32,
                         point: Point) -> bool {
        Default.on_pointer_motion(view, time, point)
    }
}
