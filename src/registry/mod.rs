//! way-cooler registry.

use std::ops::Deref;
use std::cmp::Eq;
use std::fmt::Display;
use std::hash::Hash;
use std::borrow::Borrow;

use hlua::any::AnyLuaValue;
use convert::{ToTable, FromTable, ConverterError};

use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

mod types;
pub use self::types::*; // Export constants too

#[cfg(test)]
mod tests;

pub type RegMap = HashMap<String, RegistryValue>;

lazy_static! {
    /// Registry variable for the registry
    static ref REGISTRY: RwLock<RegMap> =
        RwLock::new(HashMap::new());
}

/// Error types that can happen
#[derive(Debug, PartialEq, Eq)]
pub enum RegistryError {
    /// The value in the registry could not be parsed
    InvalidLua(ConverterError),
    /// The registry key was not found
    KeyNotFound
}

/// Acquires a read lock on the registry.
pub fn read_lock<'a>() -> RwLockReadGuard<'a, RegMap> {
    REGISTRY.read().expect("Unable to read from registry!")
}

/// Acquires a write lock on the registry.
pub fn write_lock<'a>() -> RwLockWriteGuard<'a, RegMap> {
    REGISTRY.write().expect("Unable to write to registry!")
}

/// Gets a Lua object from a registry key
pub fn get_lua<K>(name: &K) -> Option<(AccessFlags, Arc<AnyLuaValue>)>
where String: Borrow<K>, K: Hash + Eq + Display {
    trace!("get_lua: {}", *name);
    let reg = read_lock();
    reg.get(name).map(|val| (val.flags(), val.get_lua()))
}

/// Gets an object from the registry, decoding its internal Lua
/// representation.
#[allow(dead_code)]
pub fn get<K, T>(name: &K) -> Result<(AccessFlags, T), RegistryError>
    where T: FromTable, String: Borrow<K>, K: Hash + Eq + Display {
    if let Some(lua_pair) = get_lua(name) {
        let (access, lua_arc) = lua_pair;
        // Ultimately, values must be cloned out of the registry as well
        match T::from_lua_table(lua_arc.deref().clone()) {
            Ok(val) => Ok((access, val)),
            Err(e) => Err(RegistryError::InvalidLua(e))
        }
    }
    else {
        Err(RegistryError::KeyNotFound)
    }
}

/// Set a key in the registry to a particular value
#[allow(dead_code)]
pub fn set<T: ToTable>(key: String, flags: AccessFlags, val: T) {
    trace!("set: {:?} {}", flags, key);
    let regvalue = RegistryValue::new(flags, val);
    let mut write_reg = write_lock();
    write_reg.insert(key, regvalue);
}

/// Whether this map contains a key
#[allow(dead_code)]
pub fn contains_key<K>(key: &K) -> bool
where String: Borrow<K>, K: Hash + Eq + Display {
    trace!("contains_key: {}", *key);
    let read_reg = read_lock();
    read_reg.contains_key(key)
}
