//! A collection of methods that modify the background of the outputs.
use super::super::{Container, ContainerType, LayoutTree, TreeError};
use super::super::commands::CommandResult;

use uuid::Uuid;
use rustwlc::{WlcView};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackgroundErr {
    /// A background (the `WlcView`) is already attached to the output (`UUID`)
    AlreadyAttached(Uuid, WlcView)
}

impl LayoutTree {
    /// Attempts to attach the `bg` to the `outputs`.
    ///
    /// If any of them already have a background attached,
    /// ``
    pub fn attach_background(&mut self, bg: WlcView, outputs: &[Uuid])
                             -> CommandResult {
        for output_id in outputs {
            match *try!(self.lookup_mut(*output_id)) {
                Container::Output { ref mut background, .. } => {
                    if background.is_none() {
                        *background = Some(bg);
                    } else {
                        error!("HERE");
                        error!("{:?}", background.clone());
                        return Err(TreeError::Background(
                            BackgroundErr::AlreadyAttached(*output_id, background.clone().unwrap())))
                    }
                },
                _ => return Err(TreeError::UuidWrongType(*output_id,
                                                         vec![ContainerType::Output]))
            }
        }
        Ok(())
    }
}
