//! It's a tree!

use std::ptr;
use std::ops::{Deref, DerefMut};
use std::fmt::Debug;

#[derive(Debug, PartialEq, Eq)]
struct Node<T> {
    // We need a mut pointer so we can modify the parent
    parent: *mut Node<T>,
    val: T,
    children: Vec<Node<T>>
}

impl<T> Node<T> {
    /// Create a new node with the existing value.
    /// For root-style constructors.
    pub fn new(val: T) -> Node<T> {
        Node {
            parent: ptr::null_mut(),
            val: val,
            children: Vec::new()
        }
    }

    /// Add a new child node to this node, using a value
    pub fn new_child(&mut self, val: T) -> &mut Node<T> {
        let self_mut = self as *mut Node<T>;
        self.children.push(Node {
            parent: self_mut,
            val: val,
            children: Vec::new()
        });
        let last_ix = self.children.len() -1;
        &mut self.children[last_ix]
    }

    /// Whether this node has a (currently-reachable) parent
    pub fn has_parent(&self) -> bool {
        self.parent.is_null()
    }

    /// Gets the parent of this node (if it exists)
    pub fn get_parent(&self) -> Option<&mut Node<T>> {
        if self.parent.is_null() {
            return None;
        }
        unsafe {
            return Some(&mut *self.parent);
        }
    }

    /// Borrow the children of this node.
    pub fn get_children(&self) -> &[Node<T>] {
        &self.children
    }

    /// Mutably borrow the children of this mutable node
    pub fn get_mut_children(&mut self) -> &mut[Node<T>] {
        &mut self.children
    }

    /// Remove a child at the given index
    pub fn remove_child_at(&mut self, index: usize) -> Node<T> {
        let mut child = self.children.remove(index);
        child.parent = ptr::null_mut();
        child
    }

    /// Whether this node is a parent of another node
    pub fn is_parent_of(&self, other: &Node<T>) -> bool {
        self.parent == other.parent as *mut Node<T>
    }
}

impl <T: PartialEq> Node<T> {

    /// Remove a node from its parent.
    /// This method will mutate the parent if it exists.
    pub fn remove_from_parent(&mut self) {
        if self.get_parent().is_none() {
            return;
        }
        if let Some(index) = self.children.iter().position(|c| c == self) {
            self.parent = ptr::null_mut();
            self.children.remove(index);
        }
    }

    /// Removes a child from self
    pub fn remove_child(&mut self, other: &Node<T>) -> Option<Node<T>> {
        if let Some(index) = self.children.iter().position(|c| c == other) {
            let mut child = self.children.remove(index);
            child.parent = ptr::null_mut();
            Some(child)
        }
        else {
            None
        }
    }

    pub fn move_to(mut self, new_parent: &mut Node<T>) {
        self.remove_from_parent();
        self.parent = new_parent as *mut Node<T>;
        new_parent.children.push(self);
    }
}

impl<T> Drop for Node<T> {
    fn drop(&mut self) {
        println!("Dropping a node.");
        let children: &mut Vec<Node<T>> = &mut self.children;
        for mut child in children {
            child.parent = ptr::null_mut();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Node;
    use log::LogLevelFilter;
    use env_logger::LogBuilder;

    /// Nodes can have children added to them
    #[test]
    fn test_add_child() {
        let mut root = Node::new(0);
        root.new_child(1);
        root.new_child(2); // This is okay
        {
            let mut third_child = root.new_child(3);
            //root.new_child(4); // Have to wait for 3rd child to drop
        }
        root.new_child(4); // Now this works yay standard borrowing
        assert_eq!(root.children.len(), 4);
        println!("Done with the test now.");
    }

    /// These operations will for example operate on the parent
    /// under an if let. `remove_from_parent` will not panic if the node
    /// already is parentless, for example.
    #[test]
    fn optional_operations() {
        
    }
}
