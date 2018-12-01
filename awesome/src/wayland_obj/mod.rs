//! Wrappers around Wayland objects

mod output;
mod wl_compositor;
mod wl_shm;
mod xdg_shell;

pub use self::{output::{Output, WL_OUTPUT_VERSION},
               wl_compositor::{create_surface, wl_compositor_init, WL_COMPOSITOR_VERSION},
               wl_shm::{create_buffer, wl_shm_init, WL_SHM_VERSION},
               xdg_shell::{create_xdg_toplevel, xdg_shell_init, XdgToplevel, XDG_WM_BASE_VERSION}};
