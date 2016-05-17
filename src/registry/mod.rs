//! way-cooler registry.

use std::cmp::Eq;
use std::fmt::Display;
use std::hash::Hash;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use rustc_serialize::Decodable;
use rustc_serialize::json::{Json, ToJson, Decoder, DecoderError};

mod types;
mod commands;
pub use self::types::*; // Export constants too

#[cfg(test)]
mod tests;

pub type RegMap = HashMap<String, RegistryValue>;

lazy_static! {
    /// Registry variable for the registry
    static ref REGISTRY: RwLock<RegMap> = RwLock::new(HashMap::new());
}

/// Error types that can happen
#[derive(Debug, PartialEq)]
pub enum RegistryError {
    /// The value in the registry could not be parsed
    InvalidJson(DecoderError),
    /// The registry key was not found
    KeyNotFound,
    /// The registry key was of the wrong type
    WrongKeyType,
}

pub fn init() {
    commands::register_defaults();
}

/// Acquires a read lock on the registry.
pub fn read_lock<'a>() -> RwLockReadGuard<'a, RegMap> {
    REGISTRY.read().expect("Unable to read from registry!")
}

/// Acquires a write lock on the registry.
pub fn write_lock<'a>() -> RwLockWriteGuard<'a, RegMap> {
    REGISTRY.write().expect("Unable to write to registry!")
}


/// Gets a RegistryValue enum with the specified name
pub fn get_value<K>(name: &K) -> Option<RegistryValue>
    where String: Borrow<K>, K: Hash + Eq + Display {
    trace!("> get_value: {}", name);
    // clone() will either clone an Arc or an Arc+AccessFlags
    read_lock().get(name).map(|v| v.clone())
}

/// Attempts to get a command from the registry.
pub fn get_command<K>(name: &K) -> Result<CommandFn, RegistryError>
    where String: Borrow<K>, K: Hash + Eq + Display {
    trace!("< get_command: {}", name);
    get_value(name).ok_or(RegistryError::KeyNotFound)
        .and_then(|val| match val {
        RegistryValue::Command(com) => Ok(com),
        _ => Err(RegistryError::WrongKeyType)
    })
}

/// Gets a Json object from a registry key
pub fn get_json<K>(name: &K) -> Result<(AccessFlags, Arc<Json>), RegistryError>
where String: Borrow<K>, K: Hash + Eq + Display {
    trace!("< get_json: {}", name);
    get_value(name).ok_or(RegistryError::KeyNotFound)
        .and_then(|val| match val {
            RegistryValue::Object { flags, data } =>
                Ok((flags, data)),
            _ => Err(RegistryError::WrongKeyType)
        })
}

/// Gets an object from the registry, decoding its internal Json
/// representation.
#[allow(dead_code)]
pub fn get_data<K, T>(name: &K) -> Result<(AccessFlags, T), RegistryError>
    where T: Decodable, String: Borrow<K>, K: Hash + Eq + Display {
    trace!("< < get_data: {}", name);
    get_json(name).and_then(|(flags, obj)| {
        match T::decode(&mut Decoder::new((*obj).clone())) {
            Ok(val) => Ok((flags, val)),
            Err(e) => Err(RegistryError::InvalidJson(e))
        }
    })
}

/// Set a value to the given RegistryValue
#[allow(dead_code)]
pub fn set(key: String, value: RegistryValue) {
    trace!("> set: {} to {:?}", key, &value);
    write_lock().insert(key, value);
}

/// Set a command to the given Command
#[allow(dead_code)]
pub fn set_command(key: String, command: CommandFn) {
    trace!("< set_command: {}", key);
    set(key, RegistryValue::Command(command))
}

/// Set a value to the given JSON value.
pub fn set_json(key: String, flags: AccessFlags, json: Json) {
    trace!("< set_json: {}", key);
    set(key, RegistryValue::new_json(flags, json))
}

/// Sets a value to the given data, to be encoded into JSON
#[allow(dead_code)]
pub fn set_data<T: ToJson>(key: String, access: AccessFlags, val: T) {
    trace!("< < set_data: {}", key);
    set_json(key, access, val.to_json())
}

/// Whether this map contains a key of any type
#[allow(dead_code)]
pub fn contains_key<K>(key: &K) -> bool
where String: Borrow<K>, K: Hash + Eq + Display {
    trace!("contains_key: {}", *key);
    let read_reg = read_lock();
    read_reg.contains_key(key)
}
