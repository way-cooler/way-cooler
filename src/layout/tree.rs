//! Main module to handle the layout.
//! This is where the i3-specific code is.

use super::container::{Container, Handle, ContainerType};
use super::node::{Node};
use super::super::rustwlc::handle::{WlcView, WlcOutput};

use std::sync::{Mutex, MutexGuard, TryLockError};

use std::ptr;

pub type TreeResult = Result<(), TryLockError<MutexGuard<'static, Tree>>>;

pub struct Tree {
    root: Node,
    active_container: *const Node,
}

impl Tree {
    fn get_active_container(&self) -> Option<&Node> {
        if self.active_container.is_null() {
            None
        } else {
            unsafe {
                Some(&*self.active_container)
            }
        }
    }
}

unsafe impl Send for Tree {}

lazy_static! {
    static ref TREE: Mutex<Tree> = {
        Mutex::new(Tree{
            root: Node::new(Container::new_root()),
            active_container: ptr::null(),
        })
    };
}

pub fn add_output(wlc_output: WlcOutput) -> TreeResult {
    {
        let mut tree = try!(TREE.try_lock());
        let output = Container::new_output(wlc_output);
        tree.root.new_child(output);
    }
    try!(add_workspace(&"1"));
    try!(switch_workspace(&"1"));
    Ok(())
}

pub fn add_workspace(name: &str) -> TreeResult {
    trace!("Adding new workspace to root");
    let mut tree = try!(TREE.lock());
    let workspace = Container::new_workspace(name.to_string());
    // NOTE handle multiple outputs
    tree.root.get_children_mut()[0].new_child(workspace);
    Ok(())
}

pub fn add_view(wlc_view: WlcView) -> TreeResult {
    let tree = try!(TREE.lock());
    if let Some(current_workspace) = get_current_workspace(&tree) {
        trace!("Adding view {:?} to {:?}", wlc_view, current_workspace);
        current_workspace.new_child(Container::new_view(wlc_view));
    }
    Ok(())
}

pub fn remove_view(wlc_view: &WlcView) -> TreeResult {
    let tree = try!(TREE.lock());
    if let Some(view) = tree.root.find_view_by_handle(&wlc_view) {
        let parent = view.get_parent().unwrap();
        parent.remove_child(view);
    }
    Ok(())
}

pub fn switch_workspace(name: &str) -> TreeResult {
    trace!("Switching to workspace {}", name);
    let mut tree = try!(TREE.lock());
    if let Some(old_workspace) = get_current_workspace(&tree) {
        // Make all the views in the original workspace to be invisible
        for view in old_workspace.get_children_mut() {
            trace!("Setting {:?} invisible", view);
            match view.get_val().get_handle().unwrap() {
                Handle::View(view) => view.set_mask(0),
                _ => {},
            }
        }
    }
    let current_workspace: *const Node;
    {
        let new_current_workspace: &mut Node;
        if let Some(_) = get_workspace_by_name(&tree, name) {
            trace!("Found workspace {}", name);
            new_current_workspace = get_workspace_by_name_mut(&mut tree, name).unwrap();
        } else {
            drop(tree);
            try!(add_workspace(name));
            tree = try!(TREE.lock());
            new_current_workspace = get_workspace_by_name_mut(&mut tree, name).unwrap();
        }
        for view in new_current_workspace.get_children_mut() {
            trace!("Setting {:?} visible", view);
            match view.get_val().get_handle().unwrap() {
                Handle::View(view) => view.set_mask(1),
                _ => {},
            }
        }
        // Set the first view to be focused, so that the view is updated to this new workspace
        if new_current_workspace.get_children().len() > 0 {
            trace!("Focusing view");
            match new_current_workspace.get_children_mut()[0].get_val().get_handle().unwrap() {
                Handle::View(view) => view.focus(),
                _ => {},
            }
        } else {
            WlcView::root().focus();
        }
        current_workspace = new_current_workspace as *const Node;
    }
    tree.active_container = current_workspace;
    Ok(())
}

/// Finds the WlcOutput associated with the WlcView from the tree
pub fn get_output_of_view(wlc_view: &WlcView) -> Option<WlcOutput> {
    let tree = TREE.lock().unwrap();
    if let Some(view_node) = tree.root.find_view_by_handle(wlc_view) {
        if let Some(output_node) = view_node.get_ancestor_of_type(ContainerType::Output) {
            if let Some(handle) =  output_node.get_val().get_handle() {
                return match handle {
                    Handle::Output(output) => Some(output),
                    _ => None
                }
            }
        }
    }
    return None;
}

#[allow(dead_code)]
fn get_focused_workspace<'a>(tree: &'a Tree) -> Option<&'a Node> {
    for output in tree.root.get_children() {
        if output.get_val().is_focused() {
            for workspace in output.get_children() {
                if workspace.get_val().is_focused() {
                    return Some(workspace);
                }
            }
        }
    }
    None

}

fn get_current_workspace<'a>(tree: &'a Tree) -> Option<&'a mut Node> {
    if let Some(container) = tree.get_active_container() {
        //if let Some(child) = container.get_ancestor_of_type(ContainerType::Workspace) {
            //return child.get_children()[0].get_parent()

        //}
        // NOTE hack here, remove commented code above to make this work properly
        let parent = container.get_parent().unwrap();
        for child in parent.get_children_mut() {
            if child == container {
                return Some(child);
            }
        }
    }
    return None
}

fn get_workspace_by_name<'a, 'b>(tree: &'a Tree, name: &'b str) -> Option<&'a Node> {
    for child in tree.root.get_children()[0].get_children() {
        if child.get_val().get_name().unwrap() != name {
            continue
        }
        return Some(child);
    }
    return None
}

fn get_workspace_by_name_mut<'a, 'b>(tree: &'a mut Tree, name: &'b str) -> Option<&'a mut Node> {
    for child in tree.root.get_children_mut()[0].get_children_mut() {
        if child.get_val().get_name().unwrap() != name {
            continue
        }
        return Some(child);
    }
    return None
}

