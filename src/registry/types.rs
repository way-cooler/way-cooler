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

/// Function which will yield an object
pub type GetFn = Arc<Fn() -> Json + Send + Sync>;

/// Function which will set an object
pub type SetFn = Arc<Fn(Json) + Send + Sync>;

/// Enum of types of registry fields
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum FieldType { Object, Property, Command }

/// Data which can be stored in the registry
#[derive(Clone)]
pub enum RegistryField {
    /// An object with permission flags
    Object {
        /// Permission flags for Lua get/setting the value
        flags: AccessFlags,
        /// Data associated with this value
        data: Arc<Json>
    },
    /// A registry value whose get and set maps to other Rust code
    Property {
        /// Method called to set a property
        get: GetFn,
        /// Method called to set a property
        set: SetFn
    },
    /// A command
    Command(CommandFn)
}

/// Result of what can be accessed from a registry value.
#[derive(Clone)]
pub enum RegistryGetData {
    /// An object in the registry
    Object(AccessFlags, Arc<Json>),
    /// Get field of a property in the registry
    Property(GetFn)
}

impl Debug for RegistryField {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            &RegistryField::Object { ref flags, ref data } =>
                f.debug_struct("RegistryField::Object")
                .field("flags", flags as &Debug)
                .field("data", data as &Debug).finish(),
            &RegistryField::Property { .. } =>
                write!(f, "RegistryField::Property(...)"),
            &RegistryField::Command(_) =>
                write!(f, "RegistryField::Command(...)")
        }
    }
}

impl Debug for RegistryGetData {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            &RegistryGetData::Object { ref flags, ref data } =>
                f.debug_struct("RegistryGetData::Object")
                .field("flags", flags as &Debug)
                .field("data", data as &Debug).finish(),
            &RegistryField::Property { .. } =>
                write!(f, "RegistryGetData::Property(...)")
        }
    }
}

impl FieldType {
    pub fn can_set_from(self, other: FieldType) -> bool {
        match self {
            FieldType::Command => other == FieldType::Command,
            FieldType::Property =>
                other == FieldType::Object ||
                other == FieldType::Property,
            FieldType::Object => other == FieldType::Object
        }
    }
}

impl RegistryField {
    /// Creates a new registry object with the specified flags
    /// and Rust data to be converted.
    #[allow(dead_code)]
    pub fn new_value<T: ToJson>(flags: AccessFlags, data: T) -> RegistryField {
        RegistryField::Object {
            flags: flags,
            data: Arc::new(data.to_json())
        }
    }

    /// Creates a new RegistryFile with the specified Lua data.
    pub fn new_json(flags: AccessFlags, data: Json) -> RegistryField {
        RegistryField::Object {
            flags: flags, data: Arc::new(data)
        }
    }

    /// Creates a new RegistryCommand with the specified callback.
    pub fn new_command(com: CommandFn) -> RegistryField {
        RegistryField::Command(com)
    }

    /// Creates a new registry property with the specified getter and setter
    pub fn new_property(get: GetFn, set: SetFn) -> RegistryField {
        RegistryField::Property {
            get: get, set: set
        }
    }

    /// What access the module has to it
    #[allow(dead_code)]
    pub fn get_flags(&self) -> Option<AccessFlags> {
        match self {
            &RegistryField::Object { ref flags, .. } => Some(flags.clone()),
            &RegistryField::Property { .. } => Some(AccessFlags::all()),
            _ => None
        }
    }

    /// Attempts to access the RegistryField as a command
    #[allow(dead_code)]
    pub fn get_command(self) -> Option<CommandFn> {
        match self {
            RegistryField::Command(com) => Some(com),
            _ => None
        }
    }

    /// Attempts to access the RegistryField as a file
    pub fn get_data(&self) -> Option<RegistryGetData> {
        match *self {
            RegistryField::Object { ref flags, ref data } =>
                Some(RegistryGetData::Object(flags,clone(), data.clone())),
            RegistryField::Property { ref get, .. } =>
                Some(RegistryGetData::Property(get.clone())),
            _ => None
        }
    }

    /// Gets the type of this registry field
    pub fn get_type(&self) -> FieldType {
        match self {
            &RegistryField::Object { .. }   => FieldType::Object,
            &RegistryField::Property { .. } => FieldType::Property,
            &RegistryField::Command(_)      => FieldType::Command
        }
    }
}

impl RegistryGetData {
    pub fn resolve(self) -> (AccessFlags, Arc<Json>) {
        match self {
            RegistryGetData::Object(flags, data) => (flags, data),
            RegistryGetData::Property(get) =>
                (AccessFlags::all(), Arc::new(get()))
        }
    }
}
