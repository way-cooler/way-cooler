//! Main module in way-cooler

extern crate rustwlc;

#[macro_use]
extern crate lazy_static;

use rustwlc::interface::WlcInterface;

mod registry;
mod keys;
mod callbacks;

fn main() {
    let interface: WlcInterface = WlcInterface::new()
        .output_created(callbacks::output_created)
        .output_destroyed(callbacks::output_destroyed)
        .output_focus(callbacks::output_focus)
        .output_resolution(callbacks::output_resolution)
        .output_render_pre(callbacks::output_render_pre)
        .output_render_post(callbacks::output_render_post)
        .view_created(callbacks::view_created)
        .view_destroyed(callbacks::view_destroyed)
        .view_focus(callbacks::view_focus)
        .view_move_to_output(callbacks::view_move_to_output)
        .view_request_geometry(callbacks::view_request_geometry)
        .view_request_state(callbacks::view_request_state)
        .view_request_move(callbacks::view_request_move)
        .view_request_resize(callbacks::view_request_resize)
        .keyboard_key(callbacks::keyboard_key)
        .pointer_button(callbacks::pointer_button)
        .pointer_scroll(callbacks::pointer_scroll)
        .pointer_motion(callbacks::pointer_motion)
        .touch_touch(callbacks::touch)
        .compositor_ready(callbacks::compositor_ready);
        //.input_created(input_created)
        //.input_destroyed(input_destroyed);

    rustwlc::log_set_default_handler();

    let run_wlc = rustwlc::init(interface).expect("Unable to initialize wlc!");
    run_wlc();
}
