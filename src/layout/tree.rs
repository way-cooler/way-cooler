//! Main module to handle the layout.
//! This is where the i3-specific code is.

use std::sync::{Mutex, MutexGuard, TryLockError};
use std::ptr;

use super::container::{Container, Handle, ContainerType};
use super::node::{Node};
use super::super::rustwlc::handle::{WlcView, WlcOutput};


pub type TreeResult = Result<(), TryLockError<MutexGuard<'static, Tree>>>;

const ERR_BAD_TREE: &'static str = "Layout tree was in an invalid configuration";

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

    fn get_active_output(&self) -> Option<&Node> {
        if let Some(node) = self.get_active_container() {
            node.get_ancestor_of_type(ContainerType::Output)
        } else {
            None
        }
    }

    fn get_current_workspace(&self) -> Option<&mut Node> {
        if let Some(container) = self.get_active_container() {
            //if let Some(child) = container.get_ancestor_of_type(ContainerType::Workspace) {
            //return child.get_children()[0].get_parent()

            //}
            // NOTE hack here, remove commented code above to make this work properly
            let parent = container.get_parent().expect(ERR_BAD_TREE);
            for child in parent.get_children_mut() {
                if child == container {
                    return Some(child);
                }
            }
        }
        return None
    }

    fn get_output_of_view(&self, wlc_view: &WlcView) -> Option<WlcOutput> {
        if let Some(view_node) = self.root.find_view_by_handle(wlc_view) {
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

    fn get_workspace_by_name(&self, name: &str) -> Option<&Node> {
        for child in self.root.get_children()[0].get_children() {
            if child.get_val().get_name().expect(ERR_BAD_TREE) != name {
                continue
            }
            return Some(child);
        }
        return None
    }

    fn get_workspace_by_name_mut(&mut self, name: &str) -> Option<&mut Node> {
        for child in self.root.get_children_mut()[0].get_children_mut() {
            if child.get_val().get_name().expect(ERR_BAD_TREE) != name {
                continue
            }
            return Some(child);
        }
        return None
    }


    fn add_output(&mut self, wlc_output: WlcOutput) {
        self.root.new_child(Container::new_output(wlc_output));
    }

    fn add_workspace(&mut self, name: &str) {
        let workspace = Container::new_workspace(name.to_string());
        let mut index = 0;
        if let Some(output) = self.get_active_output() {
            for (cur_index, child) in self.root.get_children().iter().enumerate() {
                if child == output {
                    index = cur_index;
                    break;
                }
            }
        }
        self.root.get_children_mut()[index].new_child(workspace);
    }

    fn add_view(&self, wlc_view: WlcView) {
        if let Some(current_workspace) = self.get_current_workspace() {
            trace!("Adding view {:?} to {:?}", wlc_view, current_workspace);
            current_workspace.new_child(Container::new_view(wlc_view));
        }
    }

    fn remove_view(&self, wlc_view: &WlcView) {
        if let Some(view) = self.root.find_view_by_handle(&wlc_view) {
            let parent = view.get_parent().expect(ERR_BAD_TREE);
            parent.remove_child(view);
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
        tree.add_output(wlc_output);
    }
    try!(add_workspace(&"1"));
    try!(switch_workspace(&"1"));
    Ok(())
}

pub fn add_workspace(name: &str) -> TreeResult {
    trace!("Adding new workspace to root");
    let mut tree = try!(TREE.lock());
    tree.add_workspace(name);
    Ok(())
}

pub fn add_view(wlc_view: WlcView) -> TreeResult {
    let tree = try!(TREE.lock());
    tree.add_view(wlc_view);
    Ok(())
}

pub fn remove_view(wlc_view: &WlcView) -> TreeResult {
    let tree = try!(TREE.lock());
    tree.remove_view(wlc_view);
    Ok(())
}

pub fn switch_workspace(name: &str) -> TreeResult {
    trace!("Switching to workspace {}", name);
    let mut tree = try!(TREE.lock());
    if let Some(old_workspace) = tree.get_current_workspace() {
        // Make all the views in the original workspace to be invisible
        for view in old_workspace.get_children_mut() {
            trace!("Setting {:?} invisible", view);
            match view.get_val().get_handle().expect(ERR_BAD_TREE) {
                Handle::View(view) => view.set_mask(0),
                _ => {},
            }
        }
    }
    let current_workspace: *const Node;
    {
        let new_current_workspace: &mut Node;
        if let Some(_) = tree.get_workspace_by_name(name) {
            trace!("Found workspace {}", name);
            new_current_workspace = tree.get_workspace_by_name_mut(name)
                .expect(ERR_BAD_TREE);
        } else {
            drop(tree);
            try!(add_workspace(name));
            tree = try!(TREE.lock());
            new_current_workspace = tree.get_workspace_by_name_mut(name)
                .expect(ERR_BAD_TREE);
        }
        for view in new_current_workspace.get_children_mut() {
            trace!("Setting {:?} visible", view);
            match view.get_val().get_handle().expect(ERR_BAD_TREE) {
                Handle::View(view) => view.set_mask(1),
                _ => {},
            }
        }
        // Set the first view to be focused, so that the view is updated to this new workspace
        if new_current_workspace.get_children().len() > 0 {
            trace!("Focusing view");
            match new_current_workspace.get_children_mut()[0]
                .get_val().get_handle().expect(ERR_BAD_TREE) {
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
    let tree = TREE.lock().expect("Unable to lock layout tree!");
    tree.get_output_of_view(wlc_view)
}

