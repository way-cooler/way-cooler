//! Commands from the user to manipulate the tree

use super::try_lock_tree;
use super::{ContainerType, Direction, Layout};
use super::TreeGuard;

use rustwlc::{WlcView, WlcOutput, ViewType};

pub type CommandResult = Result<(), String>;

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

/* Commands that can be chained together with a locked tree */

/// Adds an Output to the tree. Never fails
pub fn add_output(tree: &mut TreeGuard, output: WlcOutput) -> CommandResult {
    tree.add_output(output);
    Ok(())
}

/// Adds a Workspace to the tree. Never fails
pub fn switch_to_workspace(tree: &mut TreeGuard, name: &str) -> CommandResult {
    tree.switch_to_workspace(name);
    Ok(())
}

/// Tiles the active container of some container type. Never fails
pub fn layout_active_of(tree: &mut TreeGuard, c_type: ContainerType) -> CommandResult {
    tree.layout_active_of(c_type);
    Ok(())
}

/// Adds a view to the workspace of the active container
pub fn add_view(tree: &mut TreeGuard, view: WlcView) -> CommandResult {
    let output = view.get_output();
    if tree.get_active_container().is_none() {
        return Err(format!("No active container, cannot add view {:?} to output {:?}!", view, output))
    }
    view.set_mask(output.get_mask());
    let v_type = view.get_type();
    // If it is empty, don't add to tree
    if v_type != ViewType::empty() {
        // Now focused on something outside the tree,
        // have to unset the active container
        if !tree.active_is_root() {
            tree.unset_active_container();
        }
        return Ok(())
    }
    tree.add_view(view.clone());
    tree.normalize_view(view.clone());
    tree.layout_active_of(ContainerType::Container);
    Ok(())
}

/// Attempts to remove a view from the tree. If it is not in the tree it fails
pub fn remove_view(tree: &mut TreeGuard, view: WlcView) -> CommandResult {
    tree.remove_view(&view)
}

/// Sets the view to be the new active container. Never fails
pub fn set_active_view(tree: &mut TreeGuard, view: WlcView) -> CommandResult {
    tree.set_active_container(view);
    Ok(())
}

/// Destroy the tree
pub fn destroy_tree(tree: &mut TreeGuard) -> CommandResult {
    tree.destroy_tree();
    Ok(())
}
