//! Layout handling

// remove
#![allow(unused)]

use rustwlc::handle::{WlcView, WlcOutput};
use rustwlc::types::VIEW_MAXIMIZED;
use std::rc::{Rc, Weak};

pub type Node = Box<Containable>;

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

/// Layout mode for a container
pub enum Layout {
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
    fn get_parent(&self) -> Option<Rc<Node>>;

    /// Gets the children of this container.
    ///
    /// Views never have children
    fn get_children(&self) -> Option<Vec<Rc<Node>>>;

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

    /// Returns true if this container is a parent of the child
    fn is_parent_of(&self, child: Rc<Node>) -> bool {
        unimplemented!();
    }

    /// Returns true if this view is a child is an decedent of the parent
    fn is_child_of(&self, parent: Rc<Node>) -> bool {
        unimplemented!();
    }

    fn is_root(&self) -> bool {
        self.get_type() == ContainerType::Root
    }


    /// Finds a parent container with the given type, if there is any
    fn get_parent_by_type(&self, container_type: ContainerType) -> Option<Rc<Node>> {
        let mut container = self.get_parent();
        loop {
            if let Some(parent) = container {
                if parent.get_type() == container_type {
                    return Some(parent);
                }
                container = parent.get_parent();
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

    /// Gets the active workspace of the view
    fn active_workspace(&self) -> Rc<Node>;
}

pub struct Container {
    handle: Option<WlcOutput>,

    parent: Weak<Node>,
    children: Vec<Rc<Node>>,
    container_type: ContainerType,
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
    fn get_parent(&self) -> Option<Rc<Node>> {
        match self.container_type {
            ContainerType::Root => None,
            _ => self.parent.upgrade()
        }
    }
    
    /// Gets the children of this container.
    ///
    /// Views never have children
    fn get_children(&self) -> Option<Vec<Rc<Node>>> {
        if self.children.len() == 0 {
            None
        } else {
            Some(self.children.clone())
        }
    }

    fn get_type(&self) -> ContainerType {
        self.container_type
    }

    /// Returns true if this container is focused.
    fn is_focused(&self) -> bool {
        self.is_focused
    }

    /// Removes this container and all of its children
    fn remove_container(&self) -> Result<(), &'static str> {
        if let Some(children) = self.get_children() {
            for child in children {
                child.remove_container().ok();
                drop(child);
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


pub struct View {
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
    fn get_parent(&self) -> Option<Rc<Node>> {
        self.parent.upgrade()
    }

    /// Gets the children of this container.
    ///
    /// Views never have children
    fn get_children(&self) -> Option<Vec<Rc<Node>>> {
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

impl Viewable for View {
    /// Determines if the view is full screen
    fn is_fullscreen(&self) -> bool {
        if let Some(ref handle) = self.handle {
            match handle.get_state() {
                VIEW_MAXIMIZED => true,
                _ => false,
            }
        } else {
            false
        }
    }

    /// Figures out if the view is focused
    fn is_active(&self) -> bool {
        self.is_focused
    }

    /// Gets the active workspace of the view
    fn active_workspace(&self) -> Rc<Node> {
        let mut workspace = self.get_parent();
        loop {
            if let Some(parent) = workspace {
                if parent.get_type() == ContainerType::Workspace {
                    return parent
                }
                workspace = parent.get_parent();
            } else {
                // Should never happen under our current setup
                panic!("View not attached to a workspace!")
            }
        }
    }
    
}
