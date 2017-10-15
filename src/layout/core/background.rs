//! Background for an output

use rustwlc::WlcView;
use wayland_sys::server::wl_client;

/// A background is not complete until you call the "complete" method on it.
/// This will need to be executed via the view_created callback, because before that
/// we haven't properly set it.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct IncompleteBackground {
    client: usize
}

/// A background for an output.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Background {
    pub handle: WlcView
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum MaybeBackground {
    Incomplete(IncompleteBackground),
    Complete(Background)
}

impl Into<MaybeBackground> for IncompleteBackground {
    fn into(self) -> MaybeBackground {
        MaybeBackground::Incomplete(self)
    }
}

impl Into<MaybeBackground> for Background {
    fn into(self) -> MaybeBackground {
        MaybeBackground::Complete(self)
    }
}

impl IncompleteBackground {
    pub fn new(client: *mut wl_client) -> Self {
        IncompleteBackground { client: client as _ }
    }

    /// Builds the background if the client matches
    /// If it fails, then the incomplete background is returned instead.
    pub fn build(self, client: *mut wl_client, handle: WlcView)
                 -> MaybeBackground {
        if self.client as *mut wl_client == client {
            Background { handle }.into()
        } else {
            self.into()
        }

    }
}
