//! Main module of way-cooler ipc.

mod layout;

pub const VERSION: u32 = 1;

pub fn init() {
    let layout_mapping = layout::Layout::new();
    layout_mapping.run("org.way_cooler");
}
