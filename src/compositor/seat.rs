use compositor::View;
use std::rc::Rc;
use wlroots::{Origin, SeatHandle, SeatHandler};

#[derive(Debug, Default)]
pub struct SeatManager;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Action {
    /// We are moving a view.
    ///
    /// The start is the surface level coordinates of where the first click was
    Moving { start: Origin }
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Seat {
    pub seat: SeatHandle,
    pub focused: Option<Rc<View>>,
    pub action: Option<Action>,
    pub meta: bool
}

impl Seat {
    pub fn new(seat: SeatHandle) -> Seat {
        Seat { seat,
               meta: false,
               ..Seat::default() }
    }

    pub fn clear_focus(&mut self) {
        if let Some(mut focused_view) = self.focused.take() {
            focused_view.activate(false);
        }
        with_handles!([(seat: {&mut self.seat})] => {
            seat.keyboard_clear_focus();
        }).unwrap();
    }

    pub fn focus_view(&mut self, view: Rc<View>, views: &mut Vec<Rc<View>>) {
        if let Some(ref focused) = self.focused {
            if focused == &view {
                return
            }
            focused.activate(false);
        }
        self.focused = Some(view.clone());
        view.activate(true);

        if let Some(idx) = views.iter().position(|v| *v == view) {
            let v = views.remove(idx);
            views.insert(0, v);
        }

        with_handles!([(seat: {&mut self.seat})] => {
            if let Some(keyboard) = seat.get_keyboard() {
                with_handles!([(keyboard: {keyboard}), (surface: {view.surface()})] => {
                    seat.keyboard_notify_enter(surface,
                                               &mut keyboard.keycodes(),
                                               &mut keyboard.get_modifier_masks());
                }).unwrap();
            }
        }).unwrap();
    }
}

impl SeatHandler for SeatManager {}

impl SeatManager {
    pub fn new() -> Self {
        SeatManager::default()
    }
}
