#![feature(libc)]
extern crate libc;
use std::ffi; 

extern crate rustwlc;

use rustwlc::types;
use rustwlc::types::*;
//use rustwlc::types::LibinputDevice;
use rustwlc::handle::{WlcView, WlcOutput};
use rustwlc::types::interface::*;
use rustwlc::input::pointer;
use rustwlc::input::keyboard;
/*
struct CompositorAction {
    view: WlcView,
    grab: Point,
    edges: u32
}

static mut compositor: CompositorAction = CompositorAction {
    view: WlcView(0),
    grab: Point{ x: 0, y: 0},
    edges: 0
};
*/ 
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
    rustwlc::log_set_handler(log_callback);

    if !rustwlc::init(interface) {
        panic!("Unable to initialize wlc!");
    }
    rustwlc::run_wlc();
}


extern fn log_callback(log_type: LogType, text: *const libc::c_char) {
    let string_text = rustwlc::pointer_to_string(text);
    println!("Wlc Log: {}", string_text);
}

// Important rendering functions copied from wlc/example/example.c

fn start_interactive_action(view: WlcView, origin: Point) -> bool {
    true
}

fn stop_interactive_action() {
    
}

fn start_interactive_move(view: WlcView, origin: Point) {
    
}

fn start_interactive_resize(view: WlcView, edges: u32, origin: Point) {
    
}

fn get_topmost_view(output: WlcOutput, offset: Size) {
    
}

/// From example.c:
/// very simple layout function
/// you probably don't want to layout certain type of windows in wm
fn render_output(output: WlcOutput) {
    use std::cmp;
    let resolution = output.get_resolution();
    let views = output.get_views();

    if views.len() == 0 { println!("Didn't find any views to render :/"); }

    for view in views {
        view.set_geometry(0,
                          Geometry {
                              size: Size {
                                  w: resolution.w as u32,
                                  h: resolution.h as u32
                              },
                              origin: Point {
                                  x: resolution.w as i32,
                                  y: resolution.h as i32
                              }
                          });
    }

    /*
    let mut toggle = false;
    let mut y = 0u32;
//    use std::cmp;

    let w: u32 = resolution.w as u32 / 2u32;
    let h: u32 = resolution.h as u32 / cmp::max((1u32 + views.len() as u32) / 2u32, 1);

    for view in views {
        let g = Geometry {
            size: Size {
                w: if toggle { w } else { 0 },
                h: y
            },
            origin: Point {
                x: if !toggle { resolution.w as i32 } else { w as i32 },
                y: h as i32
            }
        };
        view.set_geometry(0, g);
        toggle = !toggle;
        y = y + if !toggle { h } else { 0 };
    }*/
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
    //println!("output_render_pre");
}

extern fn output_render_post(output: WlcOutput) {
    //println!("output_render_post");
}

extern fn view_created(view: WlcView) -> bool {
    println!("view_created: {:?}", view);
    let output = view.get_output();
    println!("view_created: it's on output {:?}", output);
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
    current.set_state(ViewState::Activated, focused);
}

extern fn view_move_to_output(current: WlcView, q1: WlcView, q2: WlcView) {
    println!("view_move_to_output: {:?}, {:?}, {:?}", current, q1, q2);
}

extern fn view_request_geometry(view: WlcView, geometry: Geometry) {
    println!("view_request_geometry: {:?} wants {:?}", view, geometry);
    view.set_geometry(0, geometry);
    render_output(view.get_output());
}

extern fn view_request_state(view: WlcView, state: ViewState, handled: bool) {
    println!("view_request_state: call wlc_view_set_state({:?})", state);
    view.set_state(state, handled);
}

extern fn view_request_move(view: WlcView, dest: Point) {
    println!("view_request_move: to {}, start interactive mode.", dest);
}

extern fn view_request_resize(view: WlcView, edge: ResizeEdge, location: Point) {
    println!("view_request_resize: size {:?}, to {}, start interactive mode.",
             edge, location);
}

extern fn view_request_render_pre(view: WlcView) {
    //println!("view_request_render_pre");
}

extern fn view_request_render_post(view: WlcView) {
    //println!("view_request_render_post");
}

extern fn keyboard_key(view: WlcView, time: u32, mods: KeyboardModifiers,
                       key: u32, state: KeyState) -> bool {
    use std::process::{Command};
    println!("keyboard_key: time {}, mods {:?}, key {:?}, state {:?}",
             time, mods, key, state);
    if state == KeyState::Pressed { return false; }
    println!("Preparing to open the terminal...");
    rustwlc::exec("weston-terminal".to_string(), vec!["weston-terminal".to_string()]);
    // We are definitely able to open programs, they can definitely launch in the host X11 server.
    //rustwlc::exec("emacs".to_string(), vec!["emacs".to_string()]);
    //let output = std::process::Command::new("weston-terminal").output()
    //.unwrap_or_else(|e| { println!("Could not unwrap output!"); panic!("aaaa") });
    let mut child = Command::new("/bin/weston-terminal").spawn()
        .unwrap_or_else(|e| { println!("Error spawning child: {}", e); panic!("1") });

    //let ecode = child.wait().unwrap_or_else(|e| println!("Error unwrapping child"));
    //println!("Output: {}", String::from_utf8(output.stdout).unwrap_or("nope".to_string()));


    false
}

extern fn pointer_button(view: WlcView, button: libc::c_uint, mods: KeyboardModifiers,
                  key: u32, state: ButtonState, point: Point) -> bool {
    println!("pointer_button: key {}, point {}", key, point);

    if state == ButtonState::Pressed {
        view.focus();
    }
    false
}

extern fn pointer_scroll(view: WlcView, button: u32, mods: KeyboardModifiers,
                  axis: ScrollAxis, heights: [u64; 2]) -> bool {
    println!("pointer_scroll");
    false
}

extern fn pointer_motion(view: WlcView, time: u32, point: Point) {
    //println!("Pointer moved {} to {}", time, point);
    // TODO wlc_pointer_set_position
    pointer::set_position(point);
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
