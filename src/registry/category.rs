//! A category that contains data about its collection.
//! These are the main stores of the data used by Way Cooler and its clients.

use std::ops::{Deref, DerefMut};
use std::borrow::{Borrow, Cow};
use std::collections::hash_map::HashMap;

use rustc_serialize::json::{Json};

/// The main data mapping between a key and some Json.
pub type DataMap = HashMap<String, Json>;

/// A category that has a canonical name, and some data.
///
/// The `Category` can be used exactly like a hash map.
pub struct Category<'category> {
    name: Cow<'category, str>,
    data: HashMap<String, Json>
}


impl<'category> Category<'category> {
    /// Makes a new category that has some name.
    /// Data mapping is initially empty.
    pub fn new(name: Cow<'category, str>) -> Self {
        Category {
            name: name,
            data: HashMap::new()
        }
    }

    /// Gets the name of the Category.
    pub fn name(&self) -> &str {
        self.name.borrow()
    }
}

impl<'category> Deref for Category<'category> {
    type Target = DataMap;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'category> DerefMut for Category<'category> {
    fn deref_mut(&mut self) -> &mut DataMap {
        &mut self.data
    }
}
