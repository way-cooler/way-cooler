use std::cmp;

use petgraph::graph::NodeIndex;
use rustwlc::{WlcView, Geometry, Point, Size, ResizeEdge};

use super::super::{LayoutTree, TreeError};
use super::super::commands::CommandResult;
use super::super::core::container::{self, Container, ContainerType, ContainerErr,
                                    Layout, Handle};
use ::layout::core::borders::Borders;
use ::render::Renderable;
use ::debug_enabled;
use uuid::Uuid;


/// The mode the borders can be in. This affects the color primarily.
pub enum Mode {
    /// Borders are active, this means they are focused.
    Active,
    /// Borders are inactive, this means they are not focused.
    Inactive
}

impl LayoutTree {
    /// Sets the borders for the given node to active.
    /// Automatically sets the borders for the ancestor borders as well.
    pub fn set_borders(&mut self, mut node_ix: NodeIndex, focus: Mode)
                              -> CommandResult {
        match self.tree[node_ix] {
            Container::Root(id) |
            Container::Output {id, .. } |
            Container::Workspace {id, .. } =>
                return Err(
                    TreeError::UuidWrongType(id,
                                             vec![ContainerType::View,
                                                  ContainerType::Container])),
            _ => {}
        }
        while self.tree[node_ix].get_type() != ContainerType::Workspace {
            match focus {
                Mode::Active =>
                    if !self.tree.on_path(node_ix) { break },
                Mode::Inactive =>
                    if self.tree.on_path(node_ix)  { break }
            }
            {
                let container = &mut self.tree[node_ix];
                match focus {
                    Mode::Active => container.active_border_color()?,
                    Mode::Inactive => container.clear_border_color()?
                }
                container.draw_borders()?;
            }
            node_ix = self.tree.parent_of(node_ix)?;
        }
        Ok(())
    }
}
