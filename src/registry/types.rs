//! Types used in the registry.

use std::sync::Arc;

use rustc_serialize::json::{Json, ToJson};

bitflags! {
    /// Access permissions for items in the registry
    pub flags AccessFlags: u8 {
        const LUA_PRIVATE = 0b00000000,
        const LUA_ACCESS  = 0b00000001,
        const LUA_WRITE  = 0b00000010,
    }
}

/// Values stored in the registry
#[derive(Debug)]
pub struct RegistryValue {
    flags: AccessFlags,
    object: Arc<Json>
}

impl RegistryValue {
    /// Creates a new RegistryValue
    pub fn new<T>(flags: AccessFlags, data: T) -> RegistryValue
        where T: ToJson  {
        RegistryValue {
            flags: flags,
            object: Arc::new(data.to_json())
        }
    }

    /// What access the module has to it
    pub fn flags(&self) -> AccessFlags {
        self.flags
    }

    /// Gets the json of a registry value
    pub fn get_json(&self) -> Arc<Json> {
        self.object.clone()
    }

    pub fn set_json(&mut self, json: Json) {
        self.object = Arc::new(json);
    }
}
