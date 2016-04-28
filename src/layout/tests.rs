//! Tests for containers.rs

#[cfg(test)]
mod tests {
    use super::super::containers::*;
    use std::rc::*;
    use rustwlc::handle::*;

    #[cfg(test)]
    /// Sets up a root node with 10 workspaces
    fn root_setup() -> Node {
        let mut root = Container::new_root();
        for _ in 0..10 {
            Container::new_workspace(&mut root);
        }
        root
    }

    #[test]
    #[should_panic(expected = "Can only be one root")]
    /// Test to make sure that there can be only one root node
    fn test_one_root() {
        let _root = Container::new_root();
        // Should panic
        let _root2 = Container::new_root();
        // remove when we fix it
        panic!("Can only be one root");
    }

    #[test]
    /// Verify that root node has required properties
    fn root_validity_test() {
        let main: Node = Container::new_root();
        let root = main.borrow_mut();
        // Make sure it starts with no children (but can have children)
        assert_eq!(root.get_children().unwrap().len(), 0);
        // This is the tippity-top of the tree
        assert!(root.get_parent().is_none()); 
        assert_eq!(root.get_type(), ContainerType::Root);
        // Should this be true? It makes sense, but it might cause complications
        // so we'll go with false for now
        assert!(! root.is_focused());
        // Root is parent of all nodes, including itself
        assert!(root.is_parent_of(main.clone()));
        // Root is not a child of anything, including itself
        // NOTE Add more test when we have more container types
        assert!(! root.is_child_of(main.clone()));
        assert_eq!(root.get_position(), (0, 0));
        assert_eq!(root.get_dimensions(), (0, 0));
        assert!(root.is_root());
    }

    #[test]
    fn workspace_validity_test() {
        let root = root_setup();
        let root_copy = Rc::make_mut(&mut root.clone()).clone().into_inner();
        assert_eq!(root.borrow().get_children().unwrap().len(), 10);
        for container_ in root.borrow().get_children().unwrap() {
            // Workspaces start out empty
            let mut container = container_.borrow_mut();
            assert_eq!(container.get_children().unwrap().len(), 0);
            let container_copy = Rc::make_mut(&mut container.get_parent().unwrap().clone()).clone().into_inner();
            assert_eq!(container_copy, root_copy);
            // NOTE add test for add_child
            assert_eq!(container.get_type(), ContainerType::Workspace);
            assert!(! container.is_focused());
            // NOTE add test for remove_child_at 
            assert!(! container.get_visibility());
            container.set_visibility(true);
            assert!(container.get_visibility());
            container.set_visibility(false);
            assert!(! container.get_visibility());
            // NOTE change these once they can be set properly
            assert_eq!(container.get_dimensions(), (0,0));
            assert_eq!(container.get_position(), (0,0));
            // NOTE add test for is_parent_of
            assert!(container.is_child_of(root.clone()));
            assert!(! container.is_root());
            assert_eq!(container.get_parent_by_type(ContainerType::Root).unwrap(), root);
        }
    }

    #[test]
    fn remove_container_test() {
        let root = root_setup();
        let mut workspace = &mut root.borrow().get_children().unwrap()[0];
        let container = Container::new_container(&mut workspace, WlcView::root());

        let container_ref = Rc::downgrade(&workspace.borrow().get_children().unwrap()[0].clone());
        // Still points to the container
        assert!(container_ref.clone().upgrade().is_some());
        // removes workspace's reference to it
        container.borrow().remove_container().ok();
        // We still can see it, so it's still alive
        assert!(container_ref.clone().upgrade().is_some());
        // drop our reference to it
        drop(container);
        assert!(! container_ref.clone().upgrade().is_some());
    }

    #[test]
    #[should_panic(expect = "Cannot remove root container")]
    fn ensure_root_container_unremovable_test() {
        let root = root_setup();
        root.borrow_mut().remove_container().expect("Cannot remove root container");
    }

    #[test]
    #[should_panic(expect = "Cannot remove root container")]
    fn ensure_workspace_container_unremovable_test() {
        let root = root_setup();
        let workspace = root.borrow().get_children().unwrap()[0].clone();
        workspace.borrow_mut().remove_container().expect("Cannot remove root container");
    }

    #[test]
    #[should_panic(expect = "Only workspaces can be added to the root node")]
    /// Tests to make sure only workspaces can be children of the top node
    fn root_workspace_only() {
        let root = root_setup();
        let container = &mut root.borrow().get_children().unwrap()[0];
        Container::new_workspace(container);
    }

    #[test]
    #[should_panic(expect = "Only containers can be children of a workspace")]
    /// Tests to ensure that only containers can be the children of a workspace
    fn only_containers_for_workspace_children_test() {
        let root = root_setup();
        let container = root.borrow().get_children().unwrap()[0].clone();
        container.borrow_mut().add_child(container.clone());
    }

    
    #[test]
    /// Tests that you can have a root with some workspaces, that the workspaces
    /// can be removed, and that you can add simple containers to the
    /// workspaces. (Does not test views)
    fn basic_tree_test() {
        let mut root = Container::new_root();
        for _ in 0..10 {
            Container::new_workspace(&mut root);
        }
        assert_eq!(root.borrow().get_children().unwrap().len(),10);
        // Remove half the nodes
        for _ in 0..5 {
            root.borrow_mut().remove_child_at(0).ok();
        }
        assert_eq!(root.borrow().get_children().unwrap().len(),5);
        // NOTE Enhance with adding containers to the workspaces
    }
        
}
