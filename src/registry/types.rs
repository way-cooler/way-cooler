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
        get: Option<GetFn>,
        /// Method called to set a property
        set: Option<SetFn>
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
            &RegistryField::Property { ref get, ref set } => {
                let new_get = match get { &Some(_) => Some(true), &None => None };
                let new_set = match set { &Some(_) => Some(true), &None => None };
                f.debug_struct("RegistryField::Property")
                    .field("get", &new_get)
                    .field("set", &new_set)
                    .finish()
            }
            &RegistryField::Command(_) =>
                write!(f, "RegistryField::Command(...)")
        }
    }
}

impl Debug for RegistryGetData {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            &RegistryGetData::Object(ref flags, ref data) =>
                f.debug_struct("RegistryGetData::Object")
                .field("flags", flags as &Debug)
                .field("data", data as &Debug).finish(),
            &RegistryGetData::Property(_) =>
                write!(f, "RegistryGetData::Property")
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
    /// What access the module has to it
    #[allow(dead_code)]
    pub fn get_flags(&self) -> Option<AccessFlags> {
        match self {
            &RegistryField::Object { ref flags, .. } => Some(flags.clone()),
            &RegistryField::Property { ref get, ref set } => {
                let mut flags = AccessFlags::empty();
                if get.is_some() { flags.insert(LUA_READ); }
                if set.is_some() { flags.insert(LUA_WRITE); }
                Some(flags)
            },
            _ => None
        }
    }

    /// Attempts to access the RegistryField as a command
    pub fn get_command(&self) -> Option<CommandFn> {
        match *self {
            RegistryField::Command(ref com) => Some(com.clone()),
            _ => None
        }
    }

    /// Attempts to access the RegistryField as a file
    pub fn get_data(&self) -> Option<RegistryGetData> {
        match *self {
            RegistryField::Object { ref flags, ref data } =>
                Some(RegistryGetData::Object(flags.clone(), data.clone())),
            RegistryField::Property { ref get, .. } =>
                get.clone().and_then(|g| Some(RegistryGetData::Property(g))),
            _ => None
        }
    }

    /// Converts this RegistryField to maybe a command
    pub fn as_command(self) -> Option<CommandFn> {
        match self {
            RegistryField::Command(com) => Some(com),
            _ => None
        }
    }

    pub fn as_object(self) -> Option<(AccessFlags, Arc<Json>)> {
        match self {
            RegistryField::Object { flags, data } => Some((flags, data)),
            _ => None
        }
    }

    pub fn as_property(self) -> Option<(Option<GetFn>, Option<SetFn>)> {
        match self {
            RegistryField::Property { get, set } => Some((get, set)),
            _ => None
        }
    }

    pub fn as_property_get(self) -> Option<GetFn> {
        self.as_property().and_then(|(maybe_get, _)| maybe_get)
    }

    pub fn as_property_set(self) -> Option<SetFn> {
        self.as_property().and_then(|(_, maybe_set)| maybe_set)
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

    pub fn get_type(&self) -> FieldType {
        match self {
            &RegistryGetData::Property(_) => FieldType::Property,
            &RegistryGetData::Object(_, _) => FieldType::Object
        }
    }
}
