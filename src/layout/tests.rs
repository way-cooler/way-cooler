//! Tests for containers.rs

#[cfg(test)]
mod tests {
    use super::super::containers::*;

    #[test]
    #[should_panic(expected = "Can only be one root")]
    /// Test to make sure that there can be only one root node
    fn test_one_root() {
        let _root = Container::new_root();
        // Should panic
        let _root2 = Container::new_root();
    }

    #[test]
    /// Verify that root node has required properties
    fn root_validity() {
        let root = Container::new_root();
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
        assert_eq!(root.get_position(), (0, 0));
        assert_eq!(root.get_dimensions(), (0, 0));
        assert!(root.is_root());
    }

    #[test]
    fn workspace_validity_test() {
        let root = Container::new_root();
        for _ in 1..10 {
            Container::new_workspace(root.clone());
        }
        assert_eq!(root.get_children().unwrap().len(),10);
    }
}
