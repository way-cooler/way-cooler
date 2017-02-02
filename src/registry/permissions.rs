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

use std::borrow::Cow;
use std::collections::hash_map::HashMap;
use super::category::Category;
use super::REGISTRY2;

/// The mapping of category to the permissions the client has for that category.
pub type AccessMapping<'category> = HashMap<Cow<'category, str>, Permissions>;
/// The result of trying to use a `Client` to access a `Category`.
pub type ClientResult<'category, T> = Result<T, ClientErr<'category>>;

/// The different ways the `Client` can fail trying to access a `Category`.
#[derive(Clone, Debug)]
pub enum ClientErr<'category> {
    /// A Category does not exist (from the `Client`s perspective)
    DoesNotExist(Cow<'category, str>),
    /// The `Client` has insufficient permissions to do that operation on
    /// the provided category.
    InsufficientPermissions
}

/// The different ways a client can access the data in a `Category`.
///
/// If a permission for a particular `Category` is omitted, the client by
/// definition cannot access the `Category` from its `Client`.
#[derive(Clone, Copy, Debug)]
pub enum Permissions {
    /// The client can read all data associated with a `Category`.
    Read,
    /// The client can read and write to all data associated with a `Category`.
    Write
}

/// The way a client program accesses the categories in the registry.
///
/// Has a mapping of known `Category`s and its associated permissions.
#[derive(Clone, Debug)]
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

    /// Returns read access to the category.
    pub fn read(&self, category: Cow<'category, str>) -> ClientResult<Category> {
        if !self.access.contains_key(&category) {
            return Err(ClientErr::DoesNotExist(category))
        }
        // If it is contained in our mapping, we automatically have sufficient
        // permissions to read it.
        let reg = REGISTRY2.read().expect("Unable to read from registry!");
        reg.category(category.clone()).or(Err(ClientErr::DoesNotExist(category)))
            .map(|c| c.clone())
    }
}
