use std::ops::{Deref, DerefMut};

use uuid::Uuid;
use rustwlc::WlcView;

use ::layout::{Tree, Layout, TreeError};

type TestTreeResult = Result<UnitTestTree, TreeError>;

/// A tree that has some utility methods to aid in creating unit tests.
#[derive(Debug)]
pub struct UnitTestTree(Tree);

impl UnitTestTree {
    pub fn new() -> Self {
        UnitTestTree (
            Tree::new()
        )
    }

    /// Attempts to add an output to the tree.
    pub fn add_output(mut self) -> TestTreeResult {
        let fake_output = WlcView::root().as_output();
        self.0.add_output(fake_output)?;
        Ok(self)
    }

    /// Switches to a workspace, which is the default workspace
    /// for all the subsequent operations.
    ///
    /// NOTE If you don't add anything to this workspace, it will be
    /// removed if you switch to another one.
    pub fn switch_to_workspace(mut self, name: &str) -> TestTreeResult {
        self.0.switch_to_workspace(name)?;
        assert_eq!(self.0.current_workspace()?, name);
        Ok(self)
    }

    /// Sets the active container to the given layout.
    pub fn set_active_layout(mut self, layout: Layout) -> TestTreeResult {
        self.0.set_active_layout(layout)?;
        Ok(self)
    }

    /// Adds a view, and sets it as the active container
    pub fn add_view(mut self) -> TestTreeResult {
        let fake_view = WlcView::root();
        self.0.add_view(fake_view)?;
        Ok(self)
    }

    /// Focuses on the `Container` associated with the UUID.
    ///
    /// If the container is floating, it is not set to be the active container,
    /// per the spec.
    pub fn focus_on(mut self, id: Uuid) -> TestTreeResult {
        self.0.focus(id)?;
        Ok(self)
    }
}

impl Deref for UnitTestTree {
    type Target = Tree;

    fn deref(&self) -> &Tree {
        &self.0
    }
}

impl DerefMut for UnitTestTree {
    fn deref_mut(&mut self) -> &mut Tree {
        &mut self.0
    }
}

/// Makes a very basic tree. This is sufficient for basic tests.
/// There is only one output,
/// Two workspaces,
/// First workspace has a single view in the root container,
/// second workspace has a container with two views in it
/// (the container is a child of the root container).
///
/// The active container is the only view in the first workspace
pub fn basic_tree() -> UnitTestTree {
    UnitTestTree::new()
        .add_output().unwrap()
        .switch_to_workspace("1").unwrap()
        .add_view().unwrap()
        .switch_to_workspace("2").unwrap()
        .add_view().unwrap()
        .set_active_layout(Layout::Horizontal).unwrap()
        .add_view().unwrap()
        .switch_to_workspace("1").unwrap()
}

