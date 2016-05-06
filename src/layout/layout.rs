//! Main module to handle the layout.
//! This is where the i3-specific code is.

pub mod layout {
    use super::super::container::{Container, Handle, ContainerType};
    use super::super::tree::Node;
    use super::super::super::rustwlc::handle::{WlcView, WlcOutput};

    use std::sync::Mutex;

    use std::ptr;

    struct RootNode {
        root: Node,
        active_container: *const Node,
    }

    impl RootNode {
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

    unsafe impl Send for RootNode {}

    lazy_static! {
        static ref ROOT: Mutex<RootNode> = {
            Mutex::new(RootNode{
                root: Node::new(Container::new_root()),
                active_container: ptr::null(),
            })
        };
    }

    pub fn add_output(wlc_output: WlcOutput) {
        let mut root = ROOT.lock().unwrap();
        let output = Container::new_output(wlc_output);
        root.root.new_child(output);
        drop(root);
        add_workspace(&"1");
        switch_workspace(&"1");
    }

    pub fn add_workspace(name: &str) {
        trace!("Adding new workspace to root");
        let mut root = ROOT.lock().unwrap();
        let workspace = Container::new_workspace(name.to_string());
        // NOTE handle multiple outputs
        root.root.get_children_mut()[0].new_child(workspace);
    }

    pub fn add_view(wlc_view: WlcView) {
        let mut root = ROOT.lock().unwrap();
        if let Some(current_workspace) = get_current_workspace(&root) {
            trace!("Adding view {:?} to {:?}", wlc_view, current_workspace);
            current_workspace.new_child(Container::new_view(wlc_view));
        }
    }

    pub fn remove_view(wlc_view: &WlcView) {
        let mut root = ROOT.lock().unwrap();
        if let Some(view) = root.root.find_view_by_handle(&wlc_view) {
            let parent = view.get_parent().unwrap();
            parent.remove_child(view);
        }
    }

    pub fn switch_workspace(name: &str) {
        trace!("Switching to workspace {}", name);
        let mut root = ROOT.lock().unwrap();
        if let Some(old_workspace) = get_current_workspace(&root) {
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
            if let Some(_) = get_workspace_by_name(&root, name) {
                trace!("Found workspace {}", name);
                new_current_workspace = get_workspace_by_name_mut(&mut root, name).unwrap();
            } else {
                drop(root);
                add_workspace(name);
                root = ROOT.lock().unwrap();
                new_current_workspace = get_workspace_by_name_mut(&mut root, name).unwrap();
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
        root.active_container = current_workspace;
    }

    fn get_focused_workspace<'a>(root: &'a RootNode) -> Option<&'a Node> {
        for output in root.root.get_children() {
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

    fn get_current_workspace<'a>(root: &'a RootNode) -> Option<&'a mut Node> {
        if let Some(container) = root.get_active_container() {
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

    fn get_workspace_by_name<'a, 'b>(root: &'a RootNode, name: &'b str) -> Option<&'a Node> {
        for child in root.root.get_children()[0].get_children() {
            if child.get_val().get_name().unwrap() != name {
                continue
            }
            return Some(child);
        }
        return None
    }

    fn get_workspace_by_name_mut<'a, 'b>(root: &'a mut RootNode, name: &'b str) -> Option<&'a mut Node> {
        for child in root.root.get_children_mut()[0].get_children_mut() {
            if child.get_val().get_name().unwrap() != name {
                continue
            }
            return Some(child);
        }
        return None
    }
 }

