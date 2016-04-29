//! Layout handling

// remove
#![allow(unused)]

use rustwlc::handle::{WlcView, WlcOutput};
use rustwlc::types::VIEW_MAXIMIZED;
use std::rc::{Rc, Weak};
use std::fmt::{Debug, Formatter};
use std::fmt::Result as FmtResult;
use std::cell::RefCell;

pub type Node = Rc<RefCell<Container>>;

#[derive(Debug, Clone)]
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
    /// A Container, houses views and other containers
    Container,
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

#[derive(Clone)]
pub struct Container {
    handle: Option<Handle>,
    parent: Option<Weak<RefCell<Container>>>,
    children: Vec<Node>,
    container_type: ContainerType,
    layout: Layout,
    width: u32,
    height: u32,
    x: i32,
    y: i32,
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
        Rc::new(RefCell::new(Container {
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
        }))
    }
    
    /// Makes a new workspace container. This should only be called by root
    /// since it will properly initialize the right number and properly put
    /// them in the main tree.
    pub fn new_workspace(root: &mut Node) -> Node {
        if ! root.borrow().is_root() {
            panic!("Only workspaces can be added to the root node");
        }
        let workspace: Node =
            Rc::new(RefCell::new(Container {
                // NOTE Give this an output
                handle: None,
                parent: Some(Rc::downgrade(&root)),
                children: vec!(),
                container_type: ContainerType::Workspace,
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
        root.borrow_mut().add_child(workspace.clone());
        trace!("Workspace created");
        workspace
    }

    /// Makes a new container. These hold views and other containers.
    /// Container hold information about specific parts of the tree in some
    /// workspace and the layout of the views within.
    pub fn new_container(parent_: &mut Node, output: WlcOutput) -> Node {
        let mut parent = parent_.borrow_mut();
        if parent.is_root() {
            panic!("Container cannot be a direct child of root");
        }
        let container = Rc::new(RefCell::new(Container {
            handle: Some(Handle::Output(output)),
            parent: Some(Rc::downgrade(&parent_)),
            children: vec!(),
            container_type: ContainerType::Container,
            // NOTE Get default, either from config or from workspace
            layout: Layout::None,
            // NOTE Get this information from somewhere, or set it later
            width: 0,
            height: 0,
            x: 0,
            y: 0,
            visible: false,
            is_focused: false,
            is_floating: false,
        }));
        parent.add_child(container.clone());
        trace!("Container created");
        container
    }

    /// Makes a new view. A view holds either a Wayland or an X Wayland window.
    pub fn new_view(parent_: &mut Node, wlc_view: WlcView) -> Node {
        let mut parent = parent_.borrow_mut();
        let (mut w, mut h, mut x, mut y) = (0u32, 0u32, 0i32, 0i32);
        if let Some(geometry) = wlc_view.get_geometry().clone() {
            h = geometry.size.h;
            w = geometry.size.w;
            x = geometry.origin.x;
            y = geometry.origin.y;
        }
        if parent.is_root() {
            panic!("View cannot be a direct child of root");
        }
        let view = Rc::new(RefCell::new(Container {
            handle: Some(Handle::View(wlc_view)),
            parent: Some(Rc::downgrade(&parent_)),
            children: vec!(),
            container_type: ContainerType::View,
            layout: Layout::None,
            width: w,
            height: h,
            x: x,
            y: y,
            visible: false,
            is_focused: false,
            is_floating: false
        }));
        if parent.get_type() == ContainerType::Workspace {
            // Case of focused workspace, just create a child of it
            parent.add_child(view.clone());
        } else {
            // Regular case, create as sibling of current container
            parent.add_sibling(view.clone());
        }
        trace!("View created");
        view
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
            if let Some(parent) = self.parent.clone() {
                parent.upgrade()
            } else {
                None
            }
        }
    }

    pub fn add_child(&mut self, container: Node) -> Result<(), &'static str> {
        if self.get_type() == ContainerType::Workspace 
            && container.borrow().get_type() == ContainerType::Workspace {
            return Err("Only containers can be children of a workspace");
        } else if self.get_type() == ContainerType::View {
            return Err("Cannot add child to a view");
        }
        // NOTE check to make sure we are not adding a duplicate
        self.children.push(container);
        Ok(())
    }

    pub fn add_sibling(&mut self, container: Node) -> Result<(), &'static str> {
        if self.is_root() {
            return Err("Root has no sibling, cannot add sibling to root");
        }
        let parent = self.get_parent().unwrap();
        trace!("Borrowing container {:?} (parent of {:?}) as mutable", parent, self);
        parent.borrow_mut().add_child(container);
        Ok(())
    }

    /// Removes this container and all of its children.
    /// You MUST call this function while `borrow`ing, a mutable borrow will
    /// cause it to panic at run time
    pub fn remove_container(&self) -> Result<(), &'static str> {
        if self.is_root() {
            return Err("Cannot remove root container");
        } else if self.get_type() == ContainerType::Workspace  {
            return Err("Cannot remove workspace container");
        }
        if let Some(parent) = self.get_parent() {
            // NOTE Add check here to ensure we can borrow mutably once that
            // feature stabilizes
            trace!("Borrowing container {:?} (parent of {:?}) as mutable", parent, self);
            parent.borrow_mut().remove_child(self);
        }
        Ok(())
    }

    /// Gets the children of this container.
    ///
    /// Views never have children
    pub fn get_children(&self) -> Option<&[Node]> {
        if self.get_type() == ContainerType::View {
            None
        }
        else {
            Some(self.children.as_slice())
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
    pub fn remove_child_at(&mut self, index: usize) -> Result<Node, &'static str> {
        // NOTE Add check here to ensure we can borrow once that
        // feature stabilizes
        if self.children[index].borrow().get_type() == ContainerType::Workspace {
            return Err("Cannot remove workspace")
        }
        Ok(self.children.remove(index))
    }

    /// Removes the given child from this container's children.
    /// If the child is not present, then an error is returned
    pub fn remove_child(&mut self, node: &Container) -> Result<Node, &'static str> {
        for (index, child) in self.children.clone().iter().enumerate() {
            // NOTE Add check here to ensure we can borrow once that
            // feature stabilizes
            if *child.borrow() == *node {
                if child.borrow().get_type() == ContainerType::Workspace {
                    return Err("Can not remove workspace");
                }
                return Ok(self.children.remove(index));
            }
        }
        return Err("Could not find child in container");//format!("Could not find child {:?} in {:?}", node, self));
    }

    /// Sets this container (and everything in it) to given visibility
    pub fn set_visibility(&mut self, visibility: bool) {
        self.visible = visibility
    }

    /// Gets the visibility of the container
    pub fn get_visibility(&self) -> bool {
        self.visible
    }

    /// Gets the X and Y dimensions of the container
    pub fn get_dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Gets the position of this container on the screen
    pub fn get_position(&self) -> (i32, i32) {
        (self.x, self.y)
    }

    /// Returns true if this container is a parent of the child
    pub fn is_parent_of(&self, child: Node) -> bool {
        if self.is_root() {
            true
        } else {
            while ! child.borrow().is_root() {
                let parent = child.borrow().get_parent().unwrap();
                if child == parent {
                    return true
                }
            }
            return false;
        }
    }

    /// Returns true if this container is a child is an decedent of the parent
    pub fn is_child_of(&self, parent: Node) -> bool {
        parent.borrow().is_parent_of(Rc::new(RefCell::new(self.clone())))
    }

    pub fn is_root(&self) -> bool {
        self.get_type() == ContainerType::Root
    }

    /// Finds a parent container with the given type, if there is any
    pub fn get_parent_by_type(&self, container_type: ContainerType) -> Option<Node> {
        let mut container = self.get_parent();
        loop {
            if let Some(parent) = container {
                if parent.borrow().get_type() == container_type {
                    return Some(parent);
                }
                container = parent.borrow().get_parent();
            } else {
                return None;
            }
        }
    }
}

impl PartialEq for Container {
    fn eq(&self, other: &Container) -> bool {
        self.get_type() == other.get_type()
    }
}

impl Eq for Container { }

impl Debug for Container {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_struct("Containable")
            .field("type", &self.get_type())
            //.field("parent", &self.get_parent())
            .field("children", &self.get_children())
            .field("focused", &self.is_focused())
            .finish()
    }
}
