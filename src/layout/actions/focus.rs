use petgraph::graph::NodeIndex;

use rustwlc::WlcView;

use super::super::LayoutTree;
use super::super::core::Direction;
use super::super::core::container::{Container, ContainerType, Handle, Layout};

impl LayoutTree {
    pub fn move_focus_recurse(&mut self, node_ix: NodeIndex, direction: Direction) -> Result<NodeIndex, ()> {
        match self.tree[node_ix].get_type() {
            ContainerType::View | ContainerType::Container => { /* continue */ },
            _ => return Err(())
        }
        let parent_ix = self.tree.parent_of(node_ix)
            .expect("Active ix had no parent");
        match self.tree[parent_ix] {
            Container::Container { layout, .. } => {
                match (layout, direction) {
                    (Layout::Horizontal, Direction::Left) |
                    (Layout::Horizontal, Direction::Right) |
                    (Layout::Vertical, Direction::Up) |
                    (Layout::Vertical, Direction::Down) => {
                        let siblings = self.tree.children_of(parent_ix);
                        let cur_index = siblings.iter().position(|node| {
                            *node == node_ix
                        }).expect("Could not find self in parent");
                        let maybe_new_index = match direction {
                            Direction::Right | Direction::Down => {
                                cur_index.checked_add(1)
                            }
                            Direction::Left  | Direction::Up => {
                                cur_index.checked_sub(1)
                            }
                        };
                        if maybe_new_index.is_some() &&
                            maybe_new_index.unwrap() < siblings.len() {
                                // There is a sibling to move to.
                                let new_index = maybe_new_index.unwrap();
                                let new_active_ix = siblings[new_index];
                                match self.tree[new_active_ix].get_type() {
                                    ContainerType::Container => {
                                        let path_ix = self.tree.follow_path(new_active_ix);
                                        // If the pat wasn't complete, find the first view and focus on that
                                        let node_ix = (self.tree.descendant_of_type(path_ix, ContainerType::View)).unwrap();
                                        let parent_ix = (self.tree.parent_of(node_ix)).unwrap();
                                        match self.tree[node_ix].get_type() {
                                            ContainerType::View | ContainerType::Container => {},
                                            _ => panic!("Following path did not lead to a container or a view!")
                                        }
                                        trace!("Moving to different view {:?} in container {:?}",
                                                self.tree[node_ix], self.tree[parent_ix]);
                                        return Ok(node_ix);
                                    },
                                    ContainerType::View => {
                                        trace!("Moving to other view {:?}", self.tree[new_active_ix]);
                                        return Ok(new_active_ix)
                                    },
                                    _ => unreachable!()
                                };
                            }
                    },
                    _ => { /* We are moving out of siblings, recurse */ }
                }
            }
            Container::Workspace { .. } => {
                return Err(());
            }
            _ => unreachable!()
        }
        let parent_ix = self.tree.parent_of(node_ix)
            .expect("Node had no parent");
        return self.move_focus_recurse(parent_ix, direction);
    }

    /// Updates the current active container to be the next container or view
    /// to focus on after the previous view/container was moved/removed.
    ///
    /// A new view will tried to be set, starting with the children of the
    /// parent node. If a view cannot be found there, it starts climbing the
    /// tree until either a view is found or the workspace is (in which case
    /// it set the active container to the root container of the workspace)
    pub fn focus_on_next_container(&mut self, mut parent_ix: NodeIndex) {
        while self.tree.node_type(parent_ix)
            .expect("Node not part of the tree") != ContainerType::Workspace {
            if let Some(view_ix) = self.tree.descendant_of_type_right(parent_ix,
                                                            ContainerType::View) {
                match self.tree[view_ix]
                                    .get_handle().expect("view had no handle") {
                    Handle::View(view) => view.focus(),
                    _ => panic!("View had an output handle")
                }
                trace!("Active container set to view at {:?}", view_ix);
                let id = self.tree[view_ix].get_id();
                self.set_active_container(id)
                    .expect("Could not set active container");
                return;
            }
            parent_ix = self.tree.ancestor_of_type(parent_ix,
                                                    ContainerType::Container)
                .unwrap_or_else(|| {
                    self.tree.ancestor_of_type(parent_ix, ContainerType::Workspace)
                        .expect("Container was not part of a workspace")
                });
        }
        // If this is reached, parent is workspace
        let container_ix = self.tree.children_of(parent_ix)[0];
        // set the workspace to be active
        match self.tree[parent_ix] {
            Container::Workspace {ref mut focused, .. }=> {
                *focused = true;
            },
            _ => unreachable!()
        }
        let root_c_children = self.tree.children_of(container_ix);
        if root_c_children.len() > 0 {
            let new_active_ix = self.tree.descendant_of_type(root_c_children[0],
                                                                ContainerType::View)
                .unwrap_or(root_c_children[0]);
            let id = self.tree[new_active_ix].get_id();
            self.set_active_container(id)
                .expect("Could not set active container");
            match self.tree[new_active_ix] {
                Container::View { ref handle, .. } => handle.focus(),
                _ => {}
            };
            return;
        }
        trace!("Active container set to container {:?}", container_ix);
        let id = self.tree[container_ix].get_id();
        self.set_active_container(id)
            .expect("Could not set active container");

        // Update focus to new container
        self.get_active_container().map(|con| match *con {
            Container::View { ref handle, .. } => handle.focus(),
            Container::Container { .. } => WlcView::root().focus(),
            _ => panic!("Active container not view or container!")
        });
    }
}
