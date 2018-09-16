//! Awesome compatibility modules

#![cfg_attr(test, deny(bad_style,
       const_err,
       dead_code,
       improper_ctypes,
       legacy_directory_ownership,
       non_shorthand_field_patterns,
       no_mangle_generic_items,
       overflowing_literals,
       path_statements ,
       patterns_in_fns_without_body,
       plugin_as_library,
       private_in_public,
       private_no_mangle_fns,
       private_no_mangle_statics,
       safe_extern_statics,
       unconditional_recursion,
       unions_with_drop_fields,
       unused,
       unused_allocation,
       unused_comparisons,
       unused_parens,
       while_true))]

// Allowed by default
#![cfg_attr(test, deny(missing_docs,
       trivial_numeric_casts,
       unused_extern_crates,
       unused_import_braces,
       unused_qualifications,
       unused_results))]

#[macro_use]
extern crate bitflags;
extern crate cairo;
extern crate cairo_sys;
extern crate env_logger;
extern crate exec;
extern crate getopts;
extern crate gdk_pixbuf;
extern crate glib;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate nix;
extern crate libc;
extern crate rlua;
extern crate tempfile;
extern crate byteorder;
extern crate xcb;
#[macro_use]
extern crate wayland_client;
extern crate wayland_sys;

// TODO remove
extern crate wlroots;

#[macro_use]
mod macros;
mod objects;
mod common;
mod wayland_obj;
mod awesome;
mod keygrabber;
mod mousegrabber;
mod root;
mod lua;
mod wayland_protocols;

use std::{env, mem, path::PathBuf, process::exit, io::{self, Write}};

use exec::Command;
use rlua::{LightUserData, Lua, Table};
use log::Level;
use nix::sys::signal::{self, SaFlags, SigAction, SigHandler, SigSet};
use xcb::{xkb, Connection};
use wayland_client::{Display, GlobalManager, EventQueue, GlobalError};
use wayland_client::protocol::{wl_compositor, wl_shm, wl_output, wl_display::RequestsTrait};
use wayland_client::sys::client::wl_display;

use self::lua::{LUA, NEXT_LUA};
use self::objects::key::Key;
use self::common::{object::Object, signal::*};
use self::root::ROOT_KEYS_HANDLE;
use wayland_protocols::xdg_shell::xdg_wm_base;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const GIT_VERSION: &'static str = include_str!(concat!(env!("OUT_DIR"), "/git-version.txt"));
pub const GLOBAL_SIGNALS: &'static str = "__awesome_global_signals";
pub const XCB_CONNECTION_HANDLE: &'static str = "__xcb_connection";

/// The state passed into C to store it during the glib loop.
///
/// It's passed back to us when Awesome needs a refresh so we can
/// construct any Wayland objects.
#[repr(C)]
struct WaylandState {
    pub display: Display,
    pub event_queue: EventQueue
}

/// Called from `wayland_glib_interface.c` after every call back into the
/// wayland event loop.
///
/// This restarts the Lua thread if there is a new one pending
#[no_mangle]
pub extern "C" fn awesome_refresh(wayland_state: *mut libc::c_void) {
    // NOTE
    // This is safe because it's way back up the stack where we can't access it.
    //
    // The moment that stack is accessible this pointer will be lost.
    //
    // The only way it's unsafe is if we destructure `WaylandState`,
    // which we can't do because it's borrowed.
    let wayland_state = unsafe { &mut *(wayland_state as *mut WaylandState) };
    NEXT_LUA.with(|new_lua_check| {
        if new_lua_check.get() {
            new_lua_check.set(false);
            let awesome = env::args().next().unwrap();
            let args: Vec<_> = env::args().skip(1).collect();
            let err = Command::new(awesome)
                .args(args.as_slice())
                .exec();
            error!("error: {:?}", err);
            panic!("Could not restart Awesome");
        }
    });
}

fn main() {
    let mut opts = getopts::Options::new();
    let matches = match opts.optflag("", "version", "show version information")
                            .parse(env::args().skip(1)) {
        Ok(m) => m,
        Err(f) => {
            eprintln!("{}", f.to_string());
            exit(1);
        }
    };
    if matches.opt_present("version") {
        if !GIT_VERSION.is_empty() {
            println!("Way Cooler {} @ {}", VERSION, GIT_VERSION);
        } else {
            println!("Way Cooler {}", VERSION);
        }
        return
    }
    init_logs();
    let sig_action = SigAction::new(SigHandler::Handler(sig_handle),
                                    SaFlags::empty(),
                                    SigSet::empty());
    unsafe {
        let _ = signal::sigaction(signal::SIGINT, &sig_action)
                        .expect("Could not set SIGINT catcher");
    }
    lua::init_awesome_libraries();
    init_wayland();
    lua::run_awesome();
}

fn init_wayland() {
    let (display, mut event_queue) = match Display::connect_to_env() {
        Ok(res) => res,
        Err(err) => {
            error!("Could not connect to Wayland server. Is it running?");
            error!("{:?}", err);
            exit(1);
        }
    };
    let globals = GlobalManager::new_with_cb(
        display.get_registry().unwrap(),
        global_filter!(
            [
                wl_output::WlOutput,
                wayland_obj::WL_OUTPUT_VERSION,
                wayland_obj::Output::new
            ],
            [
                wl_compositor::WlCompositor,
                wayland_obj::WL_COMPOSITOR_VERSION,
                wayland_obj::wl_compositor_init
            ],
            [
                wl_shm::WlShm,
                wayland_obj::WL_SHM_VERSION,
                wayland_obj::wl_shm_init
            ]
        ),
    );
    event_queue.sync_roundtrip().unwrap();
    let xwm_base_proxy = match globals.instantiate_exact::
    <xdg_wm_base::XdgWmBase>(wayland_obj::XDG_WM_BASE_VERSION) {
        Err(GlobalError::Missing) => {
            error!("Missing xdg_wm_base global (version {})",
                   wayland_obj::XDG_WM_BASE_VERSION);
            error!("Your compositor doesn't support the xdg shell protocol");
            error!("This protocol is necessary for Awesome to function");
            exit(1);
        },
        Err(GlobalError::VersionTooLow(version)) => {
            error!("Got xdg_wm_base version {}, expected version {}",
                   version, wayland_obj::XDG_WM_BASE_VERSION);
            error!("Your compositor doesn't support version {} \
                    of the xdg shell protocol", wayland_obj::XDG_WM_BASE_VERSION);
            error!("Ensure your compositor is up to date");
            exit(1);
        },
        Ok(proxy) => Ok(proxy)
    };
    wayland_obj::xdg_shell_init(xwm_base_proxy, ());
    event_queue.sync_roundtrip().unwrap();
    let mut wayland_state = WaylandState { display, event_queue };
    let display_ptr = wayland_state.display.c_ptr() as *mut wl_display;
    unsafe {
        #[link(name = "wayland_glib_interface", kind = "static")]
        extern "C" {
            fn wayland_glib_interface_init(display: *mut wl_display,
                                           wayland_state: *mut libc::c_void);
        }
        wayland_glib_interface_init(display_ptr,
                                    &mut wayland_state as *mut _ as _);
        ::std::mem::forget(wayland_state);
        ::std::mem::forget(globals);
    }
}

fn setup_awesome_path(lua: &Lua) -> rlua::Result<()> {
    let globals = lua.globals();
    let package: Table = globals.get("package")?;
    let mut path = package.get::<_, String>("path")?;
    let mut cpath = package.get::<_, String>("cpath")?;

    for mut xdg_data_path in
        env::var("XDG_DATA_DIRS").unwrap_or("/usr/local/share:/usr/share".into())
                                 .split(':')
                                 .map(PathBuf::from)
    {
        xdg_data_path.push("awesome/lib");
        path.push_str(&format!(";{0}/?.lua;{0}/?/init.lua",
                               xdg_data_path.as_os_str().to_string_lossy()));
        cpath.push_str(&format!(";{}/?.so", xdg_data_path.into_os_string().to_string_lossy()));
    }

    for mut xdg_config_path in env::var("XDG_CONFIG_DIRS").unwrap_or("/etc/xdg".into())
                                                          .split(':')
                                                          .map(PathBuf::from)
    {
        xdg_config_path.push("awesome");
        cpath.push_str(&format!(";{}/?.so",
                                xdg_config_path.into_os_string().to_string_lossy()));
    }

    package.set("path", path)?;
    package.set("cpath", cpath)?;

    Ok(())
}

/// Set up global signals value
///
/// We need to store this in Lua, because this make it safer to use.
fn setup_global_signals(lua: &Lua) -> rlua::Result<()> {
    lua.set_named_registry_value(GLOBAL_SIGNALS, lua.create_table()?)
}

/// Sets up the xcb connection and stores it in Lua (for us to access it later)
fn setup_xcb_connection(lua: &Lua) -> rlua::Result<()> {
    let con = match Connection::connect(None) {
        Err(err) => {
            error!("Way Cooler requires XWayland in order to function");
            error!("However, xcb could not connect to it. Is it running?");
            error!("{:?}", err);
            panic!("Could not connect to XWayland instance");
        }
        Ok(con) => con.0
    };
    // Tell xcb we are using the xkb extension
    match xkb::use_extension(&con, 1, 0).get_reply() {
        Ok(r) => {
            if !r.supported() {
                panic!("xkb-1.0 is not supported");
            }
        }
        Err(err) => {
            panic!("Could not get xkb extension supported version {:?}", err);
        }
    }
    lua.set_named_registry_value(XCB_CONNECTION_HANDLE,
                                  LightUserData(con.get_raw_conn() as _))?;
    mem::forget(con);
    Ok(())
}

/// Formats the log strings properly
fn log_format(buf: &mut env_logger::fmt::Formatter, record: &log::Record) -> Result<(), io::Error> {
    let color = match record.level() {
        Level::Info => "",
        Level::Trace => "\x1B[37m",
        Level::Debug => "\x1B[44m",
        Level::Warn => "\x1B[33m",
        Level::Error => "\x1B[31m"
    };
    let mut module_path = record.module_path().unwrap_or("?");
    if let Some(index) = module_path.find("way_cooler::") {
        let index = index + "way_cooler::".len();
        module_path = &module_path[index..];
    }
    writeln!(buf, "{} {} [{}] \x1B[37m{}:{}\x1B[0m{0} {} \x1B[0m",
            color,
            record.level(),
            module_path,
            record.file().unwrap_or("?"),
            record.line().unwrap_or(0),
            record.args())
}

fn init_logs() {
    let env = env_logger::Env::default()
        .filter_or("WAY_COOLER_LOG", "trace");
    let _ = env_logger::Builder::from_env(env)
                .format(log_format)
                .init();
    info!("Logger initialized");
}

/// Handler for SIGINT signal
extern "C" fn sig_handle(_: nix::libc::c_int) {
    lua::terminate();
    exit(130);
}
