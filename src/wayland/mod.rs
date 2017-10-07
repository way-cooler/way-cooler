mod gamma_control;
mod desktop_shell;

/// Initializes the appropriate handlers for each wayland protocol
/// that Way Cooler supports.
pub fn init_wayland_protocols() {
    info!("Initializing wayland protocols");
    gamma_control::init();
    desktop_shell::init();
}
