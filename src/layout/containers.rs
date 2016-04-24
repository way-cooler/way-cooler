//! Layout handling

use rustwlc::handle::{WlcView, WlcOutput};
use std::rc::{Rc, Weak};

type Node = Box<Containable>;

#[derive(PartialEq, Clone, Copy)]
pub enum ContainerType {
    /// Root container, only one exists 
    Root,
    /// WlcOutput/Monitor
    Output,
    /// A workspace 
    Workspace,
    /// A view (window)
    View
}

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
    fn get_parent(&self) -> Weak<Node>;

    /// Gets the children of this container.
    ///
    /// Views never have children
    fn get_children(&self) -> Option<Vec<Weak<Node>>>;

    /// Gets the type of the container
    fn get_type(&self) -> ContainerType;

    /// Returns true if this container is focused.
    fn is_focused(&self) -> bool;

    /// Removes this container and all of its children
    fn remove_container(&self) -> Result<(), &'static str>;

    /// Sets this container (and everything in it) to given visibility
    fn set_visibility(&mut self, visibility: bool);

    /// Gets the X and Y dimensions of the container
    fn get_dimensions(&self) -> (u64, u64);

    /// Gets the position of this container on the screen
    fn get_position(&self) -> (i64, i64);

    /// Finds a parent container with the given type, if there is any
    fn get_parent_by_type(&self, type_: ContainerType) -> Option<Rc<Node>> {
        let mut container = self.get_parent().upgrade();
        loop {
            if let Some(parent) = container {
                if parent.get_type() == type_ {
                    return Some(parent);
                }
                container = parent.get_parent().upgrade();
            } else {
                return None;
            }
        }
    }
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
    handle: Option<WlcOutput>,

    parent: Weak<Node>,
    children: Vec<Rc<Node>>,
    type_: ContainerType,
    layout: Layout,

    width: u64,
    height: u64,

    x: i64,
    y: i64,

    visible: bool,
    is_focused: bool,
    is_floating: bool,
}

impl Containable for Container {

    /// Gets the parent that this container sits in.
    ///
    /// If the container is the root, it returns None
    fn get_parent(&self) -> Weak<Node> {
        self.parent.clone()
        /*
        match self.type_ {
            ContainerType::Root => { 
                // such hack
                unsafe {
                    // very unsafe
                    use std::mem;
                    let _dummy: Node = mem::uninitialized();
                    // much dummy
                    let _rc_dummy = Rc::new(_dummy);
                    Rc::downgrade(&_rc_dummy)}
            },
            _ => self.parent.clone(),
        }*/
    }
    /// Gets the children of this container.
    ///
    /// Views never have children
    fn get_children(&self) -> Option<Vec<Weak<Node>>> {
        if self.children.len() == 0 {
            None
        } else {
            Some(self.children.iter().map(|child| Rc::downgrade(&child)).collect())
        }
    }

    fn get_type(&self) -> ContainerType {
        self.type_
    }

    /// Returns true if this container is focused.
    fn is_focused(&self) -> bool {
        self.is_focused
    }

    /// Removes this container and all of its children
    fn remove_container(&self) -> Result<(), &'static str> {
        for child in self.get_children().expect("No children") {
            if let Some(child) = child.upgrade() {
                if let Ok(child) = Rc::try_unwrap(child) {
                    child.remove_container();
                    drop(child);
                }
            }
        }
        Ok(())
    }

    /// Sets this container (and everything in it) to given visibility
    fn set_visibility(&mut self, visibility: bool) {
        self.visible = visibility
    }

    /// Gets the X (width) and Y (height) dimensions of the container
    fn get_dimensions(&self) -> (u64, u64) {
        (self.width, self.height)
    }

    /// Gets the position of this container on the screen
    fn get_position(&self) -> (i64, i64) {
        (self.x, self.y)
    }

}


struct View {
    handle: Option<Box<WlcView>>,
    parent: Weak<Node>,

    width: u64,
    height: u64,

    x: i64,
    y: i64,

    visible: bool,
    is_focused: bool,
    is_floating: bool,
}

impl Containable for View {
    /// Gets the parent that this container sits in.
    ///
    /// If the container is the root, it returns None
    fn get_parent(&self) -> Weak<Node> {
        self.parent.clone()
    }

    /// Gets the children of this container.
    ///
    /// Views never have children
    fn get_children(&self) -> Option<Vec<Weak<Node>>> {
        None
    }

    /// Gets the type of the container
    fn get_type(&self) -> ContainerType {
        ContainerType::View
    }

    /// Returns true if this container is focused.
    fn is_focused(&self) -> bool {
        self.is_focused
    }

    /// Removes this container and all of its children
    fn remove_container(&self) -> Result<(), &'static str> {
        if let Some(ref handle) = self.handle {
            handle.close();
        }
        drop(self);
        Ok(())
    }

    /// Sets this container (and everything in it) to given visibility
    fn set_visibility(&mut self, visibility: bool) {
        self.visible = visibility
    }

    /// Gets the X and Y dimensions of the container
    fn get_dimensions(&self) -> (u64, u64) {
        (self.width, self.height)
    }

    /// Gets the position of this container on the screen
    fn get_position(&self) -> (i64, i64) {
        (self.x, self.y)
    }
}
