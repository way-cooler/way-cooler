//! A category that contains data about its collection.
//! These are the main stores of the data used by Way Cooler and its clients.

use std::ops::{Deref, DerefMut};
use std::collections::hash_map::HashMap;

use rustc_serialize::json::{Json};

/// The main data mapping between a key and some Json.
pub type DataMap = HashMap<String, Json>;

/// A category that has a canonical name, and some data.
///
/// The `Category` can be used exactly like a hash map.
#[derive(Clone, Debug)]
pub struct Category {
    name: String,
    data: HashMap<String, Json>
}


impl Category {
    /// Makes a new category that has some name.
    /// Data mapping is initially empty.
    pub fn new(name: String) -> Self {
        Category {
            name: name,
            data: HashMap::new()
        }
    }

    /// Gets the name of the Category.
    #[allow(dead_code)]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

impl Deref for Category {
    type Target = DataMap;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for Category {
    fn deref_mut(&mut self) -> &mut DataMap {
        &mut self.data
    }
}
