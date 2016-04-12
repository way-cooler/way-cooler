//! way-cooler registry.

use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use rustc_serialize::{Encodable, Decodable};
use rustc_serialize::json::{Json, ToJson};

mod types;
//use self::types::*;

pub use self::types::{RegistryAccess, RegistryValue};

type RegMap = HashMap<String, RegistryValue>;

lazy_static! {
    /// Registry variable for the registry
    static ref REGISTRY: RwLock<RegMap> =
        RwLock::new(HashMap::new());
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
pub fn get_json(name: &String) -> Option<Json> {
    trace!("get_json: {}", name);
    let ref reg = *read_lock();
    if let Some(ref val) = reg.get(name) {
        Some(val.get_json())
    }
    else {
        None
    }
}
