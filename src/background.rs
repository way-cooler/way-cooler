use std::mem::transmute;
use std::os::unix::io::AsRawFd;
use std::time::Duration;
use std::thread::sleep;
use std::io::Write;

use wayland_client::wayland::get_display;
use wayland_client::wayland::compositor::{WlCompositor, WlSurface};
use wayland_client::wayland::shell::{WlShellSurface, WlShell};
use wayland_client::wayland::shm::{WlBuffer, WlShm, WlShmFormat};

use rustwlc::WlcOutput;
use tempfile;

use byteorder::{NativeEndian, WriteBytesExt};

wayland_env!(WaylandEnv,
                compositor: WlCompositor,
                shell: WlShell,
                shm: WlShm
);

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
/// Holds the bytes to represent a colored background.
/// To be written into a wayland surface.
pub struct Color(pub [u8; 4]);

impl Color {
    /// Generate a new color out of a u32.
    /// E.G: 0xFFFFFFFF
    pub fn from_u32(color: u32) -> Self {
        unsafe { Color(transmute(color)) }
    }
}

pub fn generate_solid_background(color: Color, output: WlcOutput) {
    // Get shortcuts to the globals.
    let (display, iter) = get_display()
        .expect("Unable to connect to a wayland compositor");
    let (env, mut evt_iter) = WaylandEnv::init(display, iter);
    let compositor = env.compositor.as_ref().map(|o| &o.0).unwrap();
    let shell = env.shell.as_ref().map(|o| &o.0).unwrap();
    let shm = env.shm.as_ref().map(|o| &o.0).unwrap();

    // Create the surface we are going to write into
    let surface = compositor.create_surface();
    let shell_surface = shell.get_shell_surface(&surface);
    let mut tmp = tempfile::tempfile().ok().expect("Unable to create a tempfile.");

    // Calculate how big the buffer needs to be from the output resolution
    let resolution = output.get_resolution().clone();
    let (width, height) = (resolution.w as i32, resolution.h as i32);
    let size = (resolution.w * resolution.h) as i32;

    // Write in the color coding to the surface
    for _ in 0..size {
        unsafe {
            tmp.write_u32::<NativeEndian>(transmute(color.0))
                .expect("Could not write to file")
        }
    }
    tmp.flush()
        .expect("Could not flush buffer");

    // Create the buffer that is mem-mapped to the temp file descriptor
    let pool = shm.create_pool(tmp.as_raw_fd(), size);
    let buffer = pool.create_buffer(0, width, height, width, WlShmFormat::Argb8888);
    // Tell Way Cooler not to set put this in the tree, treat as background
    shell_surface.set_class("Background".into());

    // Attach the buffer to the surface
    surface.attach(Some(&buffer), 0, 0);
    surface.set_buffer_scale(4);
    surface.commit();
    evt_iter.sync_roundtrip().unwrap();

    main_background_loop(compositor, shell, shm, surface, shell_surface, buffer);
}


/// Main loop for rendering backgrounds.
/// Need to keep the surface alive, and update it if the
/// user wants to change the background.
#[allow(unused_variables)]
fn main_background_loop(compositor: &WlCompositor, shell: &WlShell, shm: &WlShm,
                        surface: WlSurface, shell_surface: WlShellSurface,
                        buffer: WlBuffer) {
    println!("Entering main loop");
    // For now just loop and do nothing
    // Eventually need to query the background state and update
    loop {sleep(Duration::new(1,0));}
}
