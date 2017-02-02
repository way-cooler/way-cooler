//! way-cooler registry.

use std::ops::Deref;
use std::collections::hash_map::{HashMap, Entry};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use rustc_serialize::Decodable;
use rustc_serialize::json::{Json, Decoder, DecoderError};

mod types;
pub use self::types::*; // Export constants too

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
}

/// Error types that can happen
#[derive(Debug, PartialEq)]
pub enum RegistryError {
    /// The registry key was not found
    KeyNotFound,
}

/// Result type of gets/sets to the registry
pub type RegistryResult<T> = Result<T, RegistryError>;

/// Initialize the registry (register default API)
pub fn init() {
}

/// Acquires a read lock on the registry.
pub fn read_lock<'a>() -> RwLockReadGuard<'a, RegMap> {
    REGISTRY.read().expect("Unable to read from registry!")
}

/// Acquires a write lock on the registry.
pub fn write_lock<'a>() -> RwLockWriteGuard<'a, RegMap> {
    REGISTRY.write().expect("Unable to write to registry!")
}

/// Gets a RegistryField enum with the specified name
pub fn get_field(name: &str) -> Option<Arc<Json>> {
    let result = read_lock().get(name).map(|v| v.clone());
    return result;
}

/// Gets a data type from the registry, returning a reference to the property
/// method if the field is a property.
pub fn get_data(name: &str) -> RegistryResult<Arc<Json>> {
    get_field(name).ok_or(RegistryError::KeyNotFound)
}

/// Get a Rust structure from the registry
#[allow(dead_code)]
pub fn get_struct<T>(name: &str)
                     -> RegistryResult<Result<T, DecoderError>>
where T: Decodable {
    get_data(name).map(|json| T::decode(&mut Decoder::new(json.deref().clone())))
}

/// Add the json object to the registry
pub fn insert_json(key: String, value: Json) -> Option<Arc<Json>> {
    write_lock().insert(key, Arc::new(value))
}

/// Set a value to the given JSON value.
pub fn set_json(key: String, json: Json) -> RegistryResult<Arc<Json>> {
    let mut reg = write_lock();
    match reg.entry(key) {
        Entry::Vacant(_vacancy) => {
            return Err(RegistryError::KeyNotFound)
        },
        Entry::Occupied(mut entry) => {
            Ok(entry.insert(Arc::new(json)))
        }
    }
}
