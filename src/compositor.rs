//! Compositor: wm state including the layout, helm, and interactions
//! state machines.

use std::sync::RwLock;

use rustwlc;
use rustwlc::*;
use rustwlc::handle::{WlcView, WlcOutput};

use super::layout::{Node, Container};

lazy_static! {
    static ref COMPOSITOR: RwLock<Compositor> = RwLock::new(Compositor::new());
}

#[derive(Debug, PartialEq)]
pub struct Compositor {
    pub view: Option<WlcView>,
    pub grab: Point,
    pub edges: ResizeEdge,
    //pub actions: ClientState
}

impl Compositor {
    fn new() -> Compositor {
        Compositor {
            view: None,
            grab: Point {x: 0, y: 0},
            edges: ResizeEdge::empty()
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ClientState {
    view_action: ViewAction,
    next_action: ClickAction
}

impl Default for ClientState {
    fn default() -> ClientState {
        ClientState {
            view_action: ViewAction::None,
            next_action: ClickAction::None
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ViewAction {
    None,
    Resize,
    Move
}

impl ViewAction {
    /// Is this ViewAction set
    pub fn is_some(&self) -> bool {
        *self != ViewAction::None
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
/// Actions that the user may set before clicking on a window
enum ClickAction {
    None,
    CloseWindow,
    BeginMoving,
    BeginResizing,
    SelectWindow,

}

pub fn stop_interactive_action() {
    let mut comp = COMPOSITOR.write().unwrap();

    match comp.view {
        None => return,
    Some(ref view) =>
    view.set_state(VIEW_RESIZING, false)
    }

    (*comp).view = None;
    comp.edges = ResizeEdge::empty();
}

pub fn start_interactive_resize(view: &WlcView, edges: ResizeEdge, origin: &Point) {
        let geometry = match view.get_geometry() {
            None => { return; }
        Some(g) => g,
    };

    if !start_interactive_action(view, origin) {
        return;
    }
    let halfw = geometry.origin.x + geometry.size.w as i32 / 2;
    let halfh = geometry.origin.y + geometry.size.h as i32 / 2;

    {
        let mut comp = COMPOSITOR.write().unwrap();
        comp.edges = edges.clone();
        if comp.edges.bits() == 0 {
            let flag_x = if origin.x < halfw {
                RESIZE_LEFT
            } else if origin.x > halfw {
                RESIZE_RIGHT
            } else {
                ResizeEdge::empty()
            };

            let flag_y = if origin.y < halfh {
                RESIZE_TOP
            } else if origin.y > halfh {
                RESIZE_BOTTOM
            } else {
                ResizeEdge::empty()
            };

            comp.edges = flag_x | flag_y;
        }
    }
    view.set_state(VIEW_RESIZING, true);
}

fn start_interactive_action(view: &WlcView, origin: &Point) -> bool {
    {
        let mut comp = COMPOSITOR.write().unwrap();
        if comp.view != None {
            return false;
        }
        comp.grab = origin.clone();
        comp.view = Some(view.clone());
    }

    view.bring_to_front();
    return true;
}

pub fn start_interactive_move(view: &WlcView, origin: &Point) -> bool {
    {
        let mut comp = COMPOSITOR.write().unwrap();
        if comp.view != None {
            return false;
        }
        comp.grab = origin.clone();
        comp.view = Some(view.clone());
    }

    view.bring_to_front();
    return true;
}

pub fn on_pointer_button(view: WlcView, _time: u32, mods: &KeyboardModifiers, button: u32,
                         state: ButtonState, point: &Point) -> bool {
    if state == ButtonState::Pressed {
        if !view.is_root() && mods.mods.contains(MOD_CTRL) {
            view.focus();
            if mods.mods.contains(MOD_CTRL) {
                // Button left, we need to include linux/input.h somehow
                if button == 0x110 {
                    start_interactive_move(&view, point);
                }
                if button == 0x111 {
                    start_interactive_resize(&view, ResizeEdge::empty(), point);
                }
            }
        }
    }
    else {
        stop_interactive_action();
    }

    {
        let comp = COMPOSITOR.read().unwrap();
        return comp.view.is_some();
    }
}

pub fn on_pointer_motion(_view: WlcView, _time: u32, point: &Point) -> bool {
    rustwlc::input::pointer::set_position(point);
    {
        let comp = COMPOSITOR.read().unwrap();
        if let Some(ref view) = comp.view {
            let dx = point.x - comp.grab.x;
            let dy = point.y - comp.grab.y;
            let mut geo = view.get_geometry().unwrap().clone();
            if comp.edges.bits() != 0 {
                let min = Size { w: 80u32, h: 40u32};
                let mut new_geo = geo.clone();

                if comp.edges.contains(RESIZE_LEFT) {
                    if dx < 0 {
                        new_geo.size.w += dx.abs() as u32;
                    } else {
                        new_geo.size.w -= dx.abs() as u32;
                    }
                    new_geo.origin.x += dx;
                }
                else if comp.edges.contains(RESIZE_RIGHT) {
                    if dx < 0 {
                        new_geo.size.w -= dx.abs() as u32;
                    } else {
                        new_geo.size.w += dx.abs() as u32;
                    }
                }

                if comp.edges.contains(RESIZE_TOP) {
                    if dy < 0 {
                        new_geo.size.h += dy.abs() as u32;
                    } else {
                        new_geo.size.h -= dy.abs() as u32;
                    }
                    new_geo.origin.y += dy;
                }
                else if comp.edges.contains(RESIZE_BOTTOM) {
                    if dy < 0 {
                        new_geo.size.h -= dy.abs() as u32;
                    } else {
                        new_geo.size.h += dy.abs() as u32;
                    }
                }

                if new_geo.size.w >= min.w {
                    geo.origin.x = new_geo.origin.x;
                    geo.size.w = new_geo.size.w;
                }

                if new_geo.size.h >= min.h {
                    geo.origin.y = new_geo.origin.y;
                    geo.size.h = new_geo.size.h;
                }

                view.set_geometry(comp.edges, &geo);
            }
            else {
                geo.origin.x += dx;
                geo.origin.y += dy;
                view.set_geometry(ResizeEdge::empty(), &geo);
            }
        }
    }
    {
        let mut comp = COMPOSITOR.write().unwrap();
        comp.grab = point.clone();
        return comp.view.is_some()
    }
}
