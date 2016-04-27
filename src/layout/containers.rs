//! Layout handling

// remove
#![allow(unused)]

use rustwlc::handle::{WlcView, WlcOutput};
use rustwlc::types::VIEW_MAXIMIZED;
use std::rc::{Rc, Weak};
use std::fmt::{Debug, Formatter};
use std::fmt::Result as FmtResult;
use std::cell::RefCell;

pub type Node = Rc<Container>;

#[derive(Debug)]
enum Handle {
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

pub struct Container {
    handle: Option<Handle>,
    parent: RefCell<Option<Weak<Container>>>,
    children: RefCell<Vec<Node>>,
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
            parent: RefCell::new(None),
            children: RefCell::new(vec!()),
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
    pub fn new_workspace(root: &mut Node) -> Node {
        println!("weak: {}, strong: {}", Rc::weak_count(root), Rc::strong_count(root));
        let workspace: Node =
            Rc::new(Container {
                handle: None,
                parent: RefCell::new(Some(Rc::downgrade(&root))),
                children: RefCell::new(vec!()),
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
        println!("weak: {}, strong: {}", Rc::weak_count(root), Rc::strong_count(root));
        if let Some(root) = Rc::get_mut(root) {
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
            // NOTE Clone has to be done here because we have to store the
            // parent as an option since the `Weak::new` is unstable
            self.parent.borrow().clone().unwrap().upgrade()
        }
    }

    fn add_child(&mut self, container: Node) {
        // NOTE check to make sure we are not adding a duplicate
        self.children.borrow_mut().push(container);
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

    /// Gets the children of this container.
    ///
    /// Views never have children
    pub fn get_children(&self) -> Option<Vec<Node>> {
        if self.children.borrow().len() > 0 {
            Some(self.children.borrow().clone())
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
        Ok(self.children.borrow_mut().remove(index))
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

impl Debug for Container {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_struct("Containable")
            .field("type", &self.get_type())
            .field("parent", &self.get_parent())
            .field("children", &self.get_children())
            .field("focused", &self.is_focused())
            .finish()
    }
}
