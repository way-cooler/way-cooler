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

use std::borrow::Cow;
use std::collections::hash_map::{Entry, HashMap};

use super::category::Category;

/// The result of doing an operation on the registry.
pub type RegistryResult<'category, T> = Result<T, RegistryErr<'category>>;

/// Ways accessing of accessing the registry incorrectly
pub enum RegistryErr<'category> {
    /// The category already exists, you cannot overwrite it.
    CategoryExists(Cow<'category, str>),
    /// The category does not exist, it needs to be created.
    CategoryDoesNotExist(Cow<'category, str>)
}

/// The main store for the registry values. It tracks category names with
/// their associated `Category`s which holds the actual data.
///
/// Permissions are NOT tracked from here, that is done with `registry::Access`.
///
/// All public access of the registry should go through an `registry::Access`
/// object, to ensure that permissions are upheld.
pub struct Registry<'category> {
    map: HashMap<Cow<'category, str>, Category<'category>>
}

impl<'category> Registry<'category> {
    /// Looks up a category by its canonical name immutably.
    pub fn category(&self, category: Cow<'category, str>)
                    -> RegistryResult<&Category<'category>> {
        self.map.get(&category)
            .ok_or_else(|| RegistryErr::CategoryDoesNotExist(category))
    }

    /// Looks up a category by its canonical name mutably.
    pub fn category_mut(&mut self, category: Cow<'category, str>)
                        -> RegistryResult<&mut Category<'category>> {
        self.map.get_mut(&category)
            .ok_or_else(|| RegistryErr::CategoryDoesNotExist(category))
    }

    /// Adds a new category to the registry. Fails if it already exists.
    pub fn add_category(&mut self, category: Cow<'category, str>)
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
