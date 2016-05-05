//! Main module to handle the layout.
//! This is where the i3-specific code is.

pub mod layout {
    use super::super::container::{Container, Handle, ContainerType};
    use super::super::tree::Node;
    use super::super::super::rustwlc::handle::{WlcView, WlcOutput};

    use std::sync::Mutex;


    struct RootNode<'a> {
        root: Node,
        active_container: Option<&'a Node>
    }

    lazy_static! {
        static ref ROOT: Mutex<RootNode<'static>> = {
            Mutex::new(RootNode{
                root: Node::new(Container::new_root()),
                active_container: None,
            })
        };

        static ref CURRENT_WORKSPACE: Mutex<u32> = Mutex::new(0);
    }

    pub fn add_output(wlc_output: WlcOutput) {
        let output = Container::new_output(wlc_output);
        ROOT.lock().unwrap().root.new_child(output);
        add_workspace();
        add_workspace();
    }

     pub fn add_workspace() {
         let workspace_count = (*ROOT.lock().unwrap()).root.get_children().len();
         let workspace = Container::new_workspace((workspace_count + 1).to_string());
         (*ROOT.lock().unwrap()).root.get_children_mut()[0].new_child(workspace);
    }

    pub fn add_view(wlc_view: WlcView) {
        let current_workspace = CURRENT_WORKSPACE.lock().unwrap();
        let mut root = ROOT.lock().unwrap();
        let mut workspace = (*root).root.get_children_mut()[0].get_children_mut().get_mut(*current_workspace as usize).unwrap();
        workspace.new_child(Container::new_view(wlc_view));
    }

    pub fn remove_view(wlc_view: &WlcView) {
        let mut root = ROOT.lock().unwrap();
        if let Some(view) = root.root.find_view_by_handle(&wlc_view) {
            let parent = view.get_parent().unwrap();
            parent.remove_child(view);
        }
    }

    pub fn switch_workspace(index: u32) {
        let mut current_workspace = CURRENT_WORKSPACE.lock().unwrap();
        let mut root = ROOT.lock().unwrap();
        let output = &mut (*root).root.get_children_mut()[0];
        let workspace_children = output.get_children_mut();
        {
            let old_workspace = workspace_children.get_mut(*current_workspace as usize).unwrap();
            for view in old_workspace.get_children_mut() {
                match view.get_val().get_handle().unwrap() {
                    Handle::View(view) => view.set_mask(0),
                    _ => {},
                }
            }
        }
        // Assume that the other workspace already exits
        trace!("here");
        let new_workspace = workspace_children.get_mut(index as usize).unwrap();
        trace!("here");
        for view in new_workspace.get_children_mut() {
            match view.get_val().get_handle().unwrap() {
                Handle::View(view) => view.set_mask(1),
                _ => {},
            }
        }
        *current_workspace = index;
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
 }

