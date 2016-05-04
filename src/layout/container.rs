//! Container types

use rustwlc::handle::{WlcView, WlcOutput};

/// A handle to either a view or output
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Handle {
    View(WlcView),
    Output(WlcOutput)
}

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

/// Layout mode for a container
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    Horizontal,
    Vertical,
    Stacked,
    Tabbed,
    Floating
}

/// Represents an item in the container tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Container {
    /// Root node of the container
    Root,
    /// Output
    Output {
        /// Handle to the wlc
        handle: WlcOutput
    },
    /// Workspace
    Workspace {
        /// Name of the workspace
        name: String,
        /// Whether the workspace is focused
        ///
        /// Multiple workspaces can be focused
        focused: bool
    },
    /// Container
    Container {
        /// How the container is layed out
        layout: Layout,
        /// Whether the container is visible
        visible: bool,
        /// If the container is focused
        focused: bool,
        /// If the container is floating
        floating: bool,
    },
    /// View or window
    View {
        /// The wlc handle to the view
        handle: WlcView,
        /// Whether this view is visible
        visible: bool,
        /// Whether this view is focused
        focused: bool,
        /// Whether this view is floating
        floating: bool,
    }
}

impl Container {
    /// Creates a new root container
    pub fn new_root() -> Container {
        Container::Root
    }

    /// Creates a new output container with the given output
    pub fn new_output(handle: WlcOutput) -> Container {
        Container::Output {
            handle: handle
        }
    }

    /// Creates a new workspace container with the given name
    pub fn new_workspace(name: String) -> Container {
        Container::Workspace {
            name: name, focused: false
        }
    }

    /// Creates a new container
    pub fn new_container() -> Container {
        Container::Container {
            layout: Layout::Horizontal, // default layout?
            visible: false,
            focused: false,
            floating: false
        }
    }

    /// Creates a new view container with the given handle
    pub fn new_view(handle: WlcView) -> Container {
        Container::View {
            handle: handle,
            visible: false,
            focused: false,
            floating: false
        }
    }

    /// Gets the type of this container
    pub fn get_type(&self) -> ContainerType {
        match *self {
            Container::Root => ContainerType::Root,
            Container::Output { .. } => ContainerType::Output,
            Container::Workspace { .. } => ContainerType::Workspace,
            Container::Container { .. } => ContainerType::Container,
            Container::View { .. } => ContainerType::View
        }
    }
}
