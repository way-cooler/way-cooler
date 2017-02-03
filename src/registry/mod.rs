//! way-cooler registry.

use std::collections::hash_map::{HashMap};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use rustc_serialize::json::{Json};

mod registry;
mod category;
mod client;

use self::registry::Registry;
pub use self::registry::{ReadHandle, WriteHandle};

#[cfg(test)]
pub mod tests;

pub use self::client::{Client, Clients, Permissions};
use uuid::Uuid;

lazy_static! {
    /// Static HashMap for the registry
    static ref REGISTRY: RwLock<Registry> = RwLock::new(Registry::new());
    static ref CLIENTS: RwLock<Clients> = RwLock::new(Clients::new());
}

/// Error types that can happen
#[derive(Debug, PartialEq)]
pub enum RegistryError {
    /// The registry key was not found
    KeyNotFound,
}

/// Result type of gets/sets to the registry
pub type RegistryResult<T> = Result<T, RegistryError>;

pub fn clients_write<'a>() -> RwLockWriteGuard<'a, Clients> {
    CLIENTS.write().expect("Unable to write client mapping")
}

pub fn clients_read<'a>() -> RwLockReadGuard<'a, Clients> {
    CLIENTS.read().expect("Unable to read client mapping")
}

/// Initialize the registry and client mapping
pub fn init() {
    let mut registry = REGISTRY.write()
        .expect("Could not write to the registry");
    // Construct the layout category
    registry.add_category("windows".into());
    // Construct the programs category
    registry.add_category("programs".into());
}
