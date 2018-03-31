//! Main module of way-cooler

#[macro_use]
extern crate lazy_static;
extern crate bitflags;
extern crate dbus_macros;
extern crate getopts;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate rlua;
extern crate rustc_serialize;
extern crate json_macro;
extern crate nix;
extern crate petgraph;
extern crate uuid;
extern crate dbus;
extern crate cairo;
extern crate cairo_sys;
extern crate gdk_pixbuf;
extern crate glib;
#[macro_use]
extern crate wayland_sys;
extern crate wayland_server;
extern crate xcb;

#[macro_use]
mod macros;
mod convert;
mod callbacks;
mod keys;
mod lua;
mod registry;
mod commands;
mod ipc;
mod layout;
mod render;
mod wayland;
mod modes;
mod awesome;

use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use getopts::Options;
use log::LogLevel;
use nix::sys::signal::{self, SigHandler, SigSet, SigAction, SaFlags};

use rustwlc::types::LogType;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const GIT_VERSION: &'static str = include_str!(concat!(env!("OUT_DIR"), "/git-version.txt"));
const DRIVER_MOD_PATH: &'static str = "/proc/modules";
const DEVICE_MOD_PATH: &'static str = "/sys/firmware/devicetree/base/model";

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
    let color = match record.level() {
        LogLevel::Info => "",
        LogLevel::Trace => "\x1B[37m",
        LogLevel::Debug => "\x1B[44m",
        LogLevel::Warn =>  "\x1B[33m",
        LogLevel::Error => "\x1B[31m",
    };
    let location = record.location();
    let file = location.file();
    let line = location.line();
    let mut module_path = location.module_path();
    if let Some(index) = module_path.find("way_cooler::") {
        let index = index + "way_cooler::".len();
        module_path = &module_path[index..];
    }
    format!("{} {} [{}] \x1B[37m{}:{}\x1B[0m{0} {} \x1B[0m",
            color, record.level(), module_path, file, line, record.args())
}

/// Ensures that the environment is set up correctly. E.g:
/// * XDG_RUNTIME_DIR is set to a directory that exists.
fn ensure_good_env() {
    use std::env::VarError;
    match env::var("XDG_RUNTIME_DIR") {
        Ok(_) => {/* Do nothing, logged in `log_environment` */},
        Err(VarError::NotUnicode(string)) => {
            error!("The value set for XDG_RUNTIME_DIR ({:?}) \
                    is not valid unicode!",
                   string);
            std::process::exit(1);
        },
        Err(VarError::NotPresent) => {
            error!("XDG_RUNTIME_DIR is not set!");
            std::process::exit(1);
        }
    }
}

/// Checks the loaded modules, and reports any problematic proprietary ones
fn detect_proprietary() {
    // If DISPLAY is present, we are running embedded
    if env::var("DISPLAY").is_ok() {
        return
    }
    match File::open(Path::new(DRIVER_MOD_PATH)) {
        Ok(file) => {
            let reader = BufReader::new(&file);
            for line in reader.lines() {
                if let Ok(line) = line {
                    if line.contains("nvidia") {
                        error!("Warning: Proprietary nvidia graphics drivers are installed, \
                               but they are not compatible with Wayland. \
                               Consider using nouveau drivers for Wayland.");
                        break;
                    } else if line.contains("fglrx") {
                        error!("Warning: Proprietary AMD graphics drivers are installed, \
                                but they are not compatible with Wayland. \
                               Consider using radeon drivers for Waylad.");
                        break;
                    }
                }
            }
        },
        Err(err) => {
            warn!("Could not read proprietary modules at \"{}\", because: {:#?}",
                  DRIVER_MOD_PATH, err);
            warn!("If you are running proprietary AMD or Nvidia graphics drivers, \
                   Way Cooler may not work for you");
        }
    }
}

/// Checks the loaded modules to ensure vc4 is loaded if we are running on a raspi.
fn detect_raspi() {
    use std::fmt::Debug;
    /// Prints a debug line from the given info.
    /// This should be used to "fail open" so that we keep trying but at least
    /// warn the user that what we are about to do probably won't work.
    fn fail_open<T: Debug>(path: &str, err: T) {
        warn!("Could not read file \"{}\" because {:#?}",
              path, err)
    }
    let raspi = match File::open(Path::new(DEVICE_MOD_PATH)) {
        Ok(f) => {
            let reader = BufReader::new(&f);
            let mut raspi = false;
            for line in reader.lines() {
                if let Ok(line) = line {
                    if line.contains("Raspberry Pi") {
                        raspi = true;
                        break;
                    }
                }
            }
            raspi
        },
        Err(_) => {
            return;
        }
    };
    let vc4 = match File::open(Path::new(DRIVER_MOD_PATH)) {
        Ok(f) => {
            let reader = BufReader::new(&f);
            let mut vc4 = false;
            for line in reader.lines() {
                if let Ok(line) = line {
                    if line.contains("vc4") {
                        vc4 = true;
                        break;
                    }
                }
            }
            vc4
        },
        Err(err) => {
            fail_open(DRIVER_MOD_PATH, err);
            return;
        }
    };
    if !vc4 && raspi {
        error!("You are running on a Raspberry Pi, but the vc4 module is not loaded!");
        error!("Set 'dtoverlay=vc4-kms-v3d' in /boot/config.txt and reboot!");
    }
}

#[inline(always)]
/// Determines if we should build with debug symbols.
pub fn debug_enabled() -> bool {
    cfg!(not(disable_debug))
}

/// Initializes the logging system.
pub fn init_logs() {
    let mut builder = env_logger::LogBuilder::new();
    builder.format(log_format);
    builder.filter(None, log::LogLevelFilter::Trace);
    if env::var("WAY_COOLER_LOG").is_ok() {
        builder.parse(&env::var("WAY_COOLER_LOG")
                      .expect("WAY_COOLER_LOG not defined"));
    }
    builder.init().expect("Unable to initialize logging!");
    info!("Logger initialized, setting wlc handlers.");
}

fn log_environment() {
    for (key, value) in env::vars() {
        info!("{}: {}", key, value);
    }
}

/// Handler for signals
extern "C" fn sig_handle(_: nix::libc::c_int) {
    rustwlc::terminate();
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optflag("", "version", "show version information");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            println!("{}", f.to_string());
            std::process::exit(1);
        }
    };
    if matches.opt_present("version") {
        if !GIT_VERSION.is_empty() {
            println!("Way Cooler {} @ {}", VERSION, GIT_VERSION);
        } else {
            println!("Way Cooler {}", VERSION);
        }
        println!("Way Cooler IPC version {}", ipc::VERSION);
        return
    }
    println!("Launching way-cooler...");

    let sig_action = SigAction::new(SigHandler::Handler(sig_handle), SaFlags::empty(),
                                    SigSet::empty());
    unsafe {signal::sigaction(signal::SIGINT, &sig_action).unwrap() };

    init_logs();
    log_environment();
    detect_proprietary();
    detect_raspi();
    ensure_good_env();
    // This prepares wlc, doesn't run main loop until run_wlc is called
    let run_wlc = rustwlc::init()
        .expect("Unable to initialize wlc!");
    wayland::init_wayland_protocols();
    rustwlc::log_set_rust_handler(log_handler);
    callbacks::init();
    commands::init();
    registry::init();
    ipc::init();

    info!("Running wlc...");
    run_wlc();
}
