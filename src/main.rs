//! Main module of way-cooler

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate bitflags;

extern crate rustwlc;

#[macro_use]
extern crate log;
extern crate env_logger;

#[macro_use]
extern crate hlua;

use std::env;

use rustwlc::types::LogType;

#[macro_use] // As it happens, it's important to declare the macros first.
mod macros;

mod callbacks;
mod keys;

mod lua;
mod registry;
mod convert;

mod layout;
mod compositor;

/// Callback to route wlc logs into env_logger
fn log_handler(level: LogType, message: &str) {
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
    debug!("Launching way-cooler...");

    // Initialize callbacks
    callbacks::init();

    // Prepare log builder
    let mut builder = env_logger::LogBuilder::new();
    builder.format(log_format);
    builder.filter(None, log::LogLevelFilter::Trace);
    if env::var("WAY_COOLER_LOG").is_ok() {
        builder.parse(&env::var("WAY_COOLER_LOG").expect("Asserted unwrap!"));
    }
    builder.init().expect("Unable to initialize logging!");
    info!("Logger initialized, setting wlc handler.");

    // Handle wlc logs
    rustwlc::log_set_rust_handler(log_handler);

    // Prepare to launch wlc
    let run_wlc = rustwlc::init2().expect("Unable to initialize wlc!");

    // Hand control over to wlc's event loop
    run_wlc();
}
