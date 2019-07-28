//! Awesome compatibility modules

#![cfg_attr(
    test,
    deny(
        bad_style,
        const_err,
        dead_code,
        improper_ctypes,
        legacy_directory_ownership,
        non_shorthand_field_patterns,
        no_mangle_generic_items,
        overflowing_literals,
        path_statements,
        patterns_in_fns_without_body,
        plugin_as_library,
        private_in_public,
        safe_extern_statics,
        unconditional_recursion,
        unions_with_drop_fields,
        unused,
        unused_allocation,
        unused_comparisons,
        unused_parens,
        while_true
    )
)]
// Allowed by default
#![cfg_attr(test, deny(missing_docs, trivial_numeric_casts))]

#[macro_use]
extern crate log;

mod area;
mod awesome;
mod common;
mod dbus;
mod keygrabber;
mod lua;
mod mousegrabber;
mod objects;
mod root;
mod wayland;

use std::{
    env,
    io::{self, Write},
    os::unix::io::RawFd,
    path::PathBuf,
    process::exit
};

use {
    clap::{App, Arg},
    exec::Command,
    log::Level,
    nix::sys::signal::{self, SaFlags, SigAction, SigHandler, SigSet},
    rlua::Table,
    wayland_client::{
        global_filter,
        protocol::{wl_compositor, wl_output, wl_shm},
        sys::client::wl_display,
        ConnectError, Display, EventQueue, GlobalManager
    },
    xcb::xkb
};

use crate::lua::{SyntaxCheckError, LUA, NEXT_LUA};

const GIT_VERSION: &'static str =
    include_str!(concat!(env!("OUT_DIR"), "/git-version.txt"));
pub const GLOBAL_SIGNALS: &'static str = "__awesome_global_signals";

thread_local! {
    static XCB_CONNECTION: xcb::Connection =
        match xcb::Connection::connect(None) {
            Err(_) => {
                fail("Way Cooler requires XWayland in order to function. \
                      However, xcb could not connect to it. Is it running?")
            },
            Ok(con) => con.0
        };
}

/// The state passed into C to store it during the glib loop.
///
/// It's passed back to us when Awesome needs a refresh so we can
/// construct any Wayland objects.
#[repr(C)]
struct WaylandState {
    pub display: Display,
    pub event_queue: EventQueue
}

#[link(name = "wayland_glib_interface", kind = "static")]
extern "C" {
    pub fn wayland_glib_interface_init(
        display: *mut wl_display,
        session_fd: RawFd,
        system_fd: RawFd,
        wayland_state: *mut libc::c_void
    );
    pub fn remove_dbus_from_glib();
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
    let _wayland_state = unsafe { &mut *(wayland_state as *mut WaylandState) };
    NEXT_LUA.with(|new_lua_check| {
        if new_lua_check.get() {
            new_lua_check.set(false);
            let mut args = env::args();
            let binary = args.next().unwrap();
            let args = args.collect::<Vec<_>>();
            let err = Command::new(binary).args(args.as_slice()).exec();
            error!("exec error: {:?}", err);
            fail("Could not restart Awesome");
        }
    });
}

fn main() {
    init_logs();

    let sig_action = SigAction::new(
        SigHandler::Handler(sig_handle),
        SaFlags::empty(),
        SigSet::empty()
    );
    unsafe {
        signal::sigaction(signal::SIGINT, &sig_action)
            .expect("Could not set SIGINT catcher");
    }

    let version = if !GIT_VERSION.is_empty() {
        concat!(
            "Awesome ",
            env!("CARGO_PKG_VERSION"),
            " @ ",
            include_str!(concat!(env!("OUT_DIR"), "/git-version.txt"))
        )
    } else {
        concat!("Awesome ", env!("CARGO_PKG_VERSION"))
    };
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(version)
        .version_short("v")
        .author(clap::crate_authors!("\n"))
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("configuration file to use")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("lua lib search")
                .long("search")
                .value_name("DIR")
                .help("add a directory to the library search path")
                .takes_value(true)
                .multiple(true)
        )
        .arg(
            Arg::with_name("lua syntax check")
                .short("k")
                .long("check")
                .help("check configure file syntax")
        )
        .arg(
            Arg::with_name("client transparency")
                .short("a")
                .long("no-argb")
                .help("disable client transparency support")
        )
        .arg(
            Arg::with_name("replace wm")
                .short("r")
                .long("replace")
                .help("replace an existing window manager")
        )
        .get_matches();
    if matches.is_present("client transparency") {
        unimplemented!()
    }
    if matches.is_present("replace wm") {
        unimplemented!()
    }
    if matches.is_present("lua syntax check") {
        let config = matches.value_of("config");

        match lua::syntax_check(config) {
            Err(SyntaxCheckError::IoError(err)) => {
                error!("{}", err);
                fail("Could not read configuration files");
            },
            Err(SyntaxCheckError::LuaError(lua_error)) => {
                error!("{}", lua_error);
                println!("✘ Configuration file syntax error.");
                exit(1);
            },
            Ok(_) => {
                println!("✔ Configuration file syntax OK.");
                exit(0)
            }
        }
    }

    setup_xcb_connection();

    let lib_paths = matches
        .values_of("lua lib search")
        .unwrap_or_default()
        .collect::<Vec<_>>();
    lua::init_awesome_libraries(&lib_paths);

    let (mut wayland_state, _globals) = init_wayland();

    let (session_fd, system_fd) =
        dbus::connect().expect("Could not set up dbus connection");

    let display_ptr = wayland_state.display.get_display_ptr();
    // NOTE This is safe because this is the top of the stack, and thus once the
    // WaylandState is popped the program ends.
    unsafe {
        wayland_glib_interface_init(
            display_ptr,
            session_fd,
            system_fd,
            &mut wayland_state as *mut _ as _
        );
    }

    let config = matches.value_of("config");
    lua::run_awesome(&lib_paths, config);
}

fn init_wayland() -> (WaylandState, GlobalManager) {
    let (display, mut event_queue) =
        Display::connect_to_env().unwrap_or_else(|err| match err {
            ConnectError::NoWaylandLib => fail(
                "Could not find Wayland library, is it installed and in PATH?"
            ),
            ConnectError::NoCompositorListening => {
                error!(
                    "WAYLAND_DISPLAY={}",
                    env::var("WAYLAND_DISPLAY").unwrap_or_else(|_| "".into())
                );
                fail("Could not connect to Wayland server. Is it running?");
            },
            ConnectError::InvalidName => {
                fail("Invalid socket name provided in WAYLAND_SOCKET")
            },
            ConnectError::XdgRuntimeDirNotSet => {
                fail("XDG_RUNTIME_DIR must be set")
            },
            ConnectError::InvalidFd => {
                fail("Invalid socket provided in WAYLAND_SOCKET")
            },
        });
    let globals = GlobalManager::new_with_cb(
        &display,
        global_filter!(
            [
                wl_output::WlOutput,
                wayland::WL_OUTPUT_VERSION,
                wayland::WlOutputManager {}
            ],
            [
                wl_compositor::WlCompositor,
                wayland::WL_COMPOSITOR_VERSION,
                wayland::WlCompositorManager {}
            ],
            [
                wl_shm::WlShm,
                wayland::WL_SHM_VERSION,
                wayland::WlShmManager {}
            ]
        )
    );
    event_queue.sync_roundtrip().unwrap();

    wayland::instantiate_global(
        &globals,
        wayland::LAYER_SHELL_VERSION,
        wayland::layer_shell_init,
        "layer shell"
    );
    wayland::instantiate_global(
        &globals,
        wayland::MOUSEGRABBER_VERSION,
        wayland::mousegrabber_init,
        "mousegrabber"
    );

    event_queue.sync_roundtrip().unwrap();
    (
        WaylandState {
            display,
            event_queue
        },
        globals
    )
}

fn setup_awesome_path(
    lua: rlua::Context,
    lib_paths: &[&str]
) -> rlua::Result<()> {
    let globals = lua.globals();
    let package: Table = globals.get("package")?;
    let mut path = package.get::<_, String>("path")?;
    let mut cpath = package.get::<_, String>("cpath")?;

    for lib_path in lib_paths {
        path.push_str(&format!(";{0}/?.lua;{0}/?/init.lua", lib_path));
        cpath.push_str(&format!(";{}/?.so", lib_path));
    }

    for mut xdg_data_path in env::var("XDG_DATA_DIRS")
        .unwrap_or("/usr/local/share:/usr/share".into())
        .split(':')
        .map(PathBuf::from)
    {
        xdg_data_path.push("awesome/lib");
        path.push_str(&format!(
            ";{0}/?.lua;{0}/?/init.lua",
            xdg_data_path.as_os_str().to_string_lossy()
        ));
        cpath.push_str(&format!(
            ";{}/?.so",
            xdg_data_path.into_os_string().to_string_lossy()
        ));
    }

    for mut xdg_config_path in env::var("XDG_CONFIG_DIRS")
        .unwrap_or("/etc/xdg".into())
        .split(':')
        .map(PathBuf::from)
    {
        xdg_config_path.push("awesome");
        cpath.push_str(&format!(
            ";{}/?.so",
            xdg_config_path.into_os_string().to_string_lossy()
        ));
    }

    package.set("path", path)?;
    package.set("cpath", cpath)?;

    Ok(())
}

/// Set up global signals value
///
/// We need to store this in Lua, because this make it safer to use.
fn setup_global_signals(lua: rlua::Context) -> rlua::Result<()> {
    lua.set_named_registry_value(GLOBAL_SIGNALS, lua.create_table()?)
}

/// Sets up the global xcb connection
fn setup_xcb_connection() {
    XCB_CONNECTION.with(|con| {
        // Tell xcb we are using the xkb extension
        match xkb::use_extension(&con, 1, 0).get_reply() {
            Ok(r) => {
                if !r.supported() {
                    fail("xkb-1.0 is not supported")
                }
            },
            Err(err) => {
                fail(&format!(
                    "Could not get xkb extension supported version {:?}",
                    err
                ));
            }
        }
    });
}

/// Formats the log strings properly
fn log_format(
    buf: &mut env_logger::fmt::Formatter,
    record: &log::Record
) -> Result<(), io::Error> {
    let color = match record.level() {
        Level::Info => "",
        Level::Trace => "\x1B[37m",
        Level::Debug => "\x1B[44m",
        Level::Warn => "\x1B[33m",
        Level::Error => "\x1B[31m"
    };
    writeln!(
        buf,
        "{} {} [{}] \x1B[37m{}:{}\x1B[0m{0} {} \x1B[0m",
        color,
        record.level(),
        record.module_path().unwrap_or("?"),
        record.file().unwrap_or("?"),
        record.line().unwrap_or(0),
        record.args()
    )
}

fn init_logs() {
    let env = env_logger::Env::default().filter_or("WAY_COOLER_LOG", "trace");
    env_logger::Builder::from_env(env).format(log_format).init();
    info!("Logger initialized");
}

/// Handler for SIGINT signal
extern "C" fn sig_handle(_: nix::libc::c_int) {
    lua::terminate();
    exit(130);
}

fn fail(msg: &str) -> ! {
    error!("{}", msg);
    exit(1);
}
