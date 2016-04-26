//! Layout handling

// remove
#![allow(unused)]

use rustwlc::handle::{WlcView, WlcOutput};
use rustwlc::types::VIEW_MAXIMIZED;
use std::rc::{Rc, Weak};
use std::fmt::{Debug, Formatter};
use std::fmt::Result as FmtResult;

pub type Container = Box<Containable>;
pub type Node = Rc<Container>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    fn get_parent(&self) -> Option<Node>;

    /// Gets the children of this container.
    ///
    /// Views never have children
    fn get_children(&self) -> Option<Vec<Node>>;

    /// Gets the type of the container
    fn get_type(&self) -> ContainerType;

    /// Returns true if this container is focused.
    fn is_focused(&self) -> bool;

    /// Removes this container and all of its children
    fn remove_container(&self) -> Result<(), &'static str>;

    /// Sets this container (and everything in it) to given visibility
    fn set_visibility(&mut self, visibility: bool);

    /// Gets the X and Y dimensions of the container
    fn get_dimensions(&self) -> (u32, u32);

    /// Gets the position of this container on the screen
    fn get_position(&self) -> (i64, i64);

    /// Returns true if this container is a parent of the child
    fn is_parent_of(&self, child: Node) -> bool {
        unimplemented!();
    }

    /// Returns true if this view is a child is an decedent of the parent
    fn is_child_of(&self, parent: Node) -> bool {
        unimplemented!();
    }

    fn is_root(&self) -> bool {
        self.get_type() == ContainerType::Root
    }

    fn add_child(&mut self, container: Node);


    /// Finds a parent container with the given type, if there is any
    fn get_parent_by_type(&self, container_type: ContainerType) -> Option<Node> {
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
    fn active_workspace(&self) -> Node;
}

pub struct Workspace {
    handle: Option<WlcOutput>,

    parent: Weak<Container>,
    children: Vec<Node>,
    container_type: ContainerType,
    layout: Layout,

    width: u32,
    height: u32,

    x: i64,
    y: i64,

    visible: bool,
    is_focused: bool,
    is_floating: bool,
}

impl Workspace {
    /// Makes a new workspace container. This should only be called by root
    /// since it will properly initialize the right number and properly put
    /// them in the main tree.
    pub fn new_workspace(root: Node) -> Node {
        let workspace: Node =
            Rc::new(Box::new(
                Workspace {
                    handle: None,
                    parent: Rc::downgrade(&root),
                    children: vec!(),
                    container_type: ContainerType::Root,
                    // NOTE Change this to some other default
                    layout: Layout::None,
                    // NOTE Figure out how to initialize these properly
                    width: 0,
                    height: 0,
                    x: 0,
                    y: 0,
                    visible: false,
                    is_focused: false,
                    is_floating: false,
                }));
        if let Some(root) = Rc::get_mut(&mut root.clone()) {
            root.add_child(workspace.clone());
            workspace
        } else {
            panic!("There was a weak reference to root, couldn't get mut");
        }
    }
}

pub struct Root {
    children: Vec<Node>,
}

impl Debug for Root {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "Root: ");
            f.debug_list()
            .entries(self.children.iter().map(|child: &Node| child.get_type()))
            .finish()
    }
}

impl Containable for Root {
    fn get_parent(&self) -> Option<Node> {
        None
    }

    fn get_children(&self) -> Option<Vec<Node>> {
        if self.children.len() == 0 {
            None
        } else {
            Some(self.children.clone())
        }
    }

    fn get_type(&self) -> ContainerType {
        ContainerType::Root
    }

    fn add_child(&mut self, container: Node) {
        // NOTE check to make sure we are not adding a duplicate
        self.children.push(container);
    }

    fn is_focused(&self) -> bool {
        false
    }

    fn remove_container(&self) -> Result<(), &'static str> {
        panic!("Cannot remove the root of the tree");
    }

    fn set_visibility(&mut self, visibility: bool) {
        trace!("Setting visibility of root");
    }

    fn get_dimensions(&self) -> (u32, u32) {
        panic!("Root has no dimensions");
    }

    fn get_position(&self) -> (i64, i64) {
        panic!("Root has no position");
    }

    fn is_parent_of(&self, child: Node) -> bool {
        true
    }

    fn is_child_of(&self, parent: Node) -> bool {
        false
    }

    fn is_root(&self) -> bool {
        true
    }
}

impl Root {
    /// Makes the root container. There should be only one of these
    /// Does not ensure that this is the only root container
    /* NOTE Need to find a way to ensure there is only one of these things
     * Perhaps set a static global variable
     */
    pub fn new_root() -> Node {
        trace!("Root created");
        Rc::new(Box::new(Root { children: vec!() }))
    }
}

impl Containable for Workspace {

    /// Gets the parent that this container sits in.
    ///
    /// If the container is the root, it returns None
    fn get_parent(&self) -> Option<Node> {
        match self.container_type {
            ContainerType::Root => None,
            _ => self.parent.upgrade()
        }
    }

    /// Gets the children of this container.
    ///
    /// Views never have children
    fn get_children(&self) -> Option<Vec<Node>> {
        if self.children.len() == 0 {
            None
        } else {
            Some(self.children.clone())
        }
    }

    fn add_child(&mut self, container: Node) {
        // NOTE check to make sure we are not adding a duplicate
        self.children.push(container);
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
    fn get_dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Gets the position of this container on the screen
    fn get_position(&self) -> (i64, i64) {
        (self.x, self.y)
    }

}

#[derive(Debug)]
pub struct View {
    handle: Option<Box<WlcView>>,
    parent: Weak<Container>,

    width: u32,
    height: u32,

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
    fn get_parent(&self) -> Option<Node> {
        self.parent.upgrade()
    }

    /// Gets the children of this container.
    ///
    /// Views never have children
    fn get_children(&self) -> Option<Vec<Node>> {
        None
    }

    fn add_child(&mut self, container: Node) {
        panic!("Views can not have children");
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
    fn get_dimensions(&self) -> (u32, u32) {
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
    fn active_workspace(&self) -> Node {
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

impl Debug for Containable {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_struct("Containable")
            .field("type", &self.get_type())
            .field("parent", &self.get_parent())
            .field("children", &self.get_children())
            .field("focused", &self.is_focused())
            .finish()
    }
}

/*
impl Debug for View {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_struct("View")
            .field("parent", &self.parent.upgrade())
            .field("handle", &self.handle)
            .field("visible", &self.visible)
            .field("is_focused", &self.is_focused)
            .field("is_floating", &self.is_floating)
            .field("x", &self.x)
            .field("y", &self.y)
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}
*/
