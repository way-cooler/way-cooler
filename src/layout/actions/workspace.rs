use petgraph::graph::NodeIndex;
use uuid::Uuid;
use rustwlc::{Geometry, Point};
use super::super::LayoutTree;
use super::super::core::container::{Container, ContainerType};

// TODO This module needs to be updated like the other modules...
// Need to add some errors for this (such as when trying to move a non-container/view,
// or when trying to grab a workspace whos name already exists)
//
// Also the code is generally pretty crap, cause it's pretty old (mid-2016)

impl LayoutTree {
    /// Gets a workspace by name or creates it
    fn get_or_make_workspace(&mut self, name: &str) -> NodeIndex {
        let active_index = self.active_ix_of(ContainerType::Output)
            .or_else(|| {
                self.tree.follow_path_until(self.tree.root_ix(), ContainerType::Output).ok()
            })
            .expect("get_or_make_wksp: Couldn't get output");
        let workspace_ix = self.tree.workspace_ix_by_name(name).unwrap_or_else(|| {
            let root_ix = self.init_workspace(name.to_string(), active_index);
            self.tree.parent_of(root_ix)
                .expect("Workspace was not properly initialized with a root container")
        });
        self.validate();
        workspace_ix
    }

    /// Initializes a workspace and gets the index of the root container
    pub fn init_workspace(&mut self, name: String, output_ix: NodeIndex)
                      -> NodeIndex {
        let size = self.tree.get(output_ix)
            .expect("init_workspace: invalid output").get_geometry()
            .expect("init_workspace: no geometry for output").size;
        let worksp = Container::new_workspace(name.to_string(), size.clone());

        trace!("Adding workspace {:?}", worksp);
        let worksp_ix = self.tree.add_child(output_ix, worksp, false);
        let geometry = Geometry {
            size: size, origin: Point { x: 0, y: 0 }
        };
        let container_ix = self.tree.add_child(worksp_ix,
                                               Container::new_container(geometry), false);
        self.tree.set_ancestor_paths_active(container_ix);
        self.validate();
        container_ix
    }

    /// Switch to the specified workspace
    pub fn switch_to_workspace(&mut self, name: &str) {
        let maybe_active_ix = self.active_container
            .or_else(|| {
                let new_active = self.tree.follow_path(self.tree.root_ix());
                match self.tree[new_active].get_type() {
                    ContainerType::View | ContainerType::Container => {
                        Some(new_active)
                    },
                    // else try and get the root container
                    _ => self.tree.descendant_of_type(new_active, ContainerType::Container).ok()
                }
            });
        if maybe_active_ix.is_none() {
            warn!("{:#?}", self);
            warn!("No active container, cannot switch");
            return;
        }
        let active_ix = maybe_active_ix.unwrap();
        // Get the old (current) workspace
        let old_worksp_ix: NodeIndex;
        if let Ok(index) = self.tree.ancestor_of_type(active_ix, ContainerType::Workspace) {
            old_worksp_ix = index;
            trace!("Switching to workspace {}", name);
        } else {
            match self.tree[active_ix].get_type() {
                ContainerType::Workspace => {
                    old_worksp_ix = active_ix;
                    trace!("Switching to workspace {}", name);
                },
                _ => {
                    warn!("Could not find old workspace, could not set invisible");
                    return;
                }
            }
        }
        // Get the new workspace, or create one if it doesn't work
        let mut workspace_ix = self.get_or_make_workspace(name);
        if old_worksp_ix == workspace_ix {
            return;
        }
        // Set the old one to invisible
        self.tree.set_family_visible(old_worksp_ix, false);
        // Set the new one to visible
        self.tree.set_family_visible(workspace_ix, true);
        // Delete the old workspace if it has no views on it
        self.active_container = None;
        if self.tree.descendant_of_type(old_worksp_ix, ContainerType::View).is_err() {
            trace!("Removing workspace: {:?}", self.tree[old_worksp_ix].get_name()
                   .expect("Workspace had no name"));
            if let Err(err) = self.remove_container(old_worksp_ix) {
                warn!("Tried to remove {:?}, got: {:#?}", old_worksp_ix, err);
                panic!("Could not remove old workspace");
            }
        }
        workspace_ix = self.tree.workspace_ix_by_name(name)
            .expect("Workspace we just made was deleted!");
        let active_ix = self.tree.follow_path(workspace_ix);
        match self.tree[active_ix].get_type() {
            ContainerType::View  => {
                match self.tree[active_ix] {
                    Container::View { ref handle, ..} => {
                        trace!("View found, focusing on {:?}", handle);
                        handle.focus();
                    },
                    _ => unreachable!()
                }
                self.active_container = Some(active_ix);
                if !self.tree[active_ix].floating() {
                    self.tree.set_ancestor_paths_active(active_ix);
                } else {
                    let root_c_ix = *self.tree.children_of(workspace_ix).get(0)
                        .expect("The workspace we are switching to had no root container");
                    self.tree.set_ancestor_paths_active(root_c_ix);
                }
                self.validate();
                self.validate_path();
                return;
            },
            _ => {
                self.active_container = self.tree.descendant_of_type(active_ix, ContainerType::View)
                    .or_else(|_| self.tree.descendant_of_type(active_ix,
                                                              ContainerType::Container)).ok();
                match self.tree[self.active_container.expect("Workspace had NO children!")] {
                    Container::View { floating, .. } => {
                        if !floating {
                            self.tree.set_ancestor_paths_active(self.active_container.unwrap());
                        } else {
                            let root_c_ix = *self.tree.children_of(workspace_ix).get(0)
                            .expect("The workspace we are switching to had no root container");
                            self.tree.set_ancestor_paths_active(root_c_ix);
                        }
                    },
                    Container::Container { .. } => {
                        self.tree.set_ancestor_paths_active(self.active_container.unwrap());
                    }
                    _ => unreachable!()
                };
            }
        }
        trace!("Focusing on next container");
        self.focus_on_next_container(workspace_ix);
        self.validate();
        self.validate_path();
    }

    /// Moves the active container to a new workspace.
    pub fn send_active_to_workspace(&mut self, name: &str) {
        if let Some(active_ix) = self.active_container {
            let id = self.tree[active_ix].get_id();
            self.send_to_workspace(id, name);
        }
    }
    /// Moves a container to a new workspace
    pub fn send_to_workspace(&mut self, id: Uuid, name: &str) {
        let node_ix = self.tree.lookup_id(id);
        // Ensure focus
        if let Some(active_ix) = node_ix.or(self.active_container) {
            let curr_work_ix = self.active_ix_of(ContainerType::Workspace)
                .expect("send_active: Not currently in a workspace!");
            if active_ix == self.tree.children_of(curr_work_ix)[0] {
                warn!("Tried to move the root container of a workspace, aborting move");
                return;
            }
            let next_work_ix = self.get_or_make_workspace(name);

            // Check if the workspaces are the same
            if next_work_ix == curr_work_ix {
                trace!("Attempted to move a view to the same workspace {}!", name);
                return;
            }
            self.tree.set_family_visible(curr_work_ix, false);

            // Save the parent of this view for focusing
            let maybe_active_parent = self.tree.parent_of(active_ix);

            // Get the root container of the next workspace
            let next_work_children = self.tree.children_of(next_work_ix);
            if cfg!(debug_assertions) {
                assert!(next_work_children.len() == 1,
                        "Next workspace has multiple roots!");
            }
            let next_work_root_ix = next_work_children[0];

            // Move the container
            info!("Moving container {:?} to workspace {}",
                self.get_active_container(), name);
            self.tree.move_node(active_ix, next_work_root_ix);

            // Update the active container
            if let Ok(parent_ix) = maybe_active_parent {
                self.tree.set_ancestor_paths_active(parent_ix);
                let ctype = self.tree.node_type(parent_ix).unwrap_or(ContainerType::Root);
                if ctype == ContainerType::Container {
                    self.focus_on_next_container(parent_ix);
                } else {
                    trace!("Send to container invalidated a NodeIndex: {:?} to {:?}",
                    parent_ix, ctype);
                }
                if self.tree.can_remove_empty_parent(parent_ix) {
                    if let Err(err) = self.remove_view_or_container(parent_ix) {
                        error!("{:#?}\nCould not remove {:#?} from tree {:#?}", err, parent_ix, self);
                        panic!("Could not remove empty parent!");
                    }
                }
            }
            else {
                self.focus_on_next_container(curr_work_ix);
            }

            self.tree.set_family_visible(curr_work_ix, true);

            if !self.tree[active_ix].floating() {
                self.normalize_container(active_ix);
            }
        }
        let root_ix = self.tree.root_ix();
        self.layout(root_ix);
        self.validate();
    }
}
