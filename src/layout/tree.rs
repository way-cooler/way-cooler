//! Main module to handle the layout.
//! This is where the i3-specific code is.

use std::sync::{Mutex, MutexGuard, TryLockError};
use std::cmp;

use petgraph::graph::NodeIndex;

use layout::container::{Container, Handle, ContainerType, Layout};
use rustwlc::{WlcView, WlcOutput, Geometry, Point, Size, ResizeEdge};

use layout::graph_tree::Tree;

/// Error for trying to lock the tree
pub type TreeErr = TryLockError<MutexGuard<'static, LayoutTree>>;
/// Result for locking the tree
pub type TreeResult = Result<MutexGuard<'static, LayoutTree>, TreeErr>;

/* An example Tree:

      Root
        |
    ____|____
   /         \
   |         |
 Output    Output
   |         |
 Workspace   |
   |        / \
   |       /   \
   | Workspace Workspace
   |     |         |
   |  Container Container
 Container
    |
   / \
  /   \
  |    \
  |     \
  |      \
  |       \
 Container \
     |      \
   View    View
 */

/// The layout tree builds on top of the graph_tree.
///
/// There are various invariants that the tree upholds:
///
///   + Root
///       - There is only one Root
///       - The root can only have Outputs (monitors) as children
///   + Output
///       - An Output must have at least one Workspace associated with it
///       - An Output must be associated with a WlcOutput (real monitor)
///       - An Output can only have Workspaces as children
///   + Workspace
///       - A Workspace must have at least one Container, even if it doesn't
///         contain any views
///       - A Workspace can only have Containers as children
///   + Container
///       - A Container can only have other Containers or Views as children
///   + View
///       - A View must be associated with a WlcView
///       - A View cannot have any children
#[derive(Debug)]
pub struct LayoutTree {
    tree: Tree,
    active_container: Option<NodeIndex>
}

lazy_static! {
    static ref TREE: Mutex<LayoutTree> = {
        Mutex::new(LayoutTree {
            tree: Tree::new(),
            active_container: None
        })
    };
}

impl LayoutTree {
    /// Sets the active container by finding the node with the WlcView
    pub fn set_active_container(&mut self, handle: WlcView) {
        info!("Active container was: {:?}", self.active_container);
        if let Some(node_ix) = self.tree.descendant_with_handle(self.tree.root_ix(), &handle) {
            self.active_container = Some(node_ix);
        }
        info!("Active container is now: {:?}", self.active_container);
    }
    /// Gets the currently active container.
    pub fn get_active_container(&self) -> Option<&Container> {
        self.active_container.and_then(|ix| self.tree.get(ix))
    }

    /// Gets the currently active container.
    #[allow(dead_code)]
    pub fn get_active_container_mut(&mut self) -> Option<&mut Container> {
        self.active_container.and_then(move |ix| self.tree.get_mut(ix))
    }

    /// Gets the index of the currently active container with the given type.
    /// Starts at the active container, moves up until either a container with
    /// that type is found or the root node is hit
    fn active_ix_of(&self, ctype: ContainerType) -> Option<NodeIndex> {
        if let Some(ix) = self.active_container {
            if self.tree[ix].get_type() == ctype {
                return Some(ix)
            }
            return self.tree.ancestor_of_type(ix, ctype)
        }
        return None
    }

    #[allow(dead_code)]
    fn active_of(&self, ctype: ContainerType) -> Option<&Container> {
        self.active_ix_of(ctype).and_then(|ix| self.tree.get(ix))
    }

    #[allow(dead_code)]
    fn active_of_mut(&mut self, ctype: ContainerType) -> Option<&mut Container> {
        self.active_ix_of(ctype).and_then(move |ix| self.tree.get_mut(ix))
    }

    /// Gets the parent of the active container
    #[allow(dead_code)]
    pub fn get_active_parent(&self) -> Option<&Container> {
        if let Some(container_ix) = self.active_container {
            if let Some(parent_ix) = self.tree.parent_of(container_ix) {
                return Some(&self.tree[parent_ix]);
            }
        }
        return None
    }

    /// Gets the WlcOutput the active container is located on
    #[allow(dead_code)]
    pub fn get_active_output(&self) -> Option<&Container> {
        self.active_of(ContainerType::Output)
    }

    /// Gets the WlcOutput the active container is located on
    #[allow(dead_code)]
    pub fn get_active_output_mut(&mut self) -> Option<&mut Container> {
        self.active_of_mut(ContainerType::Output)
    }

    /// Gets the workspace the active container is located on
    #[allow(dead_code)]
    pub fn get_active_workspace(&self) -> Option<&Container> {
        self.active_of(ContainerType::Workspace)
    }

    /// Gets the workspace the active container is located on
    #[allow(dead_code)]
    pub fn get_active_workspace_mut(&mut self) -> Option<&mut Container> {
        self.active_of_mut(ContainerType::Workspace)
    }

    /// Gets the index of the workspace of this name
    ///
    /// TODO will search all outputs, probably should be more directed
    fn workspace_ix_by_name(&self, name: &str) -> Option<NodeIndex> {
        for output in self.tree.children_of(self.tree.root_ix()) {
            for workspace in self.tree.children_of(output) {
                if self.tree[workspace].get_name()
                    .expect("workspace_by_name: bad tree structure") == name {
                    return Some(workspace)
                }
            }
        }
        return None
    }

    /// Gets a workspace by name or creates it
    fn get_or_make_workspace(&mut self, name: &str) -> NodeIndex {
        let active_index = self.active_ix_of(ContainerType::Output).expect("get_or_make_wksp: Couldn't get output");
        self.workspace_ix_by_name(name).unwrap_or_else(|| {
            let root_ix = self.init_workspace(name.to_string(), active_index);
            self.tree.parent_of(root_ix)
                .expect("Workspace was not properly initialized with a root container")
        })
    }

    /// Initializes a workspace and gets the index of the root container
    fn init_workspace(&mut self, name: String, output_ix: NodeIndex)
                      -> NodeIndex {
        let size = self.tree.get(output_ix)
            .expect("init_workspace: invalid output").get_geometry()
            .expect("init_workspace: no geometry for output").size;
        let worksp = Container::new_workspace(name.to_string(), size.clone());

        trace!("Adding workspace {:?}", worksp);
        let worksp_ix = self.tree.add_child(output_ix, worksp);
        let geometry = Geometry {
            size: size, origin: Point { x: 0, y: 0 }
        };
        let container_ix = self.tree.add_child(worksp_ix,
                                           Container::new_container(geometry));
        container_ix
    }


    /// Make a new output container with the given WlcOutput.
    ///
    /// A new workspace is automatically added to the output, to ensure
    /// consistency with the tree. By default, it sets this new workspace to
    /// be workspace "1". This will later change to be the first available
    /// workspace if using i3-style workspaces.
    pub fn add_output(&mut self, output: WlcOutput) {
        trace!("Adding new output with {:?}", output);
        let root_index = self.tree.root_ix();
        let output_ix = self.tree.add_child(root_index,
                                          Container::new_output(output));
        self.active_container = Some(self.init_workspace("1".to_string(), output_ix));
        self.validate();
    }

    /// Normalizes the geometry of a view to be the same size as it's siblings,
    /// based on the parent container's layout, at the 0 point of the parent container.
    /// Note this does not auto-tile, only modifies this one view.
    ///
    /// Useful if a container's children want to be evenly distributed, or a new view
    /// is being added.
    pub fn normalize_view(&mut self, view: WlcView) {
        if let Some(view_ix) = self.tree.descendant_with_handle(self.tree.root_ix(), &view) {
            let parent_ix = self.tree.ancestor_of_type(view_ix,
                                                       ContainerType::Container)
                .expect("View had no container parent");
            match self.tree[parent_ix] {
                Container::Container { ref layout, .. } => {
                    match *layout {
                        Layout::Horizontal => {
                            let num_siblings = cmp::max(1, self.tree.children_of(parent_ix).len() - 1)
                                as u32;
                            let parent_geometry = self.tree[parent_ix].get_geometry()
                                .expect("Parent container had no geometry");
                            let new_geometry = Geometry {
                                origin: parent_geometry.origin.clone(),
                                size: Size {
                                    w: parent_geometry.size.w / num_siblings,
                                    h: parent_geometry.size.h
                                }
                            };
                            trace!("Setting view {:?} to geometry: {:?}",
                                   self.tree[view_ix], parent_geometry);
                            view.set_geometry(ResizeEdge::empty(), &new_geometry);
                        }
                        Layout::Vertical => {
                            let num_siblings = cmp::max(1, self.tree.children_of(parent_ix).len() - 1)
                                as u32;
                            let parent_geometry = self.tree[parent_ix].get_geometry()
                                .expect("Parent container had no geometry");
                            let new_geometry = Geometry {
                                origin: parent_geometry.origin.clone(),
                                size: Size {
                                    w: parent_geometry.size.w,
                                    h: parent_geometry.size.h / num_siblings
                                }
                            };
                            trace!("Setting view {:?} to geometry: {:?}",
                                   self.tree[view_ix], parent_geometry);
                            view.set_geometry(ResizeEdge::empty(), &new_geometry);
                        }
                        _ => unimplemented!()
                    }
                },
                _ => unreachable!()
            };
        }
    }

    /// Add a new view container with the given WlcView to the active container
    pub fn add_view(&mut self, view: WlcView) {
        if let Some(mut active_ix) = self.active_container {
            if self.tree[active_ix].get_type() == ContainerType::View {
                active_ix = self.tree.parent_of(active_ix)
                    .expect("View had no parent");
            }
            let view_ix = self.tree.add_child(active_ix,
                                              Container::new_view(view));
            self.active_container = Some(view_ix);
        }
        self.validate();
    }

    //// Remove a view container from the tree
    pub fn remove_view(&mut self, view: &WlcView) {
        if let Some(view_ix) = self.tree.descendant_with_handle(self.tree.root_ix(), view) {
            self.remove_view_or_container(view_ix);
        } else {
            warn!("Could not find descendant with handle {:#?} to remove", view);
        }
        self.validate();
    }

    /// Special code to handle removing a View or Container.
    /// We have to ensure that we aren't invalidating the active container
    /// when we remove a view or container.
    fn remove_view_or_container(&mut self, node_ix: NodeIndex) {
        // make sure this remains valid
        let parent_ix = self.tree.parent_of(node_ix)
            .expect("Container we are removing had no parent");
        if self.active_container.map(|c| c == node_ix).unwrap_or(false) {
            // Update the active container if needed
            if let Some(parent_index) = self.tree.ancestor_of_type(node_ix,
                                                                       ContainerType::Container) {
                // Remove the view from the tree
                self.tree.remove(node_ix);
                self.focus_on_next_container(parent_index);
            }
        } else {
            // If the active container is the last index in the node array,
            // its index will become this one.
            if let Some(mut active_ix) = self.active_container {
                if self.tree.is_last_ix(active_ix) {
                    active_ix = node_ix;
                }
                self.tree.remove(node_ix);
                self.active_container = Some(active_ix);
            }
        }
        // Remove parent container if it is a non-root container and has no other children
        match self.tree[parent_ix].get_type() {
            ContainerType::Container => {
                if self.is_root_container(parent_ix) {
                    return;
                }
                if self.tree.children_of(parent_ix).len() == 0 {
                    self.remove_view_or_container(parent_ix);
                }
            }
            _ => {},
        }
    }

    fn is_root_container(&self, node_ix: NodeIndex) -> bool {
        self.tree[self.tree.parent_of(node_ix).unwrap_or(node_ix)].get_type() == ContainerType::Workspace
    }

    /// Splits the active container vertically
    pub fn active_split_vertical(&mut self) {
        if self.is_root_container(self.active_container.expect("No active container")) {
            match self.tree[self.active_container.unwrap()] {
                Container::Container { ref mut layout, .. } =>
                    *layout = Layout::Vertical,
                _ => unreachable!()
            }
            return;
        }
        let active_geometry = self.get_active_container()
            .expect("Could not get the active container")
            .get_geometry().expect("Active container had no geometry");

        let mut new_container = Container::new_container(active_geometry);
        new_container.set_layout(Layout::Vertical).ok();
        let active_ix = self.active_container.unwrap();
        self.add_container(new_container, active_ix);
    }

    /// Splits the active container horizontally
    pub fn active_split_horizontal(&mut self) {
        if self.is_root_container(self.active_container.expect("No active container")) {
            match self.tree[self.active_container.unwrap()] {
                Container::Container { ref mut layout, .. } =>
                    *layout = Layout::Horizontal,
                _ => unreachable!()
            }
            return;
        }
        let active_geometry = self.get_active_container()
            .expect("Could not get the active container")
            .get_geometry().expect("Active container had no geometry");
        let mut new_container = Container::new_container(active_geometry);
        new_container.set_layout(Layout::Horizontal).ok();
        let active_ix = self.active_container.unwrap();
        self.add_container(new_container, active_ix);
    }

    /// Adds the container with the node index as a child. This is how split horizontal
    /// and split vertical is implemented. The node at the node index is removed and
    /// made a child of the new container node.
    fn add_container(&mut self, container: Container, child: NodeIndex) {
        let parent_ix = self.tree.parent_of(child)
            .expect("Node had no parent");
        if self.is_root_container(parent_ix) {
            if let Some(child_container) = self.tree.remove(child) {
                match container {
                    Container::Container { ref layout, .. } => {
                        self.tree[parent_ix].set_layout(layout.clone()).ok()
                    }
                    _ => unreachable!()
                };
                let new_active_ix = self.tree.add_child(parent_ix, child_container);
                self.active_container = Some(new_active_ix);
            }
        }
        if let Some(child_container) = self.tree.remove(child) {
            let new_container_ix = self.tree.add_child(parent_ix, container);
            let new_active_ix = self.tree.add_child(new_container_ix, child_container);
            self.active_container = Some(new_active_ix);
        }
    }

    /// Remove a  container from the tree.
    /// The active container is preserved after this operation,
    /// if it was moved then it's new index will be reflected in the structure
    ///
    /// Note that because this causes N indices to be changed (where N is the
    /// number of children of the container), any node indices should be
    /// considered invalid after this operation (except for the active_container)
    fn remove_container(&mut self, container_ix: NodeIndex) {
        let mut children = self.tree.all_descendants_of(&container_ix);
        // add current container to the list as well
        children.push(container_ix);
        // Sort by highest to lowest
        children.sort_by(|a, b| b.cmp(a));
        for node_ix in children {
            trace!("Removing index {:?}: {:?}", node_ix, self.tree[node_ix]);
            match self.tree[node_ix] {
                Container::View { .. } | Container::Container { .. } => {
                    self.remove_view_or_container(node_ix);
                },
                _ => { self.tree.remove(node_ix); },
            }
        }
        self.validate();
    }

    /// Updates the current active container to be the next container or view
    /// to focus on after the previous view/container was moved/removed.
    ///
    /// A new view will tried to be set, starting with the siblings of the
    /// removed node. If a view cannot be found there, it starts climbing the
    /// tree until either a view is found or the workspace is (in which case
    /// it set the active container to the root container of the workspace)
    ///
    /// Parent should be the parent of the node that was destroyed.
    /// Note that the node should be destroyed already, otherwise this algorithm
    /// will simply relocate the node to be destroyed and set it to be the active
    /// container.
    fn focus_on_next_container(&mut self, mut parent_ix: NodeIndex) {
        while self.tree.node_type(parent_ix)
            .expect("focus_next: unable to iterate") != ContainerType::Workspace {
            if let Some(view_ix) = self.tree.descendant_of_type(parent_ix,
                                                           ContainerType::View) {
                match self.tree[view_ix]
                                    .get_handle().expect("view had no handle") {
                    Handle::View(view) => view.focus(),
                    _ => panic!("View had an output handle")
                }
                trace!("Active container set to view at {:?}", view_ix);
                self.active_container = Some(view_ix);
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
        let mut container_ix = self.tree.children_of(parent_ix)[0];
        if let Some(view_ix) = self.tree.children_of(container_ix).get(0) {
            container_ix = *view_ix;
        }
        trace!("Active container set to container {:?}", container_ix);
        self.active_container = Some(container_ix);

        // Update focus to new container
        self.get_active_container().map(|con| match *con {
            Container::View { ref handle, .. } => handle.focus(),
            Container::Container { .. } => WlcView::root().focus(),
            _ => panic!("Active container not view or container!")
        });

        self.validate();
    }

    // Updates the tree's layout recursively starting from the root.
    // This is very expensive, since it traverses the entire tree.
    #[allow(dead_code)]
    pub fn update_layout(&mut self) {
        let root_ix = self.tree.root_ix();
        self.layout(root_ix);
    }

    // Updates the tree's layout recursively starting from the active container.
    // If the active container is a view, it starts at the parent container.
    pub fn update_active_of(&mut self, c_type: ContainerType) {
        if let Some(container_ix) = self.active_ix_of(c_type) {
            match self.tree[container_ix].clone() {
                Container::Root { .. } |
                Container::Output { .. } |
                Container::Workspace { .. } => {
                    self.layout(container_ix);
                }
                Container::Container { ref geometry, .. } => {
                    self.layout_helper(container_ix, geometry.clone());
                },
                Container::View { .. } => {
                    warn!("Cannot simply update a view's geometry without {}",
                          "consulting container, updating it's parent");
                    self.update_active_of(ContainerType::Container);
                },

            }
        } else {
            error!("{:?} did not have a parent of type {:?}, doing nothing!",
                   self, c_type);
        }
    }

    /// Given the index of some container in the tree, lays out the children of
    /// that container based on what type of container it is and how big of an
    /// area is allocated for it and its children.
    fn layout(&mut self, node_ix: NodeIndex) {
        match self.tree[node_ix].get_type() {
            ContainerType::Root => {
                for output_ix in self.tree.children_of(node_ix) {
                    self.layout(output_ix);
                }
            }
            // NOTE Do we need to set the workspace to the size of the output...?
            // We should do that here, right?
            ContainerType::Output => {
                // Workspace doesn't care about how big the output actually is
                let handle = match self.tree[node_ix] {
                    Container::Output { ref handle, .. } => handle.clone(),
                    _ => unreachable!()
                };
                let size = handle.get_resolution().clone();
                let geometry = Geometry {
                    origin: Point { x: 0, y: 0 },
                    size: size
                };
                for workspace_ix in self.tree.children_of(node_ix) {
                    self.layout_helper(workspace_ix, geometry.clone());
                }
            }
            ContainerType::Workspace => {
                // Simply call layout_helper with the geometry from the parent output
                let output_ix = self.tree.ancestor_of_type(node_ix, ContainerType::Output)
                    .expect("Workspace had no output parent");
                let handle = match self.tree[output_ix] {
                    Container::Output{ ref handle, .. } => handle.clone(),
                    _ => unreachable!()
                };
                let output_geometry = Geometry {
                    origin: Point { x: 0, y: 0},
                    size: handle.get_resolution().clone()
                };
                trace!("layout: Laying out workspace, using size of the screen output {:?}", handle);
                self.layout_helper(node_ix, output_geometry);
            }
            _ => panic!("layout should not be called directly on a container, view")
        }
    }

    fn layout_helper(&mut self, node_ix: NodeIndex, geometry: Geometry) {
        match self.tree[node_ix].get_type() {
            ContainerType::Workspace => {
                // NOTE Here is where we deal with the gap for the panel
                {
                    let container_mut = self.tree.get_mut(node_ix).unwrap();
                    trace!("layout_helper: Laying out workspace {:?} with geometry constraints {:?}",
                           container_mut, geometry);
                    match *container_mut {
                        Container::Workspace { ref mut size, .. } => {
                            *size = geometry.size.clone();
                        }
                        _ => unreachable!()
                    };
                }
                for child_ix in self.tree.children_of(node_ix) {
                    self.layout_helper(child_ix, geometry.clone());
                }
            },
            ContainerType::Root | ContainerType::Output => {
                trace!("layout_helper: Laying out entire tree");
                warn!("Ignoring geometry constraint ({:?}), deferring to each output's constraints",
                      geometry);
                for child_ix in self.tree.children_of(node_ix) {
                    self.layout(child_ix);
                }
            }
            ContainerType::Container => {
                {
                    let container_mut = self.tree.get_mut(node_ix).unwrap();
                    trace!("layout_helper: Laying out container {:?} with geometry constraints {:?}",
                           container_mut, geometry);
                    match *container_mut {
                        Container::Container { geometry: ref mut c_geometry, .. } => {
                            *c_geometry = geometry.clone();
                        },
                        _ => unreachable!()
                    };
                }
                let layout = match self.tree[node_ix] {
                    Container::Container { layout, .. } => layout,
                    _ => unreachable!()
                };
                match layout {
                    Layout::Horizontal => {
                        trace!("Layout was horizontal, laying out the sub-containers horizontally");
                        let children = self.tree.children_of(node_ix);
                        let mut scale = LayoutTree::calculate_scale(children.iter().map(|child_ix| {
                            let c_geometry = self.tree[*child_ix].get_geometry()
                                .expect("Child had no geometry");
                            c_geometry.size.w as f32
                        }).collect(), geometry.size.w as f32);

                        if scale > 0.1 {
                            scale = geometry.size.w as f32 / scale;
                            let new_size_f = |child_size: Size, sub_geometry: Geometry| {
                                Size {
                                    w: ((child_size.w as f32) * scale) as u32,
                                    h: sub_geometry.size.h
                                }
                            };
                            let remaining_size_f = |sub_geometry: Geometry,
                                                    cur_geometry: Geometry| {
                                let remaining_width =
                                    cur_geometry.origin.x as u32 + cur_geometry.size.w -
                                    sub_geometry.origin.x as u32;
                                Size {
                                    w: remaining_width,
                                    h: sub_geometry.size.h
                                }
                            };
                            let new_point_f = |new_size: Size, sub_geometry: Geometry| {
                                Point {
                                    x: sub_geometry.origin.x + new_size.w as i32,
                                    y: sub_geometry.origin.y
                                }
                            };
                            self.generic_tile(node_ix, geometry, scale, children,
                                              new_size_f, remaining_size_f, new_point_f);
                        }
                    }
                    Layout::Vertical => {
                        trace!("Layout was vertical, laying out the sub-containers vertically");
                        let children = self.tree.children_of(node_ix);
                        let mut scale = LayoutTree::calculate_scale(children.iter().map(|child_ix| {
                            let c_geometry = self.tree[*child_ix].get_geometry()
                                .expect("Child had no geometry");
                            c_geometry.size.h as f32
                        }).collect(), geometry.size.h as f32);

                        if scale > 0.1 {
                            scale = geometry.size.h as f32 / scale;
                            let new_size_f = |child_size: Size, sub_geometry: Geometry| {
                                Size {
                                    w: sub_geometry.size.w,
                                    h: ((child_size.h as f32) * scale) as u32
                                }
                            };
                            let remaining_size_f = |sub_geometry: Geometry,
                                                    cur_geometry: Geometry| {
                                let remaining_height =
                                    cur_geometry.origin.y as u32 + cur_geometry.size.h -
                                    sub_geometry.origin.y as u32;
                                Size {
                                    w: sub_geometry.size.w,
                                    h: remaining_height
                                }
                            };
                            let new_point_f = |new_size: Size, sub_geometry: Geometry| {
                                Point {
                                    x: sub_geometry.origin.x,
                                    y: sub_geometry.origin.y + new_size.h as i32
                                }
                            };
                            self.generic_tile(node_ix, geometry, scale, children,
                                              new_size_f, remaining_size_f, new_point_f);
                        }
                    }
                    Layout::Floating => {
                        trace!("Layout was floating, throwing the views on the screen {}",
                               "like I'm Jackson Pollock");
                    }
                    _ => unimplemented!()
                }
            }

            ContainerType::View => {
                let handle = match self.tree[node_ix] {
                    Container::View { ref handle, .. } => handle,
                    _ => unreachable!()
                };
                trace!("layout_helper: Laying out view {:?} with geometry constraints {:?}",
                       handle, geometry);
                handle.set_geometry(ResizeEdge::empty(), &geometry);
                // yeahhhh I think I need to do something else?
                // Probably with geometry
                //self.update_geometry(node_ix);
                return;
            }
        }
    }

    /// Calculates how much to scale on average for each value given.
    /// If the value is 0 (i.e the width or height of the container is 0),
    /// then it is calculated as max / children_values.len()
    fn calculate_scale(children_values: Vec<f32>, max: f32) -> f32 {
        let mut scale = 0.0;
        let len = children_values.len();
        for mut value in children_values {
            if value <= 0.0 {
                value = max / cmp::max(1, len - 1) as f32;
            }
            scale += value;
        }
        return scale;
    }

    fn generic_tile<SizeF, RemainF, PointF>
        (&mut self,
         node_ix: NodeIndex, geometry: Geometry, scale: f32, children: Vec<NodeIndex>,
         new_size_f: SizeF, remaining_size_f: RemainF, new_point_f: PointF)
        where SizeF:   Fn(Size, Geometry) -> Size,
              RemainF: Fn(Geometry, Geometry) -> Size,
              PointF:  Fn(Size, Geometry) -> Point
    {
        trace!("Scaling factor: {:?}", scale);
        let mut sub_geometry = geometry.clone();
        for (index, child_ix) in children.iter().enumerate() {
            let child_size: Size;
            {
                let child = &self.tree[*child_ix];
                child_size = child.get_geometry()
                    .expect("Child had no geometry").size;
            }
            let new_size = new_size_f(child_size, sub_geometry.clone());
            sub_geometry = Geometry {
                origin: sub_geometry.origin.clone(),
                size: new_size.clone()
            };
            // If last child, then just give it the remaining height
            if index == children.len() - 1 {
                let new_size = remaining_size_f(sub_geometry.clone(),
                                                self.tree[node_ix].get_geometry()
                                                .expect("Container had no geometry"));
                sub_geometry = Geometry {
                    origin: sub_geometry.origin,
                    size: new_size
                };
            }
            self.layout_helper(*child_ix, sub_geometry.clone());

            // Next sub container needs to start where this one ends
            let new_point = new_point_f(new_size.clone(), sub_geometry.clone());
            sub_geometry = Geometry {
                // lambda to calculate new point, given a new size
                // which is calculated in the function
                origin: new_point,
                size: new_size
            };
        }
    }

    /// Gets the active container and toggles it based on the following rules:
    /// * If horizontal, make it vertical
    /// * else, make it horizontal
    /// This method does *NOT* update the actual views geometry, that needs to be
    /// done separately by the caller
    pub fn toggle_active_horizontal(&mut self) {
        if let Some(active_ix) = self.active_ix_of(ContainerType::Container) {
            trace!("Toggling {:?} to be horizontal or vertical...", self.tree[active_ix]);
            match self.tree[active_ix] {
                Container::Container { ref mut layout, .. } => {
                    match *layout {
                        Layout::Horizontal => {
                            trace!("Toggling to be vertical");
                            *layout = Layout::Vertical
                        }
                        _ => {
                            trace!("Toggling to be horizontal");
                            *layout = Layout::Horizontal
                        }
                    }
                },
                _ => unreachable!()
            }
        }
        error!("No active container")
    }

    /// Switch to the specified workspace
    pub fn switch_to_workspace(&mut self, name: &str) {
        if self.active_container.is_none() {
            warn!("No active container, cannot switch");
            return;
        }
        // Set old workspace to be invisible
        let old_worksp_ix: NodeIndex;
        if let Some(index) = self.active_ix_of(ContainerType::Workspace) {
            old_worksp_ix = index;
            trace!("Switching to workspace {}", name);
            self.tree.set_family_visible(old_worksp_ix, false);
        } else {
            warn!("Could not find old workspace, could not set invisible");
            return;
        }
        // Get the new workspace, or create one if it doesn't work
        let mut workspace_ix = self.get_or_make_workspace(name);
        if old_worksp_ix == workspace_ix {
            return;
        }
        // Set the new one to visible
        self.tree.set_family_visible(workspace_ix, true);
        // Delete the old workspace if it has no views on it
        self.active_container = None;
        if self.tree.descendant_of_type(old_worksp_ix, ContainerType::View).is_none() {
            trace!("Removing workspace: {:?}", self.tree[old_worksp_ix].get_name()
                   .expect("Workspace had no name"));
            self.remove_container(old_worksp_ix);
        }
        workspace_ix = self.workspace_ix_by_name(name)
            .expect("Workspace we just made was deleted!");
        self.focus_on_next_container(workspace_ix);
        self.validate();
    }

    /// Moves the current active container to a new workspace
    pub fn send_active_to_workspace(&mut self, name: &str) {
        // Ensure focus
        if self.active_container.is_none() {
            return;
        }

        let active_ix = self.active_container.expect("Asserted unwrap");
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

        // Get active
        if cfg!(debug_assertions) {
            let work_children = self.tree.children_of(curr_work_ix);
            assert!(work_children.len() != 0, "Workspace has no children");
            assert!(match self.tree[active_ix].get_type() {
                ContainerType::Container|ContainerType::View => true,
                _ => false
            }, "Invalid workspace switch type!");
        }
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
        if let Some(parent) = maybe_active_parent {
            let ctype = self.tree.node_type(parent).unwrap_or(ContainerType::Root);
            if ctype == ContainerType::Container {
                self.focus_on_next_container(parent);
            } else {
                trace!("Send to container invalidated a NodeIndex: {:?} to {:?}",
                parent, ctype);
            }
        }
        else {
            self.focus_on_next_container(curr_work_ix);
        }

        self.tree.set_family_visible(curr_work_ix, true);
        self.validate();
    }

    /// Validates the tree
    #[cfg(debug_assertions)]
    fn validate(&self) {
        warn!("Validating the tree");

        // Recursive method to ensure child/parent nodes are connected
        fn validate_node_connections(this: &LayoutTree, parent_ix: NodeIndex) {
            for child_ix in this.tree.children_of(parent_ix) {
                let child_parent = this.tree.parent_of(child_ix)
                    .expect("connections: Child did not point to parent!");
                if child_parent != parent_ix {
                    error!("Child at {:?} has parent {:?}, expected {:?}",
                           child_ix, child_parent, parent_ix);
                    trace!("The tree: {:#?}", this);
                    panic!()
                }
                validate_node_connections(this, child_ix);
            }
        }

        validate_node_connections(self, self.tree.root_ix());

        // For each view, ensure it's a node in the tree
        for output_ix in self.tree.children_of(self.tree.root_ix()) {
            let output = match self.tree[output_ix]
                .get_handle().expect("Child of root had no output") {
                    Handle::Output(output) => output,
                    _ => panic!("Output container had no output")
                };
            for view in output.get_views() {
                if self.tree.descendant_with_handle(output_ix, &view).is_none() {
                    error!("View handle {:?} could not be found for {:?}",
                           view, output);
                    trace!("The tree: {:#?}", self);
                    panic!()
                }
            }
        }

        // Ensure active container is in tree and of right type
        if let Some(active_ix) = self.active_container {
            let active = self.get_active_container()
                .expect("active_container points to invalid node");
            match active.get_type() {
                ContainerType::View | ContainerType::Container => {},
                _ => panic!("Active container was not view or container")
            }
            // Check active container in tree
            if self.tree.ancestor_of_type(active_ix, ContainerType::Root).is_none() {
                error!("Active container @ {:?} is not part of tree!", active_ix);
                error!("Active container is {:?}", active);
                trace!("The tree: {:#?}", self);
                panic!()
            }
        }

        // Ensure workspace have at least one child
        for output_ix in self.tree.children_of(self.tree.root_ix()) {
            for workspace_ix in self.tree.children_of(output_ix) {
                if self.tree.children_of(workspace_ix).len() == 0 {
                    error!("Workspace {:#?} has no children",
                           self.tree[workspace_ix]);
                    trace!("The tree: {:#?}", self);
                    panic!()
                }
            }
        }
    }

    #[cfg(not(debug_assertions))]
    fn validate(&self) {}

}

/// Attempts to lock the tree. If the Result is Err, then a thread that
/// previously had the lock panicked and potentially left the tree in a bad state
pub fn try_lock_tree() -> TreeResult {
    trace!("Locking the tree!");
    TREE.try_lock()
}


#[cfg(test)]
mod tests {

    use super::super::graph_tree::Tree;
    use super::*;
    use layout::container::*;
    use rustwlc::*;

    /// Makes a very basic tree.
    /// There is only one output,
    /// Two workspaces,
    /// First workspace has a single view in the root container,
    /// second workspace has a container with two views in it
    /// (the container is a child of the root container).
    ///
    /// The active container is the only view in the first workspace
    #[allow(unused_variables)]
    fn basic_tree() -> LayoutTree {
        let mut tree = Tree::new();
        let fake_view_1 = WlcView::root();
        let fake_output = fake_view_1.clone().as_output();
        let root_ix = tree.root_ix();
        let fake_size = Size { h: 800, w: 600 };
        let fake_geometry = Geometry {
            size: fake_size.clone(),
            origin: Point { x: 0, y: 0 }
        };

        let output_ix = tree.add_child(root_ix, Container::new_output(fake_output));
        let workspace_1_ix = tree.add_child(output_ix,
                                                Container::new_workspace("1".to_string(),
                                                                   fake_size.clone()));
        let root_container_1_ix = tree.add_child(workspace_1_ix,
                                                Container::new_container(fake_geometry.clone()));
        let workspace_2_ix = tree.add_child(output_ix,
                                                Container::new_workspace("2".to_string(),
                                                                     fake_size.clone()));
        let root_container_2_ix = tree.add_child(workspace_2_ix,
                                                Container::new_container(fake_geometry.clone()));
        /* Workspace 1 containers */
        let wkspc_1_view = tree.add_child(root_container_1_ix,
                                                Container::new_view(fake_view_1.clone()));
        /* Workspace 2 containers */
        let wkspc_2_container = tree.add_child(root_container_2_ix,
                                                Container::new_container(fake_geometry.clone()));
        let wkspc_2_sub_view_1 = tree.add_child(wkspc_2_container,
                                                Container::new_view(fake_view_1.clone()));
        let wkspc_2_sub_view_2 = tree.add_child(wkspc_2_container,
                                                Container::new_view(fake_view_1.clone()));

        let layout_tree = LayoutTree {
            tree: tree,
            active_container: Some(wkspc_1_view)
        };
        layout_tree
    }

    #[test]
    /// Ensures that getting the active container always returns either
    /// a view, a container, or nothing.
    fn active_container_tests() {
        let mut simple_tree = basic_tree();
        /* Standard active_container getters */
        {
            let active_container = simple_tree.get_active_container().unwrap();
            let view_ancestor_ix = simple_tree.active_ix_of(ContainerType::View).unwrap();
            assert_eq!(*active_container, simple_tree.tree[view_ancestor_ix]);
            match *active_container {
                Container::View { .. }| Container::Container { .. }=> {},
                _ => panic!("Active container was not a view or container!")
            }
        }
        {
            let active_container_mut = simple_tree.get_active_container_mut().unwrap();
            match *active_container_mut {
                Container::View { .. }| Container::Container { .. }=> {},
            _ => panic!("Active container was not a view or container!")
            }
        }
        /* Active workspace getters */
        {
            let active_workspace = simple_tree.get_active_workspace().unwrap();
            let workspace_ancestor_ix = simple_tree.active_ix_of(ContainerType::Workspace).unwrap();
            assert_eq!(*active_workspace, simple_tree.tree[workspace_ancestor_ix]);
            match *active_workspace {
                Container::Workspace { ref name, .. } => {
                    assert_eq!(name.as_str(), "1")
                },
                _ => panic!("get_active_workspace did not return a workspace")
            }
        }
        {
            let active_workspace_mut = simple_tree.get_active_workspace_mut().unwrap();
            match *active_workspace_mut {
                Container::Workspace { ref name, .. } => {
                assert_eq!(name.as_str(), "1")
            },
            _ => panic!("get_active_workspace did not return a workspace")
            }
        }
        /* Active output getters */
        {
            let active_output = simple_tree.get_active_output().unwrap();
            let output_ancestor_ix = simple_tree.active_ix_of(ContainerType::Output).unwrap();
            assert_eq!(*active_output, simple_tree.tree[output_ancestor_ix]);
            match *active_output {
                Container::Output { ref handle, .. } => {
                    assert_eq!(*handle, WlcView::root().as_output());
                }
                _ => panic!("get_active_output did not return an output")
            }
        }
        {
            let active_output_mut = simple_tree.get_active_output_mut().unwrap();
            match *active_output_mut {
                Container::Output { ref handle, .. } => {
                    assert_eq!(*handle, WlcView::root().as_output());
                }
                _ => panic!("get_active_output did not return an output")
            }
        }
    }

    #[test]
    /// Tests workspace functions, ensuring we can get workspaces and new
    /// ones are properly generated with a root container when we request one
    /// that doesn't yet exist
    fn workspace_tests() {
        let mut tree = basic_tree();
        /* Simple workspace access tests */
        let workspace_1_ix = tree.workspace_ix_by_name("1")
            .expect("Workspace 1 did not exist");
        assert_eq!(tree.tree[workspace_1_ix].get_type(), ContainerType::Workspace);
        assert_eq!(tree.tree[workspace_1_ix].get_name().unwrap(), "1");
        let workspace_2_ix = tree.workspace_ix_by_name("2")
            .expect("Workspace 2 did not exist");
        assert_eq!(tree.tree[workspace_2_ix].get_type(), ContainerType::Workspace);
        assert_eq!(tree.tree[workspace_2_ix].get_name().unwrap(), "2");
        assert!(tree.workspace_ix_by_name("3").is_none(),
                "Workspace three existed, expected it not to");
        /* init workspace tests */
        let output_ix = tree.active_ix_of(ContainerType::Output)
            .expect("No active output");
        let active_3_ix = tree.init_workspace("3".to_string(), output_ix);
        let workspace_3_ix = tree.tree.parent_of(active_3_ix).unwrap();
        assert!(tree.workspace_ix_by_name("3").is_some(),
                "Workspace three still does not exist, even though we just initialized it");
        assert_eq!(tree.tree[workspace_3_ix].get_type(), ContainerType::Workspace);
        assert_eq!(tree.tree[workspace_3_ix].get_name().unwrap(), "3");
    }

    #[test]
    /// Tests the view functions
    fn view_tests() {
        let mut tree = basic_tree();
        let active_container = tree.active_container.expect("No active container");
        let parent_container = tree.tree.parent_of(active_container).unwrap();
        // When the active container is a view, add it as a sibling
        assert_eq!(tree.tree.children_of(parent_container).len(), 1);
        let old_active_view = tree.active_ix_of(ContainerType::View)
            .expect("Active container was not a view");
        tree.add_view(WlcView::root());
        assert_eq!(tree.tree.children_of(parent_container).len(), 2);
        assert!(! (old_active_view == tree.active_ix_of(ContainerType::View).unwrap()));
        tree.remove_view(&WlcView::root());
        assert_eq!(tree.active_ix_of(ContainerType::View).unwrap(), old_active_view);
        assert_eq!(tree.tree.children_of(parent_container).len(), 1);
    }

    #[test]
    /// Tests that we can remove the active container and have it properly reset
    fn basic_removal() {
        let mut tree = basic_tree();
        let active_view_ix = tree.active_container
            .expect("No active container on basic tree");
        assert!(tree.tree[active_view_ix].get_type() == ContainerType::View,
                "Active container was not a view");
        let workspace_of_active = tree.tree.ancestor_of_type(active_view_ix,
                                                             ContainerType::Workspace)
            .expect("View not part of workspace");
        // The next active container should be the root container of this workspace
        let new_active_container_ix = &tree.tree.children_of(workspace_of_active)[0];

        tree.remove_view_or_container(active_view_ix);
        let new_active_container = tree.active_container
            .expect("Remove view invalidated the active container");
        assert_eq!(new_active_container, *new_active_container_ix);

    }

    #[test]
    /// Ensure that calculate_scale is fair to all it's children
    fn calculate_scale_test() {
        assert_eq!(LayoutTree::calculate_scale(vec!(), 0.0), 0.0);
        assert_eq!(LayoutTree::calculate_scale(vec!(5.0, 5.0, 5.0, 5.0, 5.0, 5.0), 0.0), 30.0);
        assert_eq!(LayoutTree::calculate_scale(vec!(5.0, 5.0, 5.0, 5.0, -5.0, 0.0), 5.0), 22.0);
    }

    #[test]
    /// Ensures that toggle horizontal key (<Leader> + e) does the same thing as it does in i3.
    /// To reiterate: it should always make the active view's parent container( or the container
    /// itself if the active container is a container, not a view) have the horizontal layout
    /// _unless_ it's already horizontal, in which case the layout should be vertical
    fn tiling_toggle_key() {
        let mut tree = basic_tree();
        // active container is the first view, so it should just change it's root.
        match *tree.get_active_parent().unwrap() {
            Container::Container { ref layout, .. } => {
                // default is horizontal
                assert_eq!(*layout, Layout::Horizontal)
            },
            _ => unreachable!()
        }
        tree.toggle_active_horizontal();
        match *tree.get_active_parent().unwrap() {
            Container::Container { ref layout, .. } => {
                // default is horizontal
                assert_eq!(*layout, Layout::Vertical)
            },
            _ => unreachable!()
        }
        // and back again
        tree.toggle_active_horizontal();
        match *tree.get_active_parent().unwrap() {
            Container::Container { ref layout, .. } => {
                // default is horizontal
                assert_eq!(*layout, Layout::Horizontal)
            },
            _ => unreachable!()
        }
    }
}
