//! Tests for containers.rs

#[cfg(test)]
mod tests {
    use super::super::containers::*;
    use std::rc::*;
    use rustwlc::handle::*;
    use rustwlc::types::*;

    #[cfg(test)]
    /// Sets up a root node with 10 workspaces
    fn root_setup() -> Node {
        let mut root = Container::new_root();
        let mut output = Container::new_output(&mut root, WlcView::root().as_output());
        for _ in 0..10 {
            Container::new_workspace(&mut output);
        }
        root
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
        assert!(root.get_position().is_none());
        assert!(root.get_dimensions().is_none());
        assert!(root.is_root());
    }

    #[test]
    fn workspace_validity_test() {
        let root = root_setup();
        assert_eq!(root.borrow().get_children().unwrap().len(), 1);
        let output = &mut root.borrow().get_children().unwrap().to_vec()[0];
        let output_copy = Rc::make_mut(&mut output.clone()).clone().into_inner();
        for container_ in output.borrow().get_children().unwrap().to_vec() {
            // Workspaces start out empty
            let mut container = container_.borrow_mut();
            assert_eq!(container.get_children().unwrap().len(), 0);
            let container_copy = Rc::make_mut(&mut container.get_parent().unwrap().clone()).clone().into_inner();
            assert_eq!(container_copy, output_copy);
            // NOTE add test for add_child
            assert_eq!(container.get_type(), ContainerType::Workspace);
            assert!(! container.is_focused());
            // NOTE add test for remove_child_at 
            assert!(! container.get_visibility());
            container.set_visibility(true);
            assert!(container.get_visibility());
            container.set_visibility(false);
            assert!(! container.get_visibility());
            // NOTE Add these once they can be set properly
            //assert_eq!(container.get_dimensions().unwrap(), (0,0));
            //assert_eq!(container.get_position().unwrap(), (0,0));
            // NOTE add test for is_parent_of
            assert!(container.is_child_of(root.clone()));
            assert!(! container.is_root());
            assert_eq!(container.get_parent_by_type(ContainerType::Root).unwrap(), root);
        }
    }

    #[test]
    fn container_validity_test() {
        let root = root_setup();
        for output in root.borrow().get_children().unwrap().to_vec() {
            let mut workspace_ = &mut output.borrow().get_children().unwrap().to_vec()[0];
            let workspace_copy = Rc::make_mut(&mut workspace_.clone()).clone().into_inner();
            let container_ = Container::new_container(&mut workspace_, Layout::Horizontal);
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
            // This new workspace will now be the outer one, the old workspace is now on the inside
            let outer_container = Container::new_container(&mut container_.clone(), Layout::Horizontal);
            container = container_.borrow_mut();
            assert_eq!(outer_container.borrow().get_children().unwrap().len(), 1);
            assert_eq!(container.get_children().unwrap().len(), 0);
            let self_as_child = outer_container.borrow().get_children().unwrap()[0].clone();
            drop(container);
            assert_eq!(*self_as_child.borrow(), *outer_container.borrow());
            let container = container_.borrow();
            assert_eq!(container.get_type(), ContainerType::Container);
            drop(container);
            let container2 = container_.borrow();
            assert!(container2.is_parent_of(outer_container.clone()));
            drop(container2);
            let mut container = container_.borrow_mut();
            // We also need to drop our copy of the container here when checking
            // if we are a child of the container, since we have to borrow the
            // container right here again in order to check if it's the parent.
            //
            // Again, this is only an "issue" because we are borrow_mut for
            // such a long period of time
            drop(container);
            assert!(outer_container.borrow().is_child_of(container_.clone()));
            container = container_.borrow_mut();
            assert!(! container.is_focused());
            container.set_visibility(true);
            assert!(container.get_visibility());
            container.set_visibility(false);
            assert!(! container.get_visibility());
            // NOTE add these once they can be set properly
            //assert_eq!(container.get_dimensions().unwrap(), (0,0));
            //assert_eq!(container.get_position().unwrap(), (0,0));
            assert!(container.is_child_of(root.clone()));
            assert!(! container.is_root());
            assert_eq!(container.get_parent_by_type(ContainerType::Workspace).unwrap(), *workspace_);
            assert_eq!(container.get_parent_by_type(ContainerType::Root).unwrap(), root);
        }
    }

    #[test]
    fn view_validity_test() {
        let root = root_setup();
        let output = &root.borrow().get_children().unwrap().to_vec()[0];
        for mut workspace_ in output.borrow().get_children().unwrap().to_vec() {
            let mut container_ = Container::new_container(&mut workspace_, Layout::Horizontal);
            // hack to give it a size and origin point.
            let view_hack = WlcView::root();
            view_hack.set_geometry(EDGE_NONE, &Geometry { origin: Point { x: 0, y: 0}, size: Size { w: 0, h: 0}});
            let view = Container::new_view(&mut container_, view_hack);
            assert_eq!(view.borrow().get_parent().unwrap(), container_);
            // Add child test
            // The view was added as a sibling, so it's not a child of the container
            assert_eq!(container_.borrow().get_children().unwrap().len(), 0);
            view.borrow_mut().add_sibling(container_.clone()).unwrap();
            assert_eq!(container_.borrow().get_children().unwrap().len(), 1);
        }
    }

    #[test]
    #[should_panic(expect = "Cannot add child to a view")]
    fn ensure_views_cant_have_children_test() {
        let root = root_setup();
        let mut workspace = root.borrow_mut().get_children().unwrap().to_vec()[0].clone();
        let view = Container::new_view(&mut workspace, WlcView::root());
        view.borrow_mut().add_child(workspace).unwrap();
    }

    #[test]
    fn remove_container_test() {
        let root = root_setup();
        let output = &mut root.borrow().get_children().unwrap().to_vec()[0];
        let workspace = &mut output.borrow().get_children().unwrap().to_vec()[0];
        let container = Container::new_container(workspace, Layout::Horizontal);

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
    #[should_panic(expect = "Only containers can be children of a workspace")]
    /// Tests to ensure that only containers can be the children of a workspace
    fn only_containers_for_workspace_children_test() {
        let root = root_setup();
        let output = &root.borrow().get_children().unwrap().to_vec()[0];
        let container = output.borrow().get_children().unwrap()[0].clone();
        container.borrow_mut().add_child(container.clone()).unwrap();
    }

    
    #[test]
    /// Tests that you can have a root with some workspaces, that the workspaces
    /// can be removed, and that you can add simple containers to the
    /// workspaces. (Does not test views)
    fn basic_tree_test() {
        let mut root = root_setup();
        assert_eq!(root.borrow().get_children().unwrap().len(), 1);
        let output = root.borrow().get_children().unwrap()[0].clone();
        assert_eq!(output.borrow().get_children().unwrap().len(), 10);
        // NOTE Enhance with adding containers to the workspaces
    }
        
}
