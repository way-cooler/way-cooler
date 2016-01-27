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
        }
    };
    println!("Created interface {:?}", interface);

    if !rustwlc::init(&interface) {
        panic!("Unable to initialize wlc!");
    }
    rustwlc::run_wlc();
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

fn view_request_geometry(handle: WlcHandle, geometry: Geometry) {
    println!("view_request_geometry: call wlc_view_set_geometry({:?})", geometry);
}

fn view_request_state(handle: WlcHandle, state: ViewState) {
    println!("view_request_state: call wlc_view_set_state({:?})", state);
}

fn view_request_move(handle: WlcHandle, dest: Point) {
    println!("view_request_move: to {}, start interactive mode.", dest);
}

fn view_request_resize(handle: WlcHandle, edge: ResizeEdge, location: Point) {
    println!("view_request_resize: size {:?}, to {}, start interactive mode.",
             edge, location);
}

fn view_request_render_pre(handle: WlcHandle) {
    println!("view_request_render_pre");
}

fn view_request_render_post(handle: WlcHandle) {
    println!("view_request_render_post");
}

fn keyboard_key(handle: WlcHandle, button: libc::c_uint, mods: KeyboardModifiers, time: u32,
                state: ButtonState, point: Point) -> bool {
    false
}

fn pointer_button(handle: WlcHandle, button: libc::c_uint, mods: KeyboardModifiers,
                  time: u32, state: ButtonState, point: Point) -> bool {
    println!("pointer_button: time {}, point {}", time, point);
    false
}

fn pointer_scroll(handle: WlcHandle, button: u32, mods: KeyboardModifiers,
                  axis: ScrollAxis, heights: [u64; 2]) -> bool {
    println!("pointer_scroll");
    false
}

fn pointer_motion(handle: WlcHandle, dist: u32, point: Point) -> bool {
    false
}

fn touch_touch(handle: WlcHandle, time: libc::c_uint, mods: KeyboardModifiers,
               touch: TouchType, key: i32, point: Point) -> bool {
    false
}

fn compositor_ready() {
    println!("Preparing compositor!");
}

fn input_created(device: LibinputDevice) {
    println!("input_created");
}

fn input_destroyed(device: LibinputDevice) {
    println!("input_destroyed");
}
