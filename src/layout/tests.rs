//! Tests for containers.rs

#[cfg(test)]
mod tests {
    use rustwlc::handle::{WlcView, WlcOutput};
    use std::rc::*;
    use super::super::containers::*;

    #[test]
    #[should_panic(expected = "Can only be one root")]
    /// Test to make sure that there can be only one root node
    fn test_one_root() {
        let root = Root::new_root();
        // Should panic
        let root2 = Root::new_root();
    }

    #[test]
    /// Verify that root node has required properties
    fn root_validity() {
        let root = Root::new_root();
        // Make sure it starts with no children
        // This will change when we add Workspaces when we make the root
        assert!(root.get_children().is_none());
        // This is the tippity-top of the tree
        assert!(root.get_parent().is_none()); 
        assert_eq!(root.get_type(), ContainerType::Root);
        // Should this be true? It makes sense, but it might cause complications
        // so we'll go with false for now
        assert!(! root.is_focused());
        // Root is parent of all nodes, including itself
        assert!(root.is_parent_of(root.clone()));
        // Root is not a child of anything, including itself
        // NOTE Add more test when we have more container types
        assert!(! root.is_child_of(root.clone()));
        assert!(root.is_root());
    }

    #[test]
    #[should_panic(expected = "Cannot remove the root of the tree")]
    /// Ensures you cannot remove the root node
    fn remove_root_test() {
        let root = Root::new_root();
        root.remove_container();
    }

    #[test]
    #[should_panic(expected = "Root has no dimensions")]
    fn get_root_dimensions_test() {
        let root = Root::new_root();
        root.get_dimensions();
    }
    
    #[test]
    #[should_panic(expected = "Root has no position")]
    fn get_root_position_test() {
        let root = Root::new_root();
        root.get_position();
    }

    #[test]
    fn workspace_validity_test() {
        let root = Root::new_root();
        for _ in 1..10 {
            Workspace::new_workspace(root.clone());
        }
        assert_eq!(root.get_children().unwrap().len(),10);
    }
}
