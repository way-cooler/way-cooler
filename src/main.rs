extern crate rustwlc;

use rustwlc::types;
use rustwlc::types::*;
use rustwlc::handle::{WlcView, WlcOutput};
use rustwlc::interface::*;
use rustwlc::input::{pointer, keyboard};

fn main() {
    let interface: WlcInterface = WlcInterface::new()
        .output_created(output_created)
        .output_destroyed(output_destroyed)
        .output_focus(output_focus)
        .output_resolution(output_resolution)
        .output_render_pre(output_render_pre)
        .output_render_post(output_render_post)
        .view_created(view_created)
        .view_destroyed(view_destroyed)
        .view_focus(view_focus)
        .view_move_to_output(view_move_to_output)
        .view_request_geometry(view_request_geometry)
        .view_request_state(view_request_state)
        .view_request_move(view_request_move)
        .view_request_resize(view_request_resize)
        .view_request_render_pre(view_request_render_pre)
        .view_request_render_post(view_request_render_post)
        .keyboard_key(keyboard_key)
        .pointer_button(pointer_button)
        .pointer_scroll(pointer_scroll)
        .pointer_motion(pointer_motion)
        .touch_touch(touch_touch)
        .compositor_ready(compositor_ready);
        //.input_created(input_created)
        //.input_destroyed(input_destroyed);

    rustwlc::log_set_default_handler();

    let run_wlc = rustwlc::init(interface).expect("Unable to initialize wlc!");
    run_wlc();
}



/// From example.c:
/// very simple layout function
/// you probably don't want to layout certain type of windows in wm
fn render_output(output: WlcOutput) {
    use std::cmp;
    let resolution = output.get_resolution();
    let views = output.get_views();
    println!("Rendering {} views for {} (resolution = {:?})", views.len(), output.get_name(), resolution);

    if views.len() == 0 { println!("Didn't find any views to render :/"); }

    for view in views {
        println!("Setting {:?}", view);
        println!("\tIts type: {:?}", view.get_type());
        println!("\tIts output: {}", view.get_output().get_name());
        println!("\tIts geometry: {:?}", view.get_geometry());
        println!("\tIts state: {:?}", view.get_state());
        println!("\tIts parent: {:?}", view.get_parent());
        println!("\tIts title: {}", view.get_title());
        // get_class doesn't work but maybe it's not supposed to
        //println!("\tIts class: {}", view.get_class());
        view.set_geometry(EDGE_NONE,
                          &Geometry {
                              size: Size {
                                  w: 0 as u32,
                                  h: 0 as u32
                              },
                              origin: Point {
                                  x: resolution.w as i32,
                                  y: resolution.h as i32
                              }
                          });
        println!("Attempted to set geometry, got {:?}", view.get_geometry());
    }
}


// Hook up basic callbacks

extern fn output_created(output: WlcOutput) -> bool {
    println!("output_created: {:?}: {}", output, output.get_name());
    return true;
}

extern fn output_destroyed(output: WlcOutput) {
    println!("output_destroyed");
}

extern fn output_focus(output: WlcOutput, focused: bool) {
    println!("output_focus: {}", focused);
}

extern fn output_resolution(output: WlcOutput,
                            old_size_ptr: &Size, new_size_ptr: &Size) {
    println!("output_resolution: {:?} from  {:?} to {:?}",
             output, *old_size_ptr, *new_size_ptr);
}

extern fn output_render_pre(output: WlcOutput) {
    //println!("output_render_pre");
}

extern fn output_render_post(output: WlcOutput) {
    //println!("output_render_post");
}

extern fn view_created(view: WlcView) -> bool {
    println!("view_created: {:?}: \"{}\"", view, view.get_title());
    let output = view.get_output();
    view.set_mask(output.get_mask());
    view.bring_to_front();
    view.focus();
    render_output(output);
    true
}

extern fn view_destroyed(view: WlcView) {
    println!("view_destroyed: {:?}", view);
    render_output(view.get_output());
}

extern fn view_focus(current: WlcView, focused: bool) {
    println!("view_focus: {:?} {}", current, focused);
    current.set_state(VIEW_ACTIVATED, focused);
}

extern fn view_move_to_output(current: WlcView, o1: WlcOutput, o2: WlcOutput) {
    println!("view_move_to_output: {:?}, {:?}, {:?}", current, o1, o2);
}

extern fn view_request_geometry(view: WlcView, geometry: &Geometry) {
    println!("view_request_geometry: {:?} wants {:?}", view, geometry);
    view.set_geometry(EDGE_NONE, geometry);
    render_output(view.get_output());
}

extern fn view_request_state(view: WlcView, state: ViewState, handled: bool) {
    view.set_state(state, handled);
}

extern fn view_request_move(view: WlcView, dest: &Point) {
    //println!("view_request_move: to {}, start interactive mode.", *dest);
}

extern fn view_request_resize(view: WlcView, edge: ResizeEdge, location: &Point) {
    println!("view_request_resize: edge {:?}, to {}, start interactive mode.",
             edge, location);
}

extern fn view_request_render_pre(view: WlcView) {
}

extern fn view_request_render_post(view: WlcView) {
}

extern fn keyboard_key(view: WlcView, time: u32, mods_ptr: &KeyboardModifiers,
                       key: u32, state: KeyState) -> bool {
    println!("keyboard_key: time {}, mods {:?}, key {:?}, state {:?}",
             time, &*mods_ptr, key, state);
    false
}

extern fn pointer_button(view: WlcView, button: u32, mods_ptr: &KeyboardModifiers,
                         key: u32, state: ButtonState, point_ptr: &Point) -> bool {
    println!("pointer_button: pressed {} at {}", key, *point_ptr);
    if state == ButtonState::Pressed && !view.is_root() {
        view.focus();
    }
    false
}

extern fn pointer_scroll(view: WlcView, button: u32, mods_ptr: &KeyboardModifiers,
                  axis: ScrollAxis, heights: [u64; 2]) -> bool {
    println!("pointer_scroll");
    false
}

extern fn pointer_motion(view: WlcView, time: u32, point: &Point) -> bool {
    pointer::set_position(point);
    false
}

extern fn touch_touch(view: WlcView, time: u32, mods_ptr: &KeyboardModifiers,
               touch: TouchType, key: i32, point_ptr: &Point) -> bool {
    false
}

extern fn compositor_ready() {
    println!("Preparing compositor!");
}

extern fn input_created(device: &LibinputDevice) -> bool {
    println!("input_created");
    false
}

extern fn input_destroyed(device: &LibinputDevice) {
    println!("input_destroyed");
}
