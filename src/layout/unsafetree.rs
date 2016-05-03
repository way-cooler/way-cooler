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
    pub fn new_child(&mut self, val: T) {
        let self_mut = self as *mut Node<T>;
        self.children.push(Node {
            parent: self_mut,
            val: val,
            children: Vec::new()
        })
    }

    /// Whether this node has a (currently-reachable) parent
    pub fn has_parent(&self) -> bool {
        self.parent.is_null()
    }

    /// Gets the parent of this node (if it exists)
    pub fn get_parent<'a>(&'a self) -> Option<&'a mut Node<T>> {
        if self.parent.is_null() {
            return None;
        }
        unsafe {
            return Some(&mut *self.parent);
        }
    }

    /// Borrow the children of this node.
    pub fn get_children<'a>(&'a self) -> &Vec<Node<T>> {
        &self.children
    }

    /// Mutably borrow the children of this mutable node
    pub fn get_mut_children(&mut self) -> &mut Vec<Node<T>> {
        &mut self.children
    }

    /// Remove a child at the given index
    pub fn remove_child_at(&mut self, index: usize) -> Node<T> {
        self.children.remove(index)
    }

    /// Whether this node is a parent of another node
    pub fn is_parent_of(&self, other: Node<T>) -> bool {
        self.parent == other.parent
    }
}

impl <T: PartialEq> Node<T> {
    pub fn remove_from_parent(&self) {
        if let Some(ref mut parent) = self.get_parent() {
            let children = parent.get_mut_children();
            if let Some(index) = children.iter().position(|c| c == self) {
                children.remove(index);
            }
        }
    }

    pub fn remove_child(&mut self, other: &Node<T>) -> Option<Node<T>> {
        if let Some(index) = self.children.iter().position(|c| c == other) {
            Some(self.children.remove(index))
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
        trace!("Dropping a node!!!!");
        self.parent = ptr::null_mut();
    }
}

#[cfg(test)]
mod tests {
    use super::Node;

    fn test_add_child() {
        
    }
}
