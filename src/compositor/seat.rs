use wlroots::SeatHandler;

#[derive(Debug, Default)]
pub struct SeatManager;

impl SeatHandler for SeatManager {}

impl SeatManager {
    pub fn new() -> Self {
        SeatManager::default()
    }
}
