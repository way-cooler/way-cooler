//! Compositor: wm state including the layout, helm, and interactions
//! state machines.

use std::sync::RwLock;

use super::layout::{Node, Container};

lazy_static! {
    static ref COMPSOITOR: RwLock<Compositor> = RwLock::new(Compositor::new());
}

#[derive(Debug, PartialEq)]
pub struct Compositor {
    pub layout: Node,
    pub actions: ClientState
}

impl Compositor {
    fn new() -> Compositor {
        Compositor {
            layout: Node::new(Container::new_root()),
            actions: ClientState::default()
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
