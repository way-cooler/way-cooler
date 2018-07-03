use wlroots::OutputLayoutHandler;

#[derive(Debug, Default)]
pub struct OutputLayoutManager;

impl OutputLayoutHandler for OutputLayoutManager {}

impl OutputLayoutManager {
    pub fn new() -> Self {
        OutputLayoutManager::default()
    }
}
