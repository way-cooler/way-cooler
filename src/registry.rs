//! way-cooler registry.

use std::collections::HashMap;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub type RegKey = String;
pub type RegVal = RegistryValue;
pub type RegMap = HashMap<RegKey, RegVal>;

lazy_static! {
    /// Registry variable for the registry
    static ref REGISTRY: RwLock<RegMap> =
        RwLock::new(HashMap::new());
}

/// A value in the registry.
#[derive(Debug, Clone)]
pub enum RegistryValue {
    /// An integer value
    Integer(i32),
    /// A floating point value
    Float(f32),
    /// A boolean value
    Boolean(bool),
    /// A string value
    String(String),
    /// A list of values.
    List(Vec<RegistryValue>)
}

/// Acquires a read lock on the registry.
fn read_lock<'a>() -> RwLockReadGuard<'a, RegMap> {
    REGISTRY.read().unwrap()
}

/// Acquires a write lock on the registry.
fn write_lock<'a>() -> RwLockWriteGuard<'a, RegMap> {
    REGISTRY.write().unwrap()
}

/// Gets a value from the regsitry.
pub fn get(name: &RegKey) -> Option<RegVal> {
    trace!("get: key {}", name);
    let ref reg = *read_lock();
    // cloned() is a method on Option<T> where T: Clone
    reg.get(name).cloned()
}

/// Gets a value from the registry, or a default
/// if the value is not found.
pub fn get_or_default(name: &RegKey, value: RegVal) -> RegVal {
    match get(name) {
        Some(value) => value,
        None => value
    }
}

/// Set a value in the registry.
///
/// If the key already exists, returns the old value.
/// Returns `None` for new keys.
pub fn set(name: RegKey, value: RegVal) -> Option<RegVal> {
    trace!("set: {} = {:?}", &name, &value);
    let ref mut reg = *write_lock();
    reg.insert(name, value)
}
