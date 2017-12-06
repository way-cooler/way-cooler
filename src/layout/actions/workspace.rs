use rustwlc::WlcOutput;
use petgraph::graph::NodeIndex;
use uuid::Uuid;
use super::super::{LayoutTree, TreeError, FocusError};
use ::layout::core::container::{Container, ContainerType, Layout, Handle};
use ::layout::core::borders::Borders;
use ::debug_enabled;

// TODO This module needs to be updated like the other modules...
// Need to add some errors for this (such as when trying to move a non-container/view,
// or when trying to grab a workspace whos name already exists)
//
// Also the code is generally pretty crap, cause it's pretty old (mid-2016)

impl LayoutTree {
    /// Gets a workspace by name or creates it
    fn get_or_make_workspace(&mut self, name: &str) -> NodeIndex {
        let active_index = self.tree.follow_path_until(self.tree.root_ix(),
                                                      ContainerType::Output)
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
        let geometry = self.tree.get(output_ix)
            .expect("init_workspace: invalid output").get_geometry()
            .expect("init_workspace: no geometry for output");
        let worksp = Container::new_workspace(name.to_string(), geometry);
        let output_handle = match self.tree[output_ix].get_handle() {
            Ok(Handle::Output(output)) => output,
            Err(err) => panic!("Could not get handle from output: {:#?}", err),
            _ => unreachable!()
        };

        trace!("Adding workspace {:?}", worksp);
        let worksp_ix = self.tree.add_child(output_ix, worksp, false);
        let borders = Borders::make_root_borders(geometry, output_handle);
        let container = Container::new_container(geometry,
                                                 output_handle,
                                                 borders);
        let container_ix = self.tree.add_child(worksp_ix, container, false);
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
        {
            // Update the border colors
            let container = &mut self.tree[active_ix];
            container.clear_border_color()
                .expect("Could not clear old active border color");
            container.draw_borders().expect("Could not draw borders");
        }
        let old_worksp_parent_ix = self.tree.parent_of(old_worksp_ix)
            .expect("Old workspace had no parent");
        let new_worksp_parent_ix = self.tree.parent_of(workspace_ix)
            .expect("New workspace had no parent");
        // Only set the old one to be invisible if new and old share output.
        if new_worksp_parent_ix == old_worksp_parent_ix {
            // Set the old one to invisible
            self.set_container_visibility(old_worksp_ix, false);
        } else {
            // Set all views on the target output to be invisible,
            // to clear the old workspace visibilty out.
            self.set_container_visibility(new_worksp_parent_ix, false);
        }
        // Set the new one to visible
        self.container_visibilty_wrapper(workspace_ix, true);
        // Focus on the new output
        match self.tree[new_worksp_parent_ix] {
            Container::Output { handle, .. } => {
                WlcOutput::focus(Some(handle))
            },
            _ => unreachable!()
        }
        // Delete the old workspace if it has no views on it
        self.active_container = None;
        if self.tree.descendant_of_type(old_worksp_ix, ContainerType::View).is_err() {
            let siblings = self.tree.children_of(old_worksp_parent_ix);
            // Only remove if it's **NOT** the only workspace on the output.
            // AND if the new workspace is on the same output.
            if siblings.len() > 1 && old_worksp_parent_ix == new_worksp_parent_ix {
                trace!("Removing workspace: {:?}", self.tree[old_worksp_ix].get_name()
                    .expect("Workspace had no name"));
                if let Err(err) = self.remove_workspace(old_worksp_ix) {
                    warn!("Tried to remove empty workspace {:#?}, error: {:?}",
                        old_worksp_ix, err);
                    info!("{:#?}", self);
                    panic!("Could not remove old workspace");
                }
            }
        }
        workspace_ix = self.tree.workspace_ix_by_name(name)
            .expect("Workspace we just made was deleted!");
        let active_ix = self.tree.follow_path(workspace_ix);
        match self.tree[active_ix].get_type() {
            ContainerType::View  => {
                match self.tree[active_ix] {
                    Container::View { id, ..} => {
                        self.focus_on(id).unwrap_or_else(|_| {
                            warn!("Could not focus on {:?}", id);
                        });
                    },
                    _ => unreachable!()
                }
                // TODO Propogate this when this is refactored
                match self.set_active_node(active_ix) {
                    Err(TreeError::Focus(
                        FocusError::BlockedByFullscreen(_, focus_id))) => {
                        // If blocked, didn't get a chance to set it
                        self.active_container = self.tree.lookup_id(focus_id);
                        Ok(())
                    },
                    other => other
                }.expect("Could not set new active node");
                self.tree.set_ancestor_paths_active(active_ix);
                self.layout(workspace_ix);
                self.validate();
                self.validate_path();
                return;
            },
            _ => {
                self.active_container = self.tree
                    .descendant_of_type(active_ix, ContainerType::View)
                    .or_else(|_| self.tree.descendant_of_type(active_ix,
                                                              ContainerType::Container)).ok();
                match self.tree[self.active_container.expect("Workspace had NO children!")] {
                    Container::View { .. } => {
                        self.tree.set_ancestor_paths_active(self.active_container.unwrap());
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
        self.layout(workspace_ix);
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
        // TODO Need to not make it default, but need to add tests to make
        // sure that doesn't cause a regression.
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
            self.set_container_visibility(curr_work_ix, false);
            let new_output_ix = self.tree.parent_of(next_work_ix)
                .expect("Target workspace had no parent");
            match self.tree[new_output_ix] {
                Container::Output { handle: output_handle, .. } => {
                    fn set_output_recurse(this: &mut LayoutTree,
                                          node_ix: NodeIndex,
                                          output_handle: WlcOutput) {
                        match this.tree[node_ix].get_type() {
                            ContainerType::View => {
                                this.tree[node_ix].update_border_output(output_handle)
                                    .expect("Could not update border output for view");
                                // TODO this is duplicated in other places,
                                // abstract into a function somewhere (not here)
                                {
                                    // Update the border colors
                                    let container = &mut this.tree[node_ix];
                                    container.clear_border_color()
                                        .expect("Could not clear old active border color");
                                    container.draw_borders().expect("Could not draw borders");
                                }
                            },
                            ContainerType::Container => {
                                this.tree[node_ix].update_border_output(output_handle)
                                    .expect("Could not update border output for view");
                                {
                                    // Update the border colors
                                    let container = &mut this.tree[node_ix];
                                    container.clear_border_color()
                                        .expect("Could not clear old active border color");
                                    container.draw_borders().expect("Could not draw borders");
                                }
                                for child_ix in this.tree.children_of(node_ix) {
                                    set_output_recurse(this, child_ix, output_handle)
                                }
                            },
                            _ => unreachable!()
                        }
                    }
                    set_output_recurse(self, active_ix, output_handle);
                },
                _ => unreachable!()
            }

            // Save the parent of this view for focusing
            let maybe_active_parent = self.tree.parent_of(active_ix);

            // Get the root container of the next workspace
            let next_work_children = self.tree.children_of(next_work_ix);
            if cfg!(debug_assertions) || !debug_enabled() {
                assert!(next_work_children.len() == 1,
                        "Next workspace has multiple roots!");
            }
            let next_work_root_ix = next_work_children[0];

            // Move the container
            info!("Moving container {:?} to workspace {}",
                self.get_active_container(), name);
            self.tree.move_node(active_ix, next_work_root_ix);

            // If different outputs, show it on the new output.
            let cur_output_ix = self.tree.parent_of(curr_work_ix)
                .expect("Couldn't get parent of current work index");
            if new_output_ix != cur_output_ix {
                self.container_visibilty_wrapper(new_output_ix, true);
            }

            // If it's a fullscreen app, then update the fullscreen lists
            self.transfer_fullscreen(curr_work_ix, next_work_ix, id);

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
            self.container_visibilty_wrapper(curr_work_ix, true);
            if !self.tree[active_ix].floating() {
                self.normalize_container(active_ix).ok();
            }
        }
        let root_ix = self.tree.root_ix();
        self.layout(root_ix);
        self.validate();
        self.validate_path();
    }

    /// Transfers a fullscreen app from this workspace to another.
    fn transfer_fullscreen(&mut self, cur_work_ix: NodeIndex, next_work_ix: NodeIndex,
                           fullscreen_id: Uuid) {
        if let Some(fullscreen_ids) = self.tree[cur_work_ix].fullscreen_c() {
            if !fullscreen_ids.iter().any(|id| *id == fullscreen_id) {
                return;
            }
        } else {
            return;
        }
        self.tree[cur_work_ix].update_fullscreen_c(fullscreen_id, false)
            .expect("cur_work_ix was not a workspace");
        self.tree[next_work_ix].update_fullscreen_c(fullscreen_id, true)
            .expect("next_work_ix was not a workspace");
    }

    /// Wrapper around `set_container_visibility`, so that tabbed/stacked
    /// is handled correctly (i.e, it's visibilty checks are skipped).
    fn container_visibilty_wrapper(&mut self, node_ix: NodeIndex, val: bool) {
        let mut set = false;
        match self.tree[node_ix] {
            Container::Container { layout, .. } => {
                match layout {
                    Layout::Tabbed | Layout::Stacked => {
                        if val {
                            set = true;
                        }
                    },
                    _ => {}
                }
            },
            _ => {}
        }
        if !set {
            self.tree[node_ix].set_visibility(val);
            for child in self.tree.children_of(node_ix) {
                self.container_visibilty_wrapper(child, val)
            }
        } else {
            self.tree.next_active_node(node_ix)
                .map(|node| self.container_visibilty_wrapper(node, val));
        }
    }

    /// Gets the current workspace we are focused on
    ///
    /// If a workspace is not found, this is considered a hard error and
    /// it will panic.
    #[allow(dead_code)]
    pub fn current_workspace(&self) -> Result<&str, TreeError> {
        let active_ix = self.active_container
            .ok_or(TreeError::NoActiveContainer)?;
        let workspace_ix = self.tree.ancestor_of_type(active_ix,
                                                      ContainerType::Workspace)?;
        Ok(self.tree[workspace_ix].get_name()
           .expect("workspace_ix didn't point to a workspace!"))
    }
}

#[cfg(test)]
mod tests {
    use ::layout::core::tree::tests::basic_tree;

    #[test]
    pub fn switch_empty_workspaces() {
        let mut tree = basic_tree();
        tree.switch_to_workspace("5");
        tree.switch_to_workspace("4");
        tree.switch_to_workspace("5");
        tree.switch_to_workspace("4");
        tree.switch_to_workspace("2");
    }
}
