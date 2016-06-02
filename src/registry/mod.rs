//! way-cooler registry.

use std::ops::Deref;
use std::cmp::Eq;
use std::fmt::Display;
use std::hash::Hash;
use std::borrow::Borrow;
use std::collections::hash_map::{HashMap, Entry};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use rustc_serialize::Decodable;
use rustc_serialize::json::{Json, ToJson, Decoder, DecoderError};

mod types;
mod commands;
pub use self::types::*; // Export constants too

#[cfg(test)]
mod tests;

pub type RegMap = HashMap<String, RegistryField>;

lazy_static! {
    /// Registry variable for the registry
    static ref REGISTRY: RwLock<RegMap> = RwLock::new(HashMap::new());
}

/// Error types that can happen
#[derive(Debug, PartialEq)]
pub enum RegistryError {
    /// The registry key was not found
    KeyNotFound,
    /// The registry key was of the wrong type
    WrongKeyType,
    /// Attempting to set a readonly value
    InvalidOperation,
}

/// Result type of gets/sets to the registry
pub type RegistryResult<T> = Result<T, RegistryError>;

/// Initialize the registry (register default commands)
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

/// Gets a RegistryField enum with the specified name
pub fn get_field(name: &str) -> Option<RegistryField> {
    // clone() will either clone an Arc or an Arc+AccessFlags
    let result = read_lock().get(name).map(|v| v.clone());
    trace!("get: {} => {:?}", name, &result);
    return result;
}

/// Attempts to get a command from the registry.
pub fn get_command(name: &str) -> RegistryResult<CommandFn> {
    get_field(name).ok_or(RegistryError::KeyNotFound)
        .and_then(|val| match val {
        RegistryField::Command(com) => Ok(com),
        _ => Err(RegistryError::WrongKeyType)
    })
}

/// Gets a data type from the registry, returning a reference to the property
/// method if the field is a property.
pub fn get_data(name: &str) -> RegistryResult<RegistryGetData> {
    get_field(name).ok_or(RegistryError::KeyNotFound)
        .and_then(|val| match val {
            RegistryField::Object { flags, data } =>
                Ok(RegistryGetData::Object(flags, data)),
            RegistryField::Property { get: maybe_get, .. } =>
                match maybe_get {
                    Some(get) => Ok(RegistryGetData::Property(get)),
                    None => Err(RegistryError::InvalidOperation)
                },
            _ => Err(RegistryError::WrongKeyType)
        })
}

/// Get a Rust structure from the registry
#[allow(dead_code)]
pub fn get_struct<T>(name: &str)
                     -> RegistryResult<(AccessFlags, Result<T, DecoderError>)>
where T: Decodable {
    get_json(name).map(|(flags, json)|
        (flags, T::decode(&mut Decoder::new(json.deref().clone()))))
}

/// Get Json data from the registry, evaluating if a property was found.
pub fn get_json(name: &str) -> RegistryResult<(AccessFlags, Arc<Json>)> {
    get_data(name).map(|val| val.resolve())
}

/// Set a value to the given registry field.
pub fn set_field(key: String, value: RegistryField)
                 -> RegistryResult<Option<RegistryField>> {
    let mut reg = write_lock();
    match reg.entry(key) {
        Entry::Occupied(mut entry) => {
            let first_type = entry.get().get_type();
            if first_type.can_set_from(value.get_type()) {
                Ok(Some(entry.insert(value)))
            }
            else {
                Err(RegistryError::WrongKeyType)
            }
        },
        Entry::Vacant(vacancy) => {
            vacancy.insert(value);
            Ok(None)
        }
    }
}

/// Set a command to the given Command
#[allow(dead_code)]
pub fn set_command(key: String, command: CommandFn)
                   -> RegistryResult<Option<CommandFn>> {
    set_field(key, RegistryField::Command(command))
        .map(|maybe_field|
              maybe_field.and_then(|field|
                                    field.get_command()))
}

/// Set a value to the given JSON value.
#[allow(dead_code)]
pub fn set_json(key: String, flags: AccessFlags, json: Json)
                -> RegistryResult<Option<(AccessFlags, Arc<Json>)>> {
    let mut reg = write_lock();
    match reg.entry(key) {
        Entry::Vacant(vacancy) => {
            vacancy.insert(RegistryField::Object {
                flags: flags, data: Arc::new(json)
            });
            return Ok(None);
        },
        Entry::Occupied(mut entry) => {
            let first_type = entry.get().get_type();
            if first_type == FieldType::Object {
                return Ok(entry.insert(RegistryField::Object {
                    flags: flags, data: Arc::new(json)
                }).as_object());
            }
            else if first_type == FieldType::Property {
                match entry.get().clone().as_property_set() {
                    Some(func) => {
                        func(json);
                        return Ok(None);
                    }
                    None => return Err(RegistryError::InvalidOperation)
                }
            }
            else {
                return Err(RegistryError::WrongKeyType);
            }
        }
    }
    return Ok(None);
}

/// Set an object/property in the registry to a value using a ToJson.
#[allow(dead_code)]
pub fn set_struct<T: ToJson>(key: String, flags: AccessFlags, value: T)
                             -> RegistryResult<Option<(AccessFlags, Arc<Json>)>> {
    set_json(key, flags, value.to_json())
}

/// Sets a value for a given Json value, optionally returning the associated property.
#[allow(dead_code)]
pub fn set_with_property(key: String, flags: AccessFlags, json: Json)
                         -> RegistryResult<Option<(Json, SetFn)>> {
    let mut reg = write_lock();
    match reg.entry(key) {
        Entry::Vacant(vacancy) => {
            vacancy.insert(RegistryField::Object {
                flags: flags, data: Arc::new(json)
            });
            Ok(None)
        },
        Entry::Occupied(mut entry) => {
            let first_type = entry.get().get_type();
            if first_type == FieldType::Object {
                entry.insert(RegistryField::Object {
                    flags: flags, data: Arc::new(json)
                });
                Ok(None)
            }
            else if first_type == FieldType::Property {
                Ok(Some((json, entry.get().clone().as_property_set()
                   .expect("set_json_property: checked value of field type"))))
            }
            else {
                Err(RegistryError::WrongKeyType)
            }
        }
    }
}

/// Binds properties to a field of the registry
#[allow(dead_code)]
pub fn set_property_field(key: String, get_fn: Option<GetFn>, set_fn: Option<SetFn>)
                          -> RegistryResult<Option<RegistryField>> {
    set_field(key, RegistryField::Property { get: get_fn, set: set_fn })
}

/// Whether this map contains a key of any type
#[allow(dead_code)]
pub fn contains_key<K>(key: &K) -> bool
where String: Borrow<K>, K: Hash + Eq + Display {
    trace!("contains_key: {}", *key);
    let read_reg = read_lock();
    read_reg.contains_key(key)
}

/// Whether the registry contains a key of some type:
/// None: no key
/// true: same type key
/// false: different type key
#[allow(dead_code)]
pub fn contains_key_of<K>(key: &K, field_type: FieldType) -> Option<bool>
where String: Borrow<K>, K: Hash + Eq + Display {
    let read_reg = read_lock();
    read_reg.get(key).map(|field| field.get_type() == field_type)
}
