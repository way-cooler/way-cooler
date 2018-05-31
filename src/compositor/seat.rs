use compositor::{Server, View};
use std::rc::Rc;
use std::time::Duration;
use wlroots::events::seat_events::SetCursorEvent;
use wlroots::pointer_events::ButtonEvent;
use wlroots::{CompositorHandle, Origin, SeatHandle, SeatHandler};

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
    pub has_client_cursor: bool,
    pub meta: bool
}

impl Seat {
    pub fn new(seat: SeatHandle) -> Seat {
        Seat { seat,
               meta: false,
               ..Seat::default() }
    }

    pub fn clear_focus(&mut self) {
        if let Some(focused_view) = self.focused.take() {
            focused_view.activate(false);
        }
        with_handles!([(seat: {&mut self.seat})] => {
            seat.keyboard_clear_focus();
        }).unwrap();
    }

    pub fn focus_view(&mut self, view: Rc<View>, views: &mut Vec<Rc<View>>) {
        if let Some(ref focused) = self.focused {
            if *focused == view {
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

    pub fn send_button(&self, event: &ButtonEvent) {
        with_handles!([(seat: {&self.seat})] => {
            seat.pointer_notify_button(Duration::from_millis(event.time_msec() as _),
            event.button(),
            event.state() as u32);
        }).unwrap();
    }
}

impl SeatHandler for SeatManager {
    fn cursor_set(&mut self, compositor: CompositorHandle, _: SeatHandle, event: &SetCursorEvent) {
        if let Some(surface) = event.surface() {
            with_handles!([(compositor: {compositor}), (surface: {surface})] => {
                let server: &mut Server = compositor.into();
                let Server { ref mut cursor,
                             ref mut seat,
                .. } = *server;
                with_handles!([(cursor: {&mut *cursor})] => {
                    let (hotspot_x, hotspot_y) = event.location();
                    let surface = &*surface;
                    cursor.set_surface(Some(surface), hotspot_x, hotspot_y);
                    seat.has_client_cursor = true;
                }).unwrap();
            }).unwrap();
        }
    }
}

impl SeatManager {
    pub fn new() -> Self {
        SeatManager::default()
    }
}
