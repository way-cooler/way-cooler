//! Layout handling

use rustwlc::{WlcView, WlcOutput};

/// Types of container
enum ContainerTypes {
    /// Root container, only one exists
    Root,
    /// WlcOutput/monitor
    Output,
    /// A workspace
    Workspace,
    /// A view (window)
    View
}

/// Layout mode for a container
enum Layout {
    None,
    Horizontal,
    Vertical,
    Stacked,
    Tabbed,
    Floating
}

/// Like i3, everything (workspaces, containers, views) are containable.
pub trait Containable {
    /// Gets the parent that this container sits in.
    ///
    /// If the container is the root, it returns None
    fn get_parent<T: Containable>(&self) -> Option<T>;

    /// Gets the children of this container.
    ///
    /// Views never have children
    fn get_children<T: Containable>(&self) -> Vec<T>;

    /// Returns true if this container is focused.
    fn is_focused(&self) -> bool;

    /// Removes this container and all of its children
    fn remove_container(self);

    /// Sets this container (and everything in it) to given visibility
    fn set_visibility(visibilty: bool);

    /// Gets the X and Y dimensions of the container
    fn get_dimensions(&self) -> (u64, u64);

    /// Gets the position of this container on the screen
    fn get_position(&self) -> (i64, i64);

    /// Finds a parent container with the given type
    fn get_parent_by_type<T: Containable>(&self, type_: T) -> T;
}

/// View specific functions
pub trait Viewable {
    /// Determines if the view is full screen
    fn is_fullscreen(&self) -> bool;

    /// Figures out if the view is focused
    fn is_active(&self) -> bool;

    /// Returns true if this view is a parent is an ancestor of the child
    fn is_parent_of<T: Containable>(&self, child: T) -> bool;

    /// Returns true if this view is a child is an decedent of the parent
    fn is_child_of<T: Containable>(&self, parent: T) -> bool;

    /// Gets the active workspace of the view
    fn active_workspace<T: Containable>(&self) -> T;
}

struct Container {
    // Can't be a view, that's the View struct's job
    handle: Option<WlcOutput>,
    type_: ContainerTypes,
    layout: Layout,

    width: u64,
    height: u64,

    x: i64,
    y: i64,

    visible: bool,
    is_focused: bool,
    is_floating: bool,
}

struct View {
    handle: Option<WlcView>,

    width: u64,
    height: u64,

    x: i64,
    y: i64,

    visible: bool,
    is_focused: bool,
    is_floating: bool,
}
