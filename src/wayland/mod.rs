pub mod gamma_control;

/// Initializes the appropriate handlers for each wayland protocol
/// that Way Cooler supports.
pub fn init_wayland_protocols() {
    info!("Initializing wayland protocols");
    gamma_control::init();
}
