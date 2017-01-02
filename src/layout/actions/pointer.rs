use rustwlc::{input, Point, ResizeEdge, Geometry};

use super::super::{LayoutTree, TreeError};
use super::super::commands::{CommandResult};
use uuid::Uuid;

impl LayoutTree {
    /// Sets the absolute position of the cursor on the screen.
    pub fn set_pointer_pos(&mut self, point: Point) -> CommandResult {
        input::pointer::set_position(point);
        Ok(())
    }
}
