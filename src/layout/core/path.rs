use std::cmp::{PartialOrd, Ord, Ordering};
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
/// A path through the tree, holds metadate associated with nodes
/// and their connections to other nodes.
pub struct Path {
    /// The weight used to order this `Path` with other `Paths`.
    pub weight: u32,
    /// A flag to indicate if this path has been active lately.
    /// This allows the tree to 'remember' recently used paths.
    ///
    /// The closer it is to 0, the more recently it was made active,
    /// with 0 meaning it is currently the active path.
    /// e.g 1 means it was just the active path.
    pub active: u32
}

impl Path {
    /// Constructs a new Path, with the given weight and active number.
    pub fn new(weight: u32, active: u32) -> Self {
        Path { weight: weight, active: active }
    }

    /// Returns an active path with a weight of zero and an active number of 0.
    pub fn zero() -> Self {
        Path { weight: 0, active: 0 }
    }

    /// Determines if the path is active.
    /// The path is active when self.active == 0
    pub fn is_active(&self) -> bool {
        self.active == 0
    }
}


impl Ord for Path {
    fn cmp(&self, other: &Self) -> Ordering {
        self.weight.cmp(&other.weight)
    }
}

impl PartialOrd for Path {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Deref for Path {
    type Target = u32;

    fn deref(&self) -> &u32 {
        &self.weight
    }
}

impl DerefMut for Path {
    fn deref_mut(&mut self) -> &mut u32 {
        &mut self.weight
    }
}
