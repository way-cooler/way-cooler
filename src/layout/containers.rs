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

#[derive(Clone)]
pub struct Container {
    handle: Option<Handle>,
    parent: Option<Weak<RefCell<Container>>>,
    children: Vec<Node>,
    container_type: ContainerType,
    layout: Option<Layout>,
    visible: bool,
    is_focused: bool,
    is_floating: bool,
}

/// Like i3, everything (workspaces, containers, views) are containable.
impl Container {
    
    /// Makes the root container. There should be only one of these
    /// Does not ensure that this is the only root container
    pub fn new_root() -> Node {
        trace!("Root created");
        Rc::new(RefCell::new(Container {
            handle: None,
            parent: None,
            children: vec!(),
            container_type: ContainerType::Root,
            layout: None,
            visible: false,
            is_focused: false,
            is_floating: false
        }))
    }

    /// Makes a new output. These can only be children of the root node. They
    /// contain information about which monitor (output) its children are being
    /// displayed on.
    pub fn new_output(root: &mut Node, wlc_output: WlcOutput) -> Node {
        if ! root.borrow().is_root() {
            panic!("Output containers can only be children of the root, not {:?}", root.borrow().get_type());
        }

        let size = wlc_output.get_resolution();
        let output = Rc::new(RefCell::new(Container {
            handle: Some(Handle::Output(wlc_output.clone())),
            parent: Some(Rc::downgrade(&root)),
            children: vec!(),
            container_type: ContainerType::Output,
            layout: None,
            visible: false,
            is_focused: false,
            is_floating: false
        }));
        // NOTE Should automatically add a new workspace here, there are edge
        // cases to worry about though so leaving that out for now
        root.borrow_mut().add_child(output.clone());
        output
    }
    
    /// Makes a new workspace container. These can only be attached to output
    /// containers. They will inherit the output of the parent container.
    pub fn new_workspace(parent_: &mut Node) -> Node {
        let parent = parent_.borrow();
        if parent.get_type() != ContainerType::Output {
            panic!("Workspaces can only be children of an output container, not {:?}", parent.get_type());
        }
        let workspace: Node =
            Rc::new(RefCell::new(Container {
                handle: Some(parent.get_handle().expect("Workspace could not get handle from output parent")),
                parent: Some(Rc::downgrade(&parent_)),
                children: vec!(),
                container_type: ContainerType::Workspace,
                layout: Some(Layout::Horizontal),
                visible: false,
                is_focused: false,
                is_floating: false,
                }));
        drop(parent);
        parent_.borrow_mut().add_child(workspace.clone());
        trace!("Workspace created");
        workspace
    }

    /// Makes a new container. These hold views and other containers.
    /// Container hold information about specific parts of the tree in some
    /// workspace and the layout of the views within.
    pub fn new_container(parent_: &mut Node, mut layout: Layout) -> Node {
        let mut parent = parent_.borrow_mut();
        if ! (parent.get_type() == ContainerType::Container 
            || parent.get_type() == ContainerType::Workspace) {
            panic!("Container can only be child of container or workspace, not {:?}", parent.get_type());
        }
        let output = parent.get_handle().expect("Container could not get handle from parent");
        // NOTE Make sure we do all of the Container specific init stuff
        let container = Rc::new(RefCell::new(Container {
            handle: Some(output),
            parent: Some(Rc::downgrade(&parent_)),
            children: vec!(),
            container_type: ContainerType::Container,
            layout: Some(layout),
            visible: false,
            is_focused: false,
            is_floating: false,
        }));
        if parent.get_type() == ContainerType::Workspace {
            let mut workspace = parent;
            if let Some(layout) = workspace.get_layout() {
                container.borrow_mut().set_layout(layout);
            }
            workspace.add_child(container.clone());
        } else {
            // Need to add the "parent" as the child of the container we just made.
            drop(parent);
            let child_ = parent_.clone();
            let child = child_.borrow();
            let parent_of_child = child.get_parent().expect("Parent container has no parent");
            let child_clone = Rc::make_mut(&mut child_.clone()).clone().into_inner();
            // Need to remove the child from their parent
            parent_of_child.borrow_mut().remove_child(&child_clone.clone());
            // Add the container we just made as a child, replacing the one we removed
            parent_of_child.borrow_mut().add_child(container.clone());
            // The new container is now a child of the old child's parent
            container.borrow_mut().add_child(child_.clone());
        }
        trace!("Container created");
        container
    }

    /// Makes a new view. A view holds either a Wayland or an X Wayland window.
    pub fn new_view(parent_: &mut Node, wlc_view: WlcView) -> Node {
        let parent = parent_.borrow();
        if parent.is_root() {
            panic!("View cannot be a direct child of root, not {:?}", parent.get_type());
        }
        let view = Rc::new(RefCell::new(Container {
            handle: Some(Handle::View(wlc_view.clone())),
            parent: Some(Rc::downgrade(&parent_)),
            children: vec!(),
            container_type: ContainerType::View,
            layout: None,
            visible: false,
            is_focused: false,
            is_floating: false
        }));
        if parent.get_type() == ContainerType::Workspace {
            // Case of focused workspace, just create a child of it
            drop(parent);
            let mut parent = parent_.borrow_mut();
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
        self.children.push(container);
        Ok(())
    }

    pub fn add_sibling(&self, container: Node) -> Result<(), &'static str> {
        if self.is_root() {
            return Err("Root has no sibling, cannot add sibling to root");
        }
        let parent = self.get_parent().expect("Could not get parent, was removed from tree");
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

    /// Gets the handle of the container. This could either be an output (a
    /// monitor), or a view (a Wayland or X Wayland Window)
    pub fn get_handle(&self) -> Option<Handle> {
        self.handle.clone()
    }

    /// Gets the layout of the container, if there is one.
    pub fn get_layout(&self) -> Option<Layout> {
        self.layout
    }


    /// Returns true if this container is focused.
    pub fn is_focused(&self) -> bool {
        self.is_focused
    }

    /// Removes the child at the specified index
    pub fn remove_child_at(&mut self, index: usize) -> Result<Node, &'static str> {
        if self.children[index].borrow().get_type() == ContainerType::Workspace {
            return Err("Cannot remove workspace")
        }
        Ok(self.children.remove(index))
    }

    /// Removes the given child from this container's children.
    /// If the child is not present, then an error is returned
    pub fn remove_child(&mut self, node: &Container) -> Result<Node, &'static str> {
        for (index, child) in self.children.clone().iter().enumerate() {
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

    /// Sets the layout of the container.
    pub fn set_layout(&mut self, layout: Layout) {
        self.layout = Some(layout)
    }

    /// Gets the visibility of the container
    pub fn get_visibility(&self) -> bool {
        self.visible
    }

    /// Gets the X and Y dimensions of the container. Width is the first
    /// value, height is the second.
    pub fn get_dimensions(&self) -> Option<(u32, u32)> {
        if let Some(ref handle) = self.handle {
            match handle {
                &Handle::Output(ref output) => {
                    let size = output.get_resolution();
                    Some((size.w, size.h))
                }
                &Handle::View(ref view) => {
                    let ref size = view.get_geometry().expect("View did not have a geometry").size;
                    Some((size.w, size.h))
                }
            }
        } else {
            None
        }
    }

    /// Gets the position of this container on the screen. 
    pub fn get_position(&self) -> Option<(i32, i32)> {
        if let Some(ref handle) = self.handle {
            match handle {
                &Handle::Output(ref output) => None,
                &Handle::View(ref view) => {
                    let ref origin = view.get_geometry().expect("View did not have a geometry").origin;
                    Some((origin.x, origin.y))
                }
            }
        } else {
            None
        }
    }

    /// Returns true if this container is a parent of the child
    pub fn is_parent_of(&self, child: Node) -> bool {
        if self.is_root() {
            true
        } else {
            while ! child.borrow().is_root() {
                let parent = child.borrow().get_parent().expect("Could not get parent, was removed from tree");
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
        if self.get_type() == other.get_type() {
            if let Some(self_handle) = self.get_handle() {
                if let Some(other_handle) = other.get_handle() {
                    if self_handle == other_handle {
                        return true
                    }
                }
            } else {
                return self.get_handle().is_none() && other.get_handle().is_none()
            }
        }
        return false
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
