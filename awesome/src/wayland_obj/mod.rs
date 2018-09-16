//! Wrappers around Wayland objects

mod output;
mod xdg_shell;
mod wl_compositor;

pub use self::output::{WL_OUTPUT_VERSION, Output};
pub use self::xdg_shell::{XDG_WM_BASE_VERSION, XdgToplevel, xdg_shell_init, create_xdg_toplevel};
pub use self::wl_compositor::{WL_COMPOSITOR_VERSION, WL_SHM_VERSION, wl_compositor_init, wl_shm_init, create_surface, create_buffer};
