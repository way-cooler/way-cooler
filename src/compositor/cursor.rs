use wlroots::CursorHandler;
#[derive(Debug, Default)]
pub struct CursorManager;

impl CursorHandler for CursorManager {}

impl CursorManager {
    pub fn new() -> Self {
        CursorManager::default()
    }
}
