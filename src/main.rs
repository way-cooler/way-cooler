#![feature(libc)]
extern crate libc;

extern crate rustwlc;

use rustwlc::types;
use rustwlc::types::*;
//use rustwlc::types::LibinputDevice;
use rustwlc::handle::WlcHandle;
use rustwlc::types::interface::*;

fn main() {
    let interface: WlcInterface = WlcInterface {
        output: OutputInterface {
            created: output_created,
            destroyed: output_destroyed,
            focus: output_focus,
            resolution: output_resolution,

            render: RenderInterface {
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
                render: RenderInterface {
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

extern fn output_created(handle: WlcHandle) -> bool {
    println!("output_created");
    return true;
}

extern fn output_destroyed(handle: WlcHandle) {
    println!("output_destroyed");
}

extern fn output_focus(handle: WlcHandle, focused: bool) {
    println!("output_focus: {}", focused);
}

extern fn output_resolution(handle: WlcHandle, old_size: Size, new_size: Size) {
    println!("output_resolution: {:?} to {:?}", old_size, new_size)
}

extern fn output_render_pre(handle: WlcHandle) {
    println!("output_render_pre");
}

extern fn output_render_post(handle: WlcHandle) {
    println!("output_render_post");
}

extern fn view_created(handle: WlcHandle) -> bool {
    println!("view_created");
    true
}

extern fn view_destroyed(handle: WlcHandle) {
    println!("view_destroyed");
}

extern fn view_focus(current: WlcHandle, focused: bool) {
    println!("view_focus: {}", focused);
}

extern fn view_move_to_output(current: WlcHandle, q1: WlcHandle, q2: WlcHandle) {
    println!("view_move_to_output");
}

extern fn view_request_geometry(handle: WlcHandle, geometry: Geometry) {
    println!("view_request_geometry: call wlc_view_set_geometry({:?})", geometry);
}

extern fn view_request_state(handle: WlcHandle, state: ViewState, handled: bool) {
    println!("view_request_state: call wlc_view_set_state({:?})", state);
}

extern fn view_request_move(handle: WlcHandle, dest: Point) {
    println!("view_request_move: to {}, start interactive mode.", dest);
}

extern fn view_request_resize(handle: WlcHandle, edge: ResizeEdge, location: Point) {
    println!("view_request_resize: size {:?}, to {}, start interactive mode.",
             edge, location);
}

extern fn view_request_render_pre(handle: WlcHandle) {
    println!("view_request_render_pre");
}

extern fn view_request_render_post(handle: WlcHandle) {
    println!("view_request_render_post");
}

extern fn keyboard_key(handle: WlcHandle, time: u32, mods: KeyboardModifiers,
                       key: u32, state: KeyState) -> bool {
    false
}

extern fn pointer_button(handle: WlcHandle, button: libc::c_uint, mods: KeyboardModifiers,
                  time: u32, state: ButtonState, point: Point) -> bool {
    println!("pointer_button: time {}, point {}", time, point);
    false
}

extern fn pointer_scroll(handle: WlcHandle, button: u32, mods: KeyboardModifiers,
                  axis: ScrollAxis, heights: [u64; 2]) -> bool {
    println!("pointer_scroll");
    false
}

extern fn pointer_motion(handle: WlcHandle, dist: u32, point: Point) {
    println!("Pointer moved {} pixels? to {}", dist, point);
}

extern fn touch_touch(handle: WlcHandle, time: libc::c_uint, mods: KeyboardModifiers,
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
