#![feature(libc)]
extern crate libc;

extern crate rustwlc;

use rustwlc::types;
use rustwlc::types::*;
//use rustwlc::types::LibinputDevice;
use rustwlc::handle::{WlcView, WlcOutput};
use rustwlc::types::interface::*;

fn main() {
    let interface: WlcInterface = WlcInterface {
        output: OutputInterface {
            created: output_created,
            destroyed: output_destroyed,
            focus: output_focus,
            resolution: output_resolution,

            render: OutputRenderInterface {
                pre: output_render_pre,
                post: output_render_post
            }
        },
        view: ViewInterface {
            created: view_created,
            destroyed: view_destroyed,
            focus: view_focus,
            move_to_output: view_move_to_output,
            request: RequestInterface {
                geometry: view_request_geometry,
                state: view_request_state,
                move_: view_request_move,
                resize: view_request_resize,
                render: ViewRenderInterface {
                    pre: view_request_render_pre,
                    post: view_request_render_post
                }
            }
        },
        keyboard: KeyboardInterface {
            key: keyboard_key
        },
        pointer: PointerInterface {
            button: pointer_button,
            scroll: pointer_scroll,
            motion: pointer_motion
        },
        touch: TouchInterface {
            touch: touch_touch
        },
        compositor: CompositorInterface {
            ready: compositor_ready
        },
        input: InputInterface {
            created: input_created,
            destroyed: input_destroyed
        }
    };
    // Interfaces don't derive debug
    //println!("Created interface {:?}", interface);

    if !rustwlc::init(interface) {
        panic!("Unable to initialize wlc!");
    }
    rustwlc::run_wlc();
}

// Hook up basic callbacks

extern fn output_created(output: WlcOutput) -> bool {
    println!("output_created");
    return true;
}

extern fn output_destroyed(output: WlcOutput) {
    println!("output_destroyed");
}

extern fn output_focus(output: WlcOutput, focused: bool) {
    println!("output_focus: {}", focused);
}

extern fn output_resolution(output: WlcOutput, old_size: Size, new_size: Size) {
    println!("output_resolution: {:?} to {:?}", old_size, new_size)
}

extern fn output_render_pre(output: WlcOutput) {
    println!("output_render_pre");
}

extern fn output_render_post(output: WlcOutput) {
    println!("output_render_post");
}

extern fn view_created(view: WlcView) -> bool {
    println!("view_created");
    true
}

extern fn view_destroyed(view: WlcView) {
    println!("view_destroyed");
}

extern fn view_focus(current: WlcView, focused: bool) {
    println!("view_focus: {}", focused);
}

extern fn view_move_to_output(current: WlcView, q1: WlcView, q2: WlcView) {
    println!("view_move_to_output");
}

extern fn view_request_geometry(view: WlcView, geometry: Geometry) {
    println!("view_request_geometry: call wlc_view_set_geometry({:?})", geometry);
}

extern fn view_request_state(view: WlcView, state: ViewState, handled: bool) {
    println!("view_request_state: call wlc_view_set_state({:?})", state);
}

extern fn view_request_move(view: WlcView, dest: Point) {
    println!("view_request_move: to {}, start interactive mode.", dest);
}

extern fn view_request_resize(view: WlcView, edge: ResizeEdge, location: Point) {
    println!("view_request_resize: size {:?}, to {}, start interactive mode.",
             edge, location);
}

extern fn view_request_render_pre(view: WlcView) {
    println!("view_request_render_pre");
}

extern fn view_request_render_post(view: WlcView) {
    println!("view_request_render_post");
}

extern fn keyboard_key(view: WlcView, time: u32, mods: KeyboardModifiers,
                       key: u32, state: KeyState) -> bool {
    false
}

extern fn pointer_button(view: WlcView, button: libc::c_uint, mods: KeyboardModifiers,
                  time: u32, state: ButtonState, point: Point) -> bool {
    println!("pointer_button: time {}, point {}", time, point);
    false
}

extern fn pointer_scroll(view: WlcView, button: u32, mods: KeyboardModifiers,
                  axis: ScrollAxis, heights: [u64; 2]) -> bool {
    println!("pointer_scroll");
    false
}

extern fn pointer_motion(view: WlcView, dist: u32, point: Point) {
    println!("Pointer moved {} pixels? to {}", dist, point);
}

extern fn touch_touch(view: WlcView, time: libc::c_uint, mods: KeyboardModifiers,
               touch: TouchType, key: i32, point: Point) -> bool {
    false
}

extern fn compositor_ready() {
    println!("Preparing compositor!");
}

extern fn input_created(device: LibinputDevice) -> bool {
    println!("input_created");
    false
}

extern fn input_destroyed(device: LibinputDevice) {
    println!("input_destroyed");
}
