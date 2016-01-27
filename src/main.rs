extern crate rustwlc;

use rustwlc::types;
use rustwlc::types::*;
use rustwlc::types::interface;
use rustwlc::types::interface::*;

fn start_interactive_action(view: rustwlc::WLCHandle, origin: rustwlc::Point) -> bool
{
    data.handle = view;
    data.grab = origin;
    view.bring_to_front();
    return true;
}

fn main() {
    let interface = WlcInterface {
        output: OutputInterface {
            created: output_created
        }
    };
    println!("Hello, world!");

    if !rustwlc::init(interface, 0, "") {
        panic!("Unable to initialize wlc!");
    }
    rustwlc::run();
}

// Hook up basic callbacks

fn output_created(handle: WlcHandle) -> bool {
    println!("output_created");
    return true;
}

fn output_destroyed(handle: WlcHandle) {
    println!("output_destroyed");
}

fn output_focus(handle: WlcHandle, focused: bool) {
    println!("output_focus: {}", focused);
}

fn output_resolution(handle: WlcHandle, old_size: Size, new_size: Size) {
    println!("output_resolution: {} to {}", old_size, new_size)
}

fn output_render_pre(handle: WlcHandle) {
    println!("output_render_pre");
}

fn output_render_post(handle: WlcHandle) {
    println!("output_render_post");
}

fn view_created(handle: WlcHandle) -> bool {
    println!("view_created");
    true
}

fn view_destroyed(handle: WlcHandle) {
    println!("view_destroyed");
}

fn view_focus(current: WlcHandle, focused: bool) {
    println!("view_focus: {}", focused);
}

fn view_move_to_output(current: WlcHandle, q1: WlcHandle, q2: WlcHandle) {
    println!("view_move_to_output");
}

fn view_request(handle: WlcHandle) {
    println!("view_request");
}

fn keyboard_key(handle: WlcHandle, button: libc::c_uint, mods: KeyboardModifiers, time: u32,
                state: ButtonState, point: Point) -> bool {
    true
}

fn pointer_button(handle: WlcHandle, button: libc::c_uint, mods: KeyboardModifiers,
                  time: u32, state: ButtonState, point: Point) -> bool {
    println!("pointer_button: time {}, point {}", time, point);
    true
}

fn pointer_scroll(handle: WlcHandle, button: u32, mods: KeyboardModifiers,
                  axis: ScrollAxis, heights: [u64; 2]) -> bool {
    println!("pointer_scroll: time: {}, point: {}", time, point);
    true
}

fn pointer_motion(handle: WlcHandle, dist: u32, point: Point) -> bool {
    true
}

fn touch_touch(handle: WlcHandle, time: libc::c_uint, mods: KeyboardModifiers,
               touch: TouchType, key: i32, point: Point) -> bool {
    true
}

fn compositor_ready() {
    println!("Preparing compositor!");
}

fn input_created(device: LibntputDevice) {
    println!("input_created");
}

fn input_destroyed(device: LibinputDevice) {
    println!("input_destroyed");
}
