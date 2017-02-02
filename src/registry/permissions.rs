//! The module that controls access to the registry.
//!
//! Permissions for clients live here, which can either be `Read` or `Write`
//! (which includes ability to read).
//!
//! A `Permission` is associated with a canonical category name, with the
//! `Client` controlling all permissions for known categories.
//!
//! Note that it's possible for the `Client` to not know about any given
//! category, effectively making those permissions `None` (because it doesn't
//! even know that it exists).
//!
//! This is the only way to access the registry, allowing the underlying
//! implementation to be simple and allow later optimizations.

use std::collections::hash_map::HashMap;
use super::category::Category;

/// The mapping of category to the permissions the client has for that category.
pub type AccessMapping<'category> = HashMap<Category<'category>, Permissions>;

/// The different ways a client can access the data in a `Category`.
///
/// If a permission for a particular `Category` is omitted, the client by
/// definition cannot access the `Category` from its `Client`.
pub enum Permissions {
    /// The client can read all data associated with a `Category`.
    Read,
    /// The client can read and write to all data associated with a `Category`.
    Write
}

/// The way a client program accesses the categories in the registry.
///
/// Has a mapping of known `Category`s and its associated permissions.
pub struct Client<'category> {
    access: AccessMapping<'category>
}

impl<'category> Client<'category> {
    /// Makes a new client, with the given permissions.
    pub fn new(access: AccessMapping<'category>) -> Self {
        Client {
            access: access
        }
    }
}
