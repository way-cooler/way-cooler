//! way-cooler registry.

use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use rustc_serialize::{Encodable, Decodable};
use rustc_serialize::json;
use rustc_serialize::json::{Json, ToJson, EncoderError};

mod types;

pub use self::types::{RegistryAccess, RegistryValue};

type RegMap = HashMap<String, RegistryValue>;

lazy_static! {
    /// Registry variable for the registry
    static ref REGISTRY: RwLock<RegMap> =
        RwLock::new(HashMap::new());
}

/// Error types that can happen
#[derive(Debug, PartialEq, Eq)]
pub enum RegistryError {
    InvalidJson,
    KeyNotFound
}

/// Acquires a read lock on the registry.
fn read_lock<'a>() -> RwLockReadGuard<'a, RegMap> {
    REGISTRY.read().unwrap()
}

/// Acquires a write lock on the registry.
fn write_lock<'a>() -> RwLockWriteGuard<'a, RegMap> {
    REGISTRY.write().unwrap()
}

/// Gets a Json object from a registry key
pub fn get_json(name: &String) -> Option<Arc<Json>> {
    trace!("get_json: {}", name);
    let ref reg = *read_lock();
    if let Some(val) = reg.get(name) {
        Some(val.get_json())
    }
    else {
        None
    }
}

use std::ops::Deref;

pub fn get<T: Decodable>(name: &String) -> Result<T, RegistryError> {
    let maybe_json = get_json(name);
    if let Some(json_arc) = maybe_json {
        let mut decoder = json::Decoder::new(json_arc.deref().to_json());
        match T::decode(&mut decoder) {
            Ok(val) => Ok(val),
            Err(e) => Err(RegistryError::InvalidJson)
        }
    }
    else {
        Err(RegistryError::KeyNotFound)
    }
}

pub fn set<T: ToJson>(key: String, val: T) {
    trace!("set: {}", key);
    let ref mut write_reg = *write_lock();
    let regvalue = RegistryValue::new(RegistryAccess::Public, val);
    write_reg.insert(key, regvalue);
}
