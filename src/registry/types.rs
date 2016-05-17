//! Types used in the registry.

use std::sync::Arc;
use std::fmt::{Debug, Formatter};
use std::fmt::Result as FmtResult;

use rustc_serialize::json::{Json, ToJson};

bitflags! {
    /// Access permissions for items in the registry
    pub flags AccessFlags: u8 {
        #[allow(dead_code)]
        /// Default flags
        const LUA_PRIVATE = 0,
        /// Lua thread can read the data
        const LUA_READ    = 1 << 0,
        /// Lua thread can write to the data
        const LUA_WRITE   = 1 << 1,
    }
}

/// Command type for Rust function
pub type CommandFn = Arc<Fn() + Send + Sync>;

/// Data which can be stored in the registry
#[derive(Clone)]
pub enum RegistryValue {
    /// An object with permission flags
    Object {
        /// Permission flags for Lua get/setting the value
        flags: AccessFlags,
        /// Data associated with this value
        data: Arc<Json>
    },
    /// A command
    Command(CommandFn)
}

impl Debug for RegistryValue {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            &RegistryValue::Object { ref flags, ref data } =>
                f.debug_struct("RegistryValue::Object")
                .field("flags", flags as &Debug)
                .field("data", data as &Debug).finish(),
            &RegistryValue::Command(_) =>
                write!(f, "RegistryValue::Command(...)")
        }
    }
}

impl RegistryValue {
    /// Creates a new registry object with the specified flags
    /// and Rust data to be converted.
    #[allow(dead_code)]
    pub fn new_value<T: ToJson>(flags: AccessFlags, data: T) -> RegistryValue {
        RegistryValue::Object {
            flags: flags,
            data: Arc::new(data.to_json())
        }
    }

    /// Creates a new RegistryFile with the specified Lua data.
    pub fn new_json(flags: AccessFlags, data: Json) -> RegistryValue {
        RegistryValue::Object {
            flags: flags, data: Arc::new(data)
        }
    }

    /// Creates a new RegistryCommand with the specified callback.
    pub fn new_command(com: CommandFn) -> RegistryValue {
        RegistryValue::Command(com)
    }

    /// What access the module has to it
    #[allow(dead_code)]
    pub fn get_flags(&self) -> Option<AccessFlags> {
        match self {
            &RegistryValue::Object { ref flags, .. } => Some(flags.clone()),
            _ => None
        }
    }

    /// Attempts to access the RegistryValue as a command
    #[allow(dead_code)]
    pub fn get_command(self) -> Option<CommandFn> {
        match self {
            RegistryValue::Command(com) => Some(com),
            _ => None
        }
    }

    /// Attempts to access the RegistryValue as a file
    pub fn get_data(&self) -> Option<(AccessFlags, Arc<Json>)> {
        match *self {
            RegistryValue::Object { ref flags, ref data } =>
                Some((flags.clone(), data.clone())),
            _ => None
        }
    }
}
