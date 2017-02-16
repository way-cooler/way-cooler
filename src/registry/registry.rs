//! A registry of values, available both internally to Way Cooler
//! and its clients.
//!
//! The registry is divided into "categories", which gives the values some
//! semblance of structure (e.g window properties are part of the "windows"
//! category) while also allowing fine grain permissions for clients
//! (e.g most clients can read properties about windows,
//! but cannot modify them)
//!
//! Once a category has been created, you cannot overwrite it. However, you can
//! overwrite the data within the category assuming you have write permissions
//! for that category.
//!
//! Access is controlled by the `registry::Access` struct, which ensures
//! that the user of the registry can actually access the values its
//! trying to access

use std::collections::hash_map::{Entry, HashMap};
use std::sync::{RwLockReadGuard, RwLockWriteGuard, LockResult};
use uuid::Uuid;
use super::category::Category;
use super::client::{Client, ClientErr, ClientResult, Permissions};
use super::REGISTRY;

/// The result of doing an operation on the registry.
pub type RegistryResult<T> = Result<T, RegistryErr>;

/// Ways accessing of accessing the registry incorrectly
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RegistryErr {
    /// The category already exists, you cannot overwrite it.
    CategoryExists(String),
    /// The category does not exist, it needs to be created.
    CategoryDoesNotExist(String)
}

/// The main store for the registry values. It tracks category names with
/// their associated `Category`s which holds the actual data.
///
/// Permissions are NOT tracked from here, that is done with `registry::Access`.
///
/// All public access of the registry should go through an `registry::Access`
/// object, to ensure that permissions are upheld.
pub struct Registry {
    map: HashMap<String, Category>
}

impl Registry {
    /// Makes a new registry, with no categories or data.
    pub fn new() -> Self {
        Registry { map: HashMap::new() }
    }
    /// Looks up a category by its canonical name immutably.
    pub fn category(&self, category: String) -> RegistryResult<&Category> {
        self.map.get(&category)
            .ok_or_else(|| RegistryErr::CategoryDoesNotExist(category))
    }

    /// Looks up a category by its canonical name mutably.
    pub fn category_mut(&mut self, category: String)
                        -> RegistryResult<&mut Category> {
        self.map.get_mut(&category)
            .ok_or_else(|| RegistryErr::CategoryDoesNotExist(category))
    }

    /// Adds a new category to the registry. Fails if it already exists.
    pub fn add_category(&mut self, category: String)
                        -> RegistryResult<()> {
        match self.map.entry(category.clone()) {
            Entry::Occupied(_) =>
                Err(RegistryErr::CategoryExists(category.into())),
            Entry::Vacant(entry) => {
                entry.insert(Category::new(category));
                Ok(())
            }
        }
    }
}

/// A handle for accessing the registry behind a read lock.
/// Holds the lock and a reference to the client who is using the
/// handle to access the registry.
pub struct ReadHandle<'lock> {
    handle: LockResult<RwLockReadGuard<'lock, Registry>>,
    client: &'lock Client
}

impl<'lock> ReadHandle<'lock> {
    /// Makes a new handle to the registry with the given permissions.
    pub fn new(client: &'lock Client) -> Self {
        ReadHandle {
            handle: REGISTRY.read(),
            client: client
        }
    }

    /// Attempts to access the data behind the category.
    pub fn read(&self, category: String) -> ClientResult<&Category> {
        if !self.client.categories().any(|permission| *permission.0 == category) {
            if self.client.id() != Uuid::nil() {
                return Err(ClientErr::DoesNotExist(category))
            }
        }
        // if we have it in our permissions, we automatically can read it.
        let handle = self.handle.as_ref().expect("handle.was poisoned!");
        handle.category(category.clone())
            .or(Err(ClientErr::DoesNotExist(category)))
    }
}

/// A handle for accessing the registry behind a write lock.
/// Holds the lock and a reference to the client who is using the
/// handle to access the registry.
pub struct WriteHandle<'lock> {
    handle: LockResult<RwLockWriteGuard<'lock, Registry>>,
    client: &'lock Client
}

impl<'lock> WriteHandle<'lock> {
    /// Makes a new handle to the registry with the given permissions.
    pub fn new(client: &'lock Client) -> Self {
        WriteHandle {
            handle: REGISTRY.write(),
            client: client
        }
    }

    /// Writes to the data behind a category.
    ///
    /// If the category does not exist, it is automatically created.
    pub fn write(&mut self, category: String) -> ClientResult<&mut Category> {
        let mut categories = self.client.categories();
        if self.client.id() != Uuid::nil() {
            try!(categories.find(|cat| *cat.0 == category)
                .ok_or_else(|| ClientErr::DoesNotExist(category.clone()))
                .and_then(|category| {
                    if *category.1 != Permissions::Write {
                        Err(ClientErr::InsufficientPermissions)
                    } else {
                        Ok(())
                    }
                }));
        }
        let handle = self.handle.as_mut().expect("handle.was poisoned!");
        if !self.client.categories().any(|permission| *permission.0 == category) {
            handle.add_category(category.clone()).ok();
        }
        handle.category_mut(category.clone())
            .or(Err(ClientErr::DoesNotExist(category)))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_basic_category_manipulation() {
        let mut reg = Registry::new();
        reg.add_category("Test".into()).unwrap();
        assert_eq!(reg.add_category("Test".into()), Err(RegistryErr::CategoryExists("Test".into())));
        assert!(reg.category("Test".into()).is_ok());
        assert!(reg.category_mut("Test".into()).is_ok());
        assert_eq!(reg.category("test".into()),
                   Err(RegistryErr::CategoryDoesNotExist("test".into())));
        assert_eq!(reg.category_mut("test".into()),
                   Err(RegistryErr::CategoryDoesNotExist("test".into())));
        assert_eq!(reg.category("whatever".into()),
                   Err(RegistryErr::CategoryDoesNotExist("whatever".into())));
        assert_eq!(reg.category_mut("whatever".into()),
                   Err(RegistryErr::CategoryDoesNotExist("whatever".into())));
    }
}
