extern crate rustwlc;

use rustwlc::types;
use rustwlc::types::*;
use rustwlc::handle::{WlcView, WlcOutput};
use rustwlc::interface::*;
use rustwlc::input::{pointer, keyboard};

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
            created: Some(output_created),
            destroyed: Some(output_destroyed),
            focus: Some(output_focus),
            resolution: Some(output_resolution),

            render: OutputRenderInterface {
                pre: Some(output_render_pre),
                post: Some(output_render_post)
            }
        },
        view: ViewInterface {
            created: Some(view_created),
            destroyed: Some(view_destroyed),
            focus: Some(view_focus),
            move_to_output: Some(view_move_to_output),
            request: RequestInterface {
                geometry: Some(view_request_geometry),
                state: Some(view_request_state),
                move_: Some(view_request_move),
                resize: Some(view_request_resize),
                render: ViewRenderInterface {
                    pre: Some(view_request_render_pre),
                    post: Some(view_request_render_post)
                }
            }
        },
        keyboard: KeyboardInterface {
            key: Some(keyboard_key)
        },
        pointer: PointerInterface {
            button: Some(pointer_button),
            scroll: Some(pointer_scroll),
            motion: Some(pointer_motion)
        },
        touch: TouchInterface {
            touch: Some(touch_touch)
        },
        compositor: CompositorInterface {
            ready: Some(compositor_ready)
        },
        input: InputInterface {
            created: Some(input_created),
            destroyed: Some(input_destroyed)
        }
    };

    if !rustwlc::init(interface) {
        panic!("Unable to initialize wlc!");
    }
    rustwlc::run_wlc();
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
    println!("Rendering {} views for {} (resolution = {:?})", views.len(), output.get_name(), resolution);

    if views.len() == 0 { println!("Didn't find any views to render :/"); }

    for view in views {
        println!("Setting {:?}", view);
        println!("\tIts type: {}", view.get_type());
        println!("\tIts output: {}", view.get_output().get_name());
        println!("\tIts geometry: {:?}", view.get_geometry());
        println!("\tIts state: {}", view.get_state());
        println!("\tIts parent: {:?}", view.get_parent());
        println!("\tIts title: {}", view.get_title());
        // get_class doesn't work but maybe it's not supposed to
        //println!("\tIts class: {}", view.get_class());
        view.set_geometry(0,
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
    println!("view_created: {:?}", view);
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
    current.set_state(ViewState::Activated, focused);
}

extern fn view_move_to_output(current: WlcView, q1: WlcView, q2: WlcView) {
    println!("view_move_to_output: {:?}, {:?}, {:?}", current, q1, q2);
}

extern fn view_request_geometry(view: WlcView, geometry: &Geometry) {
    println!("view_request_geometry: {:?} wants {:?}", view, geometry);
    view.set_geometry(0, geometry);
    render_output(view.get_output());
}

extern fn view_request_state(view: WlcView, state: ViewState, handled: bool) {
    view.set_state(state, handled);
}

extern fn view_request_move(view: WlcView, dest: &Point) {
    //println!("view_request_move: to {}, start interactive mode.", *dest);
}

extern fn view_request_resize(view: WlcView, edge: ResizeEdge, location: &Point) {
    println!("view_request_resize: size {:?}, to {}, start interactive mode.",
             edge, *location);
}

extern fn view_request_render_pre(view: WlcView) {
}

extern fn view_request_render_post(view: WlcView) {
}

extern fn keyboard_key(view: WlcView, time: u32, mods_ptr: &KeyboardModifiers,
                       key: u32, state: KeyState) -> bool {
    use std::process::{Command};
    println!("keyboard_key: time {}, mods {:?}, key {:?}, state {:?}",
             time, &*mods_ptr, key, state);
    if state == KeyState::Pressed { return false; }
    if key == 67 {
        println!("Preparing to open the terminal...");
        let mut child = Command::new("/bin/weston-terminal").spawn()
            .unwrap_or_else(|e| { println!("Error spawning child: {}", e); panic!("1") });
    }
    false
}

extern fn pointer_button(view: WlcView, button: u32, mods_ptr: &KeyboardModifiers,
                         key: u32, state: ButtonState, point_ptr: &Point) -> bool {
    println!("pointer_button: pressed {} at {}", key, *point_ptr);
    if state == ButtonState::Pressed {
        view.focus();
    }
    false
}

extern fn pointer_scroll(view: WlcView, button: u32, mods_ptr: &KeyboardModifiers,
                  axis: ScrollAxis, heights: [u64; 2]) -> bool {
    println!("pointer_scroll");
    false
}

extern fn pointer_motion(view: WlcView, time: u32, point: &Point) {
    pointer::set_position(point);
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
