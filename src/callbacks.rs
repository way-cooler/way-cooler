//! Callback methods for rustwlc
use rustwlc::handle::{WlcOutput, WlcView};
use rustwlc::types::*;
use rustwlc::input::{pointer, keyboard};
use rustwlc::xkb::Keysym;

use super::keys;
use super::lua;
use super::keys::{KeyEvent, KeyPress};

/// If the event is handled by way-cooler
const EVENT_HANDLED: bool = true;

/// If the event should be passed through to clients
const EVENT_PASS_THROUGH: bool = false;

// wlc callbacks

pub extern fn output_created(output: WlcOutput) -> bool {
    println!("output_created: {:?}: {}", output, output.get_name());
    return true;
}

pub extern fn output_destroyed(output: WlcOutput) {
    println!("output_destroyed: {:?}", output);
}

pub extern fn output_focus(output: WlcOutput, focused: bool) {
    println!("output_focus: {:?} focus={}", output, focused);
}

pub extern fn output_resolution(output: WlcOutput,
                            old_size_ptr: &Size, new_size_ptr: &Size) {
    println!("output_resolution: {:?} from  {:?} to {:?}",
             output, *old_size_ptr, *new_size_ptr);
}
/*
pub extern fn output_render_pre(output: WlcOutput) {
    //println!("output_render_pre");
}

pub extern fn output_render_post(output: WlcOutput) {
    //println!("output_render_post");
}
*/
pub extern fn view_created(view: WlcView) -> bool {
    println!("view_created: {:?}: \"{}\"", view, view.get_title());
    let output = view.get_output();
    view.set_mask(output.get_mask());
    view.bring_to_front();
    view.focus();
    return true;
}

pub extern fn view_destroyed(view: WlcView) {
    println!("view_destroyed: {:?}", view);
}

pub extern fn view_focus(current: WlcView, focused: bool) {
    println!("view_focus: {:?} {}", current, focused);
    current.set_state(VIEW_ACTIVATED, focused);
}

pub extern fn view_move_to_output(current: WlcView,
                                  o1: WlcOutput, o2: WlcOutput) {
    println!("view_move_to_output: {:?}, {:?}, {:?}", current, o1, o2);
}

pub extern fn view_request_geometry(view: WlcView, geometry: &Geometry) {
    println!("view_request_geometry: {:?} wants {:?}", view, geometry);
    view.set_geometry(EDGE_NONE, geometry);
}

pub extern fn view_request_state(view: WlcView, state: ViewState, handled: bool) {
    println!("view_request_state: {}, {:?}, handled: {}",
    view.get_title(), state, handled);
    view.set_state(state, handled);
}

pub extern fn view_request_move(view: WlcView, dest: &Point) {
    // Called by views when they have a dang resize mouse thing, we should only
    // let it happen in view floating mode
    println!("view_request_move: to {}, start interactive mode.", *dest);
}

pub extern fn view_request_resize(view: WlcView,
                              edge: ResizeEdge, location: &Point) {
    println!("view_request_resize: edge {:?}, to {}, start interactive mode.",
             edge, location);
}

pub extern fn keyboard_key(_view: WlcView, _time: u32, mods: &KeyboardModifiers,
                       key: u32, state: KeyState) -> bool {
    println!("[key] pressed, mods {:?}, key {:?}, state {:?}",
             &*mods, key, state);

    if state == KeyState::Pressed {
        // TODO this function will throw an error in Rustwlc right now
        // let mut keys = keyboard::get_current_keys().into_iter()
        //      .map(|&k| Keysym::from(k)).collect();
        let sym = keyboard::get_keysym_for_key(key, &KeyMod::empty());
        println!("[key] Found sym named {}", sym.get_name().unwrap());
        let mut keys = vec![sym];

        let press = KeyPress::new(mods.mods, keys);
        println!("[key] Created Keypress {:?}", press);

        if let Some(action) = keys::get(&press) {
            println!("[key] Found a key!");
            action();
            return EVENT_HANDLED;
        }
        else {
            println!("[key] No keypresses found.");
        }
    }

    return EVENT_PASS_THROUGH;
}

pub extern fn pointer_button(view: WlcView, button: u32,
                         mods_ptr: &KeyboardModifiers, key: u32,
                         state: ButtonState, point_ptr: &Point) -> bool {
    println!("pointer_button: pressed {} at {}", key, *point_ptr);
    if state == ButtonState::Pressed && !view.is_root() {
        view.focus();
    }
    false
}

pub extern fn pointer_scroll(_view: WlcView, button: u32,
                         _mods_ptr: &KeyboardModifiers, axis: ScrollAxis,
                         heights: [u64; 2]) -> bool {
    println!("pointer_scroll: press {}, {:?} to {:?}", button, axis, heights);
    false
}

pub extern fn pointer_motion(_view: WlcView, _time: u32, point: &Point) -> bool {
    pointer::set_position(point);
    false
}

/*
pub extern fn touch(view: WlcView, time: u32, mods_ptr: &KeyboardModifiers,
               touch: TouchType, key: i32, point_ptr: &Point) -> bool {
    false
}
*/

pub extern fn compositor_ready() {
    println!("Preparing compositor!");
    lua::init();
}
