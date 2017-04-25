//! A custom Lua callback mapping, where the default operation is done and
//! then a custom, user-defined Lua callback is ran.
use std::collections::BTreeMap;
use rustwlc::*;
use rustwlc::Size;
use ::lua::{self, LuaQuery, LuaSendError};
use ::layout::{try_lock_tree};
use rustc_serialize::json::{Json, ToJson};
use super::default::Default;
use super::Mode;

pub struct CustomLua;

fn send_to_lua<Q: Into<String>>(msg: Q) {
    let msg = msg.into();
    match lua::send(LuaQuery::Execute(msg.clone())) {
        Ok(result) => {
            error!("Got back {:?}", result.recv())
        },
        Err(LuaSendError::Sender(err)) =>
            error!("Error while executing {:?}: {:?}", msg, err),
        Err(LuaSendError::ThreadUninitialized) =>
            error!("Thread was not initilazed yet, could not execute {:?}",
                   msg),
        Err(LuaSendError::ThreadClosed) => {
            error!("Thread closed, could not execute {:?}", msg)
        }
    }
}

fn size_to_lua(size: Size) -> String {
    let mut map = BTreeMap::new();
    map.insert("w".into(), size.w.to_json());
    map.insert("h".into(), size.h.to_json());
    format!("{}", map.to_json()).replace(":", "=").replace("\"", "")
}

fn point_to_lua(point: Point) -> String {
    let mut map = BTreeMap::new();
    map.insert("x".into(), point.x.to_json());
    map.insert("y".into(), point.y.to_json());
    format!("{}", map.to_json()).replace(":", "=").replace("\"", "")
}

impl Mode for CustomLua {
    /// Sends the id of the created output to Lua.
    fn output_created(&self, output: WlcOutput) -> bool {
        let result = Default.output_created(output);
        if let Ok(tree) = try_lock_tree() {
            if let Ok(id) = tree.lookup_handle(output.into()) {
                send_to_lua(format!("way_cooler.on_output_created(\"{}\")",
                                    id.hyphenated().to_string()));
            }
        }
        result
    }

    /// Sends the id of the destroyed output to Lua.
    fn output_destroyed(&self, output: WlcOutput) {
        // Get the output ID before it's destroyed
        let mut id = None;
        if let Ok(tree) = try_lock_tree() {
            id = tree.lookup_handle(output.into()).ok();
        }
        Default.output_destroyed(output);
        if let Some(id) = id {
            send_to_lua(format!("way_cooler.on_output_destroyed(\"{}\")",
                                id.hyphenated().to_string()))
        }
    }

    /// Sends the id of the output, and whether it gained or lost focus.
    fn output_focused(&self, output: WlcOutput, focused: bool) {
        Default.output_focused(output, focused);
        if let Ok(tree) = try_lock_tree() {
            if let Ok(id) = tree.lookup_handle(output.into()) {
                send_to_lua(format!("way_cooler.on_output_focused(\"{}\", {})",
                                    id.hyphenated().to_string(),
                                    focused));
            }
        }
    }

    /// Sends the id of the output, and the old and new size of the output.
    fn output_resolution(&self, output: WlcOutput,
                         old_size: Size, new_size: Size) {
        Default.output_resolution(output, old_size, new_size);
        if let Ok(tree) = try_lock_tree() {
            if let Ok(id) = tree.lookup_handle(output.into()) {
                send_to_lua(format!(
                    "way_cooler.on_output_resolution_change({}, {}, {})",
                    id.hyphenated().to_string(),
                    size_to_lua(old_size),
                    size_to_lua(new_size)));
            }
        }
    }

    fn output_render_post(&self, output: WlcOutput) {
        Default.output_render_post(output);
        send_to_lua("way_cooler.on_output_render_post()")
    }
    fn view_moved_to_output(&self,
                            view: WlcView,
                            o1: WlcOutput,
                            o2: WlcOutput) {
        Default.view_moved_to_output(view, o1, o2);
        send_to_lua("way_cooler.on_view_moved_to_output()")
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

    /// Sends the UUID of the pointer clicked, any modifiers that were held
    /// down, where absolutely on the screen it was clicked, and what button
    /// was used to Lua.
    fn on_pointer_button(&self,
                         view: WlcView,
                         time: u32,
                         mods: KeyboardModifiers,
                         button: u32,
                         state: ButtonState,
                         point: Point) -> bool {
        let result = Default.on_pointer_button(view, time, mods, button, state, point);
        warn!("Sending button thing to Lua!");
        if let Ok(tree) = try_lock_tree() {
            if let Ok(id) = tree.lookup_handle(view.into()) {
                warn!("Ok really sending it!");
                send_to_lua(format!("way_cooler.on_pointer_button(\
                                     \"{view}\", {mods}, {pos}, {button})",
                                    view = id,
                                    mods = mods.mods.bits(),
                                    pos = point_to_lua(point),
                                    button = button))
            }
        }
        result
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
