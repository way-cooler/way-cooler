use compositor::View;
use wlroots::{SeatHandle, SeatHandler};

#[derive(Debug, Default)]
pub struct SeatManager;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Action {
    Moving(View),
    Resizing(View)
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Seat {
    pub seat: SeatHandle,
    pub action: Option<Action>
}

impl Seat {
    pub fn new(seat: SeatHandle) -> Seat {
        Seat { seat,
               ..Seat::default() }
    }
}

impl SeatHandler for SeatManager {}

impl SeatManager {
    pub fn new() -> Self {
        SeatManager::default()
    }
}
