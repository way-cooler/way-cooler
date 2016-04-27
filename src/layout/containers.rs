//! Layout handling

// remove
#![allow(unused)]

use rustwlc::handle::{WlcView, WlcOutput};
use rustwlc::types::VIEW_MAXIMIZED;
use std::rc::{Rc, Weak};
use std::fmt;

pub type Node = Rc<Container>;

#[derive(Debug)]
enum Handle {
    View(WlcView),
    Output(WlcOutput)
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
#[derive(Debug)]
pub enum Layout {
    None,
    Horizontal,
    Vertical,
    Stacked,
    Tabbed,
    Floating
}

#[derive(Debug)]
pub struct Container {
    handle: Option<Handle>,
    parent: Option<Weak<Container>>,
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

/// Like i3, everything (workspaces, containers, views) are containable.
impl Container {
    
    /// Makes the root container. There should be only one of these
    /// Does not ensure that this is the only root container
    // NOTE Need to find a way to ensure there is only one of these things
    // Perhaps set a static global variable
    pub fn new_root() -> Node {
        trace!("Root created");
        Rc::new(Container {
            handle: None,
            parent: None,
            children: vec!(),
            container_type: ContainerType::Root,
            layout: Layout::None,
            width: 0,
            height: 0,
            x: 0,
            y: 0,
            visible: false,
            is_focused: false,
            is_floating: false
        })
    }
    
    /// Makes a new workspace container. This should only be called by root
    /// since it will properly initialize the right number and properly put
    /// them in the main tree.
    pub fn new_workspace(root: Node) -> Node {
        let workspace: Node =
            Rc::new(Container {
                handle: None,
                parent: Some(Rc::downgrade(&root)),
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
                });
        if let Some(root) = Rc::get_mut(&mut root.clone()) {
            root.add_child(workspace.clone());
            workspace
        } else {
            panic!("There was a weak reference to root, couldn't initialize workspace");
        }
    }
    /// Gets the parent that this container sits in.
    ///
    /// If the container is the root, it returns None
    pub fn get_parent(&self) -> Option<Node> {
        if self.is_root() {
            None
        } else {
            // NOTE Clone has to be done here because e have to store the parent
            // as an option since `Weak::new` is unstable
            self.parent.clone().unwrap().upgrade()
        }
    }

    /// Gets the children of this container.
    ///
    /// Views never have children
    pub fn get_children(&self) -> Option<Vec<Node>> {
        if self.children.len() > 0 {
            Some(self.children.clone())
        } else {
            None
        }
    }

    /// Gets the type of the container
    pub fn get_type(&self) -> ContainerType {
        self.container_type
    }

    /// Returns true if this container is focused.
    pub fn is_focused(&self) -> bool {
        self.is_focused
    }

    /// Removes the child at the specified index
    // NOTE Make a wrapper function that can take a reference and remove it
    pub fn remove_child(&mut self, index: usize) -> Result<Node, &'static str> {
        Ok(self.children.remove(index))
    }

    /// Sets this container (and everything in it) to given visibility
    pub fn set_visibility(&mut self, visibility: bool) {
        self.visible = visibility
    }

    /// Gets the X and Y dimensions of the container
    pub fn get_dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Gets the position of this container on the screen
    pub fn get_position(&self) -> (i64, i64) {
        (self.x, self.y)
    }

    /// Returns true if this container is a parent of the child
    pub fn is_parent_of(&self, child: Node) -> bool {
        if self.is_root() {
            true
        } else {
            unimplemented!();
        }
    }

    /// Returns true if this view is a child is an decedent of the parent
    pub fn is_child_of(&self, parent: Node) -> bool {
        if self.is_root() {
            false
        } else {
            unimplemented!();
        }
    }

    pub fn is_root(&self) -> bool {
        self.get_type() == ContainerType::Root
    }

    /// Adds the node to the children of this container
    pub fn add_child(&mut self, container: Node) {
        //if ! self.children.contains(&container) {
            self.children.push(container);
        //}
    }

    /// Finds a parent container with the given type, if there is any
    pub fn get_parent_by_type(&self, container_type: ContainerType) -> Option<Node> {
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
