//! A collection of methods that modify the background of the outputs.
use super::super::{Container, ContainerType, LayoutTree, TreeError,
                   MaybeBackground, IncompleteBackground};
use super::super::commands::CommandResult;

use uuid::Uuid;
use wayland_sys::server::wl_client;
use rustwlc::WlcView;
use rustwlc::wayland::wlc_view_get_wl_client;

impl LayoutTree {
    /// Attempts to attach the `bg` to the `outputs`.
    ///
    /// If any of them already have a background attached,
    pub fn attach_background(&mut self, bg: WlcView, output_id: Uuid)
                             -> Result<bool, TreeError> {
        match *self.lookup_mut(output_id)? {
            Container::Output { ref mut background, .. } => {
                match *background {
                    None => {
                        // The background wasn't expecting it
                        Ok(false)
                    }
                    Some(MaybeBackground::Incomplete(incomplete)) => {
                        let client: *mut wl_client;
                        unsafe {
                            client = wlc_view_get_wl_client(bg.0 as _) as _;
                        }
                        *background = Some(incomplete.build(client, bg));
                        Ok(match background.unwrap() {
                            MaybeBackground::Incomplete(_) => false,
                            MaybeBackground::Complete(_) => true,
                        })
                    },
                    _ => Ok(false)
                }
            },
            _ => Err(TreeError::UuidWrongType(output_id,
                                              vec![ContainerType::Output]))
        }
    }

    pub fn attach_incomplete_background(&mut self,
                                        bg: IncompleteBackground,
                                        output_id: Uuid) -> CommandResult {
        match *self.lookup_mut(output_id)? {
            Container::Output { ref mut background, .. } => {
                match *background {
                    None => {
                        *background = Some(bg.into());
                        Ok(())
                    },
                    Some(MaybeBackground::Complete(complete)) => {
                        warn!("Tried to set background while one is still active");
                        warn!("This operation is not allowed, due to a bug with xwayland");
                        Ok(())
                    }
                    _ => {
                        warn!("Tried to set background while the other was still loading!");
                        Ok(())
                    }
                }
            },
            _ => Err(TreeError::UuidWrongType(output_id,
                                              vec![ContainerType::Output]))
        }
    }
}
