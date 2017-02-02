//! way-cooler registry.

use std::collections::hash_map::{HashMap};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use rustc_serialize::json::{Json};

mod registry;
mod category;
mod permissions;

use self::registry::Registry;

#[cfg(test)]
pub mod tests;

pub type RegMap = HashMap<String, Arc<Json>>;

#[cfg(not(test))]
#[inline]
fn new_map() -> RegMap {
    HashMap::new()
}

#[cfg(test)]
#[inline]
fn new_map() -> RegMap {
    self::tests::registry_map()
}

lazy_static! {
    /// Static HashMap for the registry
    static ref REGISTRY: RwLock<RegMap> = RwLock::new(new_map());
    static ref REGISTRY2: RwLock<Registry> = RwLock::new(Registry::new());
}

/// Error types that can happen
#[derive(Debug, PartialEq)]
pub enum RegistryError {
    /// The registry key was not found
    KeyNotFound,
}

/// Result type of gets/sets to the registry
pub type RegistryResult<T> = Result<T, RegistryError>;

/// Acquires a read lock on the registry.
fn read_lock<'a>() -> RwLockReadGuard<'a, RegMap> {
    REGISTRY.read().expect("Unable to read from registry!")
}

/// Initialize the registry (register default API)
pub fn init() {
}

/// Acquires a write lock on the registry.
fn write_lock<'a>() -> RwLockWriteGuard<'a, RegMap> {
    REGISTRY.write().expect("Unable to write to registry!")
}

/// Gets a data type from the registry
pub fn get_data(key: &str) -> RegistryResult<Arc<Json>> {
    read_lock().get(key).cloned().ok_or(RegistryError::KeyNotFound)
}

pub fn set_data(key: String, value: Json) -> RegistryResult<Option<Arc<Json>>> {
    Ok(write_lock().insert(key, Arc::new(value)))
}
