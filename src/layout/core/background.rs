//! Background for an output

use rustwlc::WlcView;
use wayland_sys::server::wl_client;

/// A background is not complete until you call the "complete" method on it.
/// This will need to be executed via the view_created callback, because before that
/// we haven't properly set it.
#[derive(Debug)]
pub struct IncompleteBackground {
    client: *mut wl_client
}

#[derive(Debug)]
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
        IncompleteBackground { client }
    }

    /// Builds the background if the client matches
    /// If it fails, then the incomplete background is returned instead.
    pub fn build(self, client: *mut wl_client, handle: WlcView)
                 -> Result<Background, IncompleteBackground> {
        if self.client == client {
            Ok(Background { handle })
        } else {
            Err(self)
        }

    }
}

/// A background for an output.
#[derive(Debug)]
pub struct Background {
    handle: WlcView
}
