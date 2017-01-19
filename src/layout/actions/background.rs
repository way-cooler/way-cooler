//! A collection of methods that modify the background of the outputs.
use super::super::{Container, ContainerType, LayoutTree, TreeError};
use super::super::commands::CommandResult;

use uuid::Uuid;
use rustwlc::{WlcView};

use std::collections::HashSet;

impl LayoutTree {
    /// Attempts to attach the `bg` to the `outputs`.
    ///
    /// If any of them already have a background attached,
    /// ``
    pub fn attach_background(&mut self, bg: WlcView, outputs: &[Uuid])
                             -> CommandResult {
        // TODO Remove this to remove when the wlc bug is fixed
        // https://github.com/Cloudef/wlc/issues/221
        let mut to_remove = HashSet::with_capacity(outputs.len());
        for output_id in outputs {
            match *try!(self.lookup_mut(*output_id)) {
                Container::Output { ref mut background, .. } => {
                    if background.is_none() {
                        *background = Some(bg);
                    } else {
                        // TODO This can't be used right now, see this bug:
                        // https://github.com/Cloudef/wlc/issues/221
                        /*return Err(TreeError::Background(
                            BackgroundErr::AlreadyAttached(*output_id, background.clone().unwrap())))*/
                        if let Some(view) = background.take() {
                            to_remove.insert(view);
                        }
                        *background = Some(bg);
                    }
                },
                _ => return Err(TreeError::UuidWrongType(*output_id,
                                                         vec![ContainerType::Output]))
            }
        }
        for view in to_remove {
            view.close();
        }
        Ok(())
    }
}
