//! Wrappers around Wayland objects

mod output;
mod xdg_shell;
mod wl_compositor;

pub use self::output::Output;
pub use self::xdg_shell::xdg_shell_init;
pub use self::wl_compositor::{wl_compositor_init, create_surface};
