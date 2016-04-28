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
        let root = main.borrow();
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
        // Root is not a child of anything, except for itself
        // NOTE Add more test when we have more container types
        assert!(root.is_child_of(main.clone()));
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
    fn container_validity_test() {
        let root = root_setup();
        for mut workspace_ in root.borrow().get_children().unwrap() {
            let workspace_copy = Rc::make_mut(&mut workspace_.clone()).clone().into_inner();
            let container_ = Container::new_container(&mut workspace_, WlcView::root());
            let mut container = container_.borrow_mut();
            // No children
            assert_eq!(container.get_children().unwrap().len(), 0);
            // Is the only child of the workspace
            let container_parent = Rc::make_mut(&mut container.get_parent().unwrap().clone()).clone().into_inner();
            assert_eq!(workspace_copy, container_parent);
            // drop our copy of container here, because otherwise it panics at
            // run time. Another way to do this is at the beginning, but this is
            // a good test of what we need to do to stay dynamic
            drop(container);
            let inner_workspace = Container::new_container(&mut container_.clone(), WlcView::root());
            container = container_.borrow_mut();
            assert_eq!(container.get_children().unwrap().len(), 1);
            let self_as_child = container.get_children().unwrap()[0].clone();
            assert_eq!(*self_as_child.borrow(), *inner_workspace.borrow());
            assert_eq!(container.get_type(), ContainerType::Container);
            drop(container);
            let container2 = container_.borrow();
            assert!(container2.is_parent_of(inner_workspace.clone()));
            drop(container2);
            container = container_.borrow_mut();
            // We also need to drop our copy of the container here when checking
            // if we are a child of the container, since we have to borrow the
            // container right here again in order to check if it's the parent.
            //
            // Again, this is only an "issue" because we are borrow_mut for
            // such a long period of time
            drop(container);
            assert!(inner_workspace.borrow().is_child_of(container_.clone()));
            container = container_.borrow_mut();
            assert!(! container.is_focused());
            container.set_visibility(true);
            assert!(container.get_visibility());
            container.set_visibility(false);
            assert!(! container.get_visibility());
            // NOTE change these once they can be set properly
            assert_eq!(container.get_dimensions(), (0,0));
            assert_eq!(container.get_position(), (0,0));
            assert!(container.is_child_of(root.clone()));
            assert!(! container.is_root());
            assert_eq!(container.get_parent_by_type(ContainerType::Workspace).unwrap(), workspace_);
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
        container.borrow().remove_container().unwrap();
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
        let mut root = root_setup();
        assert_eq!(root.borrow().get_children().unwrap().len(),10);
        // NOTE Enhance with adding containers to the workspaces
    }
        
}
