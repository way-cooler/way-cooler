//! Container types

use uuid::Uuid;

use rustwlc::handle::{WlcView, WlcOutput};
use rustwlc::{Geometry, Point, Size};

/// A handle to either a view or output
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Handle {
    View(WlcView),
    Output(WlcOutput)
}

/// Types of containers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerType {
    /// Root container, only one exists
    Root,
    /// WlcOutput/Monitor
    Output,
    /// A workspace
    Workspace,
    /// A Container, houses views and other containers
    Container,
    /// A view (window)
    View
}

impl ContainerType {
    /// Whether this container can be used as the parent of another
    pub fn can_have_child(self, other: ContainerType) -> bool {
        use self::ContainerType::*;
        match self {
            Root => other == Output,
            Output => other == Workspace,
            Workspace => other == Container,
            Container => other == Container || other == View,
            View => false
        }
    }
}

/// Layout mode for a container
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    Horizontal,
    Vertical
}

/// Represents an item in the container tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Container {
    /// Root node of the container
    Root(Uuid),
    /// Output
    Output {
        /// Handle to the wlc
        handle: WlcOutput,
        /// UUID associated with container, client program can use container
        id: Uuid,
    },
    /// Workspace
    Workspace {
        /// Name of the workspace
        name: String,
        /// The size of the workspace on the screen.
        /// Might be different if there is e.g a bar present
        size: Size,
        /// UUID associated with container, client program can use container
        id: Uuid,
    },
    /// Container
    Container {
        /// How the container is layed out
        layout: Layout,
        /// If the container is floating
        floating: bool,
        /// The geometry of the container, relative to the parent container
        geometry: Geometry,
        /// UUID associated with container, client program can use container
        id: Uuid,
    },
    /// View or window
    View {
        /// The wlc handle to the view
        handle: WlcView,
        /// Whether this view is floating
        floating: bool,
        /// UUID associated with container, client program can use container
        id: Uuid,
    }
}

impl Container {
    /// Creates a new root container.
    pub fn new_root() -> Container {
        Container::Root(Uuid::new_v4())
    }
    /// Creates a new output container with the given output
    pub fn new_output(handle: WlcOutput) -> Container {
        Container::Output {
            handle: handle,
            id: Uuid::new_v4()
        }
    }

    /// Creates a new workspace container with the given name and size.
    /// Usually the size is the same as the output it resides on,
    /// unless there is a bar or something.
    pub fn new_workspace(name: String, size: Size) -> Container {
        Container::Workspace {
            name: name,
            size: size,
            id: Uuid::new_v4()
        }
    }

    /// Creates a new container
    pub fn new_container(geometry: Geometry) -> Container {
        Container::Container {
            layout: Layout::Horizontal,
            floating: false,
            geometry: geometry,
            id: Uuid::new_v4()
        }
    }

    /// Creates a new view container with the given handle
    pub fn new_view(handle: WlcView) -> Container {
        Container::View {
            handle: handle,
            floating: false,
            id: Uuid::new_v4()
        }
    }

    /// Sets the visibility of this container
    pub fn set_visibility(&mut self, visibility: bool) {
        let mask = if visibility { 1 } else { 0 };
        if let Some(handle) = self.get_handle() {
            match handle {
                Handle::View(view) => {
                    view.set_mask(mask)
                },
                _ => {},
            }
        }
    }

    /// Gets the type of this container
    pub fn get_type(&self) -> ContainerType {
        match *self {
            Container::Root(_) => ContainerType::Root,
            Container::Output { .. } => ContainerType::Output,
            Container::Workspace { .. } => ContainerType::Workspace,
            Container::Container { .. } => ContainerType::Container,
            Container::View { .. } => ContainerType::View
        }
    }

    /// Gets the view handle of the view container, if this is a view container
    pub fn get_handle(&self) -> Option<Handle> {
        match *self {
            Container::View { ref handle, ..} => Some(Handle::View(handle.clone())),
            Container::Output { ref handle, .. } => Some(Handle::Output(handle.clone())),
            _ => None
        }
    }

    /// Gets the name of the workspace, if this container is a workspace.
    pub fn get_name(&self) -> Option<&str> {
        match *self {
            Container::Workspace { ref name, ..} => Some(name),
            _ => None
        }
    }

    /// Gets the geometry of the container, if the container has one.
    /// Root: Returns None
    /// Workspace/Output: Size is the size of the screen, origin is just 0,0
    /// Container/View: Size is the size of the container,
    /// origin is the coordinates relative to the parent container.
    pub fn get_geometry(&self) -> Option<Geometry> {
        match *self {
            Container::Root(_)  => None,
            Container::Output { ref handle, .. } => Some(Geometry {
                origin: Point { x: 0, y: 0 },
                size: handle.get_resolution()
                    .expect("Couldn't get output resolution")
            }),
            Container::Workspace { ref size, .. } => Some(Geometry {
                origin: Point { x: 0, y: 0},
                size: size.clone()
            }),
            Container::Container { ref geometry, .. } => Some(geometry.clone()),
            Container::View { ref handle, ..} =>
                handle.get_geometry().map(|geo| geo.clone()),
        }
    }

    pub fn set_layout(&mut self, new_layout: Layout) -> Result<(), String>{
        match *self {
            Container::Container { ref mut layout, .. } => *layout = new_layout,
            ref other => return Err(
                format!("Can only set the layout of a container, not {:?}",
                        other))
        }
        Ok(())
    }

    pub fn get_id(&self) -> Uuid {
        match *self {
            Container::Root(id) | Container::Output { id, .. } |
            Container::Workspace { id, .. } | Container::Container { id, .. } |
            Container::View { id, .. } => {
                id
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustwlc::*;

    #[test]
    fn can_have_child() {
        let root = ContainerType::Root;
        let output = ContainerType::Output;
        let workspace = ContainerType::Workspace;
        let container = ContainerType::Container;
        let view = ContainerType::View;

        assert!(root.can_have_child(output),         "Root      > output");
        assert!(output.can_have_child(workspace),    "Output    > workspace");
        assert!(workspace.can_have_child(container), "Workspace > container");
        assert!(container.can_have_child(container), "Container > container");
        assert!(container.can_have_child(view),      "Container > view");

        assert!(!root.can_have_child(root),      "! Root > root");
        assert!(!root.can_have_child(workspace), "! Root > workspace");
        assert!(!root.can_have_child(container), "! Root > container");
        assert!(!root.can_have_child(view),      "! Root > view");

        assert!(!output.can_have_child(root),      "! Output > root");
        assert!(!output.can_have_child(output),    "! Output > output");
        assert!(!output.can_have_child(container), "! Output > container");
        assert!(!output.can_have_child(view),      "! Output > view");

        assert!(!workspace.can_have_child(root),      "! Workspace > root");
        assert!(!workspace.can_have_child(output),    "! Workspace > output");
        assert!(!workspace.can_have_child(workspace), "! Workspace > worksp");
        assert!(!workspace.can_have_child(view),      "! Workspace > view");

        assert!(!container.can_have_child(root),      "! Container > root");
        assert!(!container.can_have_child(workspace), "! Container > worksp");
        assert!(!container.can_have_child(output),    "! Container > contanr");

        assert!(!view.can_have_child(root),      "! View > root");
        assert!(!view.can_have_child(output),    "! View > output");
        assert!(!view.can_have_child(workspace), "! View > workspace");
        assert!(!view.can_have_child(container), "! View > container");
        assert!(!view.can_have_child(view),      "! View > view");
    }

    #[test]
    #[allow(unused_variables)]
    /// Tests set and get geometry
    fn geometry_test() {
        use rustwlc::*;
        let test_geometry1 = Geometry {
            origin: Point { x: 800, y: 600 },
            size: Size { w: 500, h: 500}
        };
        let test_geometry2 = Geometry {
            origin: Point { x: 1024, y: 2048},
            size: Size { w: 500, h: 700}
        };
        let root = Container::new_root();
        assert!(root.get_geometry().is_none());

        let output = Container::new_output(WlcView::root().as_output());

        let workspace = Container::new_workspace("1".to_string(),
                                                     Size { w: 500, h: 500 });
        assert_eq!(workspace.get_geometry(), Some(Geometry {
            size: Size { w: 500, h: 500},
            origin: Point { x: 0, y: 0}
        }));
    }

    #[test]
    fn layout_change_test() {
        let root = Container::new_root();
        let output = Container::new_output(WlcView::root().as_output());
        let workspace = Container::new_workspace("1".to_string(),
                                                     Size { w: 500, h: 500 });
        let mut container = Container::new_container(Geometry {
            origin: Point { x: 0, y: 0},
            size: Size { w: 0, h:0}
        });
        let view = Container::new_view(WlcView::root());

        /* Container first, the only thing we can set the layout on */
        let layout = match container {
            Container::Container { ref layout, .. } => layout.clone(),
            _ => panic!()
        };
        assert_eq!(layout, Layout::Horizontal);
        let layouts = [Layout::Vertical, Layout::Horizontal];
        for new_layout in &layouts {
            container.set_layout(*new_layout).ok();
            let layout = match container {
                Container::Container { ref layout, .. } => layout.clone(),
                _ => panic!()
            };
            assert_eq!(layout, *new_layout);
        }

        for new_layout in &layouts {
            for container in &mut [root.clone(), output.clone(),
                                   workspace.clone(), view.clone()] {
                let result = container.set_layout(*new_layout);
                assert!(result.is_err());
            }
        }
    }
}
