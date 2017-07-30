use std::fmt::{Debug, Formatter};
use std::fmt::Result as FmtResult;
use super::KeyEvent;

/// An action performed by the key binding.
/// Contains the event that is performed (e.g what Lua or Rust function
/// to call), and meta data around the action such as whether to include
/// passthrough to the client or not.
#[derive(Clone)]
pub struct Action {
    pub event: KeyEvent,
    pub passthrough: bool
}

impl Debug for Action {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "Event: {:?}, passthrough: {:?} ",
               self.event, self.passthrough)
    }
}
