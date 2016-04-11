//! Main module in way-cooler

extern crate rustwlc;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate hlua;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate libc;

use rustwlc::interface::WlcInterface;
use rustwlc::types::LogType;

use std::env;

mod registry;
mod keys;
mod callbacks;
mod lua;

/// Callback to route wlc logs into env_logger
extern "C" fn log_handler(level: LogType, message_ptr: *const libc::c_char) {
    let message = unsafe { rustwlc::pointer_to_string(message_ptr) };
    match level {
        LogType::Info => info!("wlc: {}", message),
        LogType::Warn => warn!("wlc: {}", message),
        LogType::Error => error!("wlc: {}", message),
        LogType::Wayland => info!("wayland: {}", message)
    }
}

/// Formats the log strings properly
fn log_format(record: &log::LogRecord) -> String {
    format!("{} [{}] {}", record.level(), record.location().module_path(),
            record.args())
}

fn main() {
    let interface: WlcInterface = WlcInterface::new()
        .output_created(callbacks::output_created)
        .output_destroyed(callbacks::output_destroyed)
        .output_focus(callbacks::output_focus)
        .output_resolution(callbacks::output_resolution)
        //.output_render_pre(callbacks::output_render_pre)
        //.output_render_post(callbacks::output_render_post)
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
        //.touch_touch(callbacks::touch)
        .compositor_ready(callbacks::compositor_ready);
        //.input_created(input_created)
        //.input_destroyed(input_destroyed);

    let mut builder = env_logger::LogBuilder::new();
    builder.format(log_format);
    builder.filter(None, log::LogLevelFilter::Trace);
    if env::var("WAY_COOLER_LOG").is_ok() {
        builder.parse(&env::var("WAY_COOLER_LOG").unwrap());
    }
    builder.init().unwrap();
    info!("Logger initialized, setting wlc handler.");
    rustwlc::log_set_handler(log_handler);

    let run_wlc = rustwlc::init(interface).expect("Unable to initialize wlc!");

    info!("Started logger");

    run_wlc();
}
