//! Commands from the user to manipulate the tree

use super::try_lock_tree;
use super::{ContainerType, Direction, Layout};

pub fn remove_active() {
    if let Ok(mut tree) = try_lock_tree() {
        tree.remove_active();
    }
}

pub fn tile_switch() {
    if let Ok(mut tree) = try_lock_tree() {
        tree.toggle_active_horizontal();
        tree.layout_active_of(ContainerType::Workspace);
    }
}

pub fn split_vertical() {
    if let Ok(mut tree) = try_lock_tree() {
        tree.toggle_active_layout(Layout::Vertical);
    }
}

pub fn split_horizontal() {
    if let Ok(mut tree) = try_lock_tree() {
        tree.toggle_active_layout(Layout::Horizontal);
    }
}

pub fn focus_left() {
    if let Ok(mut tree) = try_lock_tree() {
        tree.move_focus(Direction::Left);
    }
}

pub fn focus_right() {
    if let Ok(mut tree) = try_lock_tree() {
        tree.move_focus(Direction::Right);
    }
}

pub fn focus_up() {
    if let Ok(mut tree) = try_lock_tree() {
        tree.move_focus(Direction::Up);
    }
}

pub fn focus_down() {
    if let Ok(mut tree) = try_lock_tree() {
        tree.move_focus(Direction::Down);
    }
}
