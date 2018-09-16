//! Wrappers around Wayland objects

mod output;
mod xdg_shell;
mod wl_compositor;

pub use self::output::Output;
pub use self::xdg_shell::{XdgToplevel, xdg_shell_init, create_xdg_toplevel};
pub use self::wl_compositor::{wl_compositor_init, create_surface};
