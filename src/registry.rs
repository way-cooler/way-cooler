//! way-cooler registry.

use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use rustc_serialize::{Encodable, Decodable};
use rustc_serialize::json::{Json, ToJson};

pub type RegKey = String;
pub type RegVal = RegistryValue;
pub type RegMap = HashMap<RegKey, RegVal>;

lazy_static! {
    /// Registry variable for the registry
    static ref REGISTRY: RwLock<RegMap> =
        RwLock::new(HashMap::new());
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RegistryAccess {
    Public,
    Lua,
    Private
}

/// Values stored in the registry
pub struct RegistryValue {
    access: RegistryAccess,
    object: Arc<ToJson + Sync + Send>
}

impl RegistryValue {
    /// What access the module has to it
    pub fn access(&self) -> RegistryAccess {
        self.access
    }

    /// Gets the json of a registry value
    pub fn get_json(&self) -> Json {
        self.object.to_json()
    }
}

/// Acquires a read lock on the registry.
fn read_lock<'a>() -> RwLockReadGuard<'a, RegMap> {
    REGISTRY.read().unwrap()
}

/// Acquires a write lock on the registry.
fn write_lock<'a>() -> RwLockWriteGuard<'a, RegMap> {
    REGISTRY.write().unwrap()
}

/// Gets a Json object from a registry key
pub fn get_json(name: &RegKey) -> Option<Json> {
    trace!("get_json: {}", name);
    let ref reg = *read_lock();
    if let Some(ref val) = reg.get(name) {
        Some(val.get_json())
    }
    else {
        None
    }
}

pub struct JsonWrap(Json);

use hlua::{Push, PushGuard, AsMutLua, LuaContext};
use hlua::LuaTable;
use hlua_ffi;

fn push_iter<L, V, I>(mut lua: L, iterator: I) -> PushGuard<L>
    where L: AsMutLua, V: for<'b> Push<&'b mut L>, I: Iterator<Item=V>
{
    // creating empty table
    unsafe { hlua_ffi::lua_newtable(lua.as_mut_lua().0) };

    for (elem, index) in iterator.zip((1 ..)) {
        let size = elem.push_to_lua(&mut lua).forget();

        match size {
            0 => continue,
            1 => {
                let index = index as u32;
                index.push_to_lua(&mut lua).forget();
                unsafe { hlua_ffi::lua_insert(lua.as_mut_lua().0, -2) }
                unsafe { hlua_ffi::lua_settable(lua.as_mut_lua().0, -3) }
            },
            2 => unsafe { hlua_ffi::lua_settable(lua.as_mut_lua().0, -3) },
            _ => unreachable!()
        }
    }

    PushGuard { lua: lua, size: 1 }
}


fn push_rec_iter<L, V, I>(mut lua: L, iterator: I) -> PushGuard<L>
    where L: AsMutLua, V: for<'a> Push<&'a mut L>, I: Iterator<Item=V>
{
    let (nrec, _) = iterator.size_hint();

    // creating empty table with pre-allocated non-array elements
    unsafe { hlua_ffi::lua_createtable(lua.as_mut_lua().0, 0, nrec as i32) };

    for elem in iterator {
        let size = elem.push_to_lua(&mut lua).forget();

        match size {
            0 => continue,
            2 => unsafe { hlua_ffi::lua_settable(lua.as_mut_lua().0, -3) },
            _ => unreachable!()
        }
    }

    PushGuard { lua: lua, size: 1 }
}

fn push_table_field<L, V>(mut lua: L, name: String, val: V)
    where L: AsMutLua, V: for<'a> Push<&'a mut L> {
    name.push_to_lua(&mut lua);
    val.push_to_lua(&mut lua);
    unsafe { hlua_ffi::lua_settable(lua.as_mut_lua().0, -3); }
}

use std::collections::BTreeMap;

impl<L: AsMutLua> Push<L> for JsonWrap {
    fn push_to_lua(self, lua: L) -> PushGuard<L> {
        unsafe {
            match self.0 {
                Json::I64(val) => {
                    (val as i32).push_to_lua(lua)
                },
                Json::U64(val) => {
                    (val as u32).push_to_lua(lua)
                },
                Json::F64(val) => {
                    val.push_to_lua(lua)
                },
                Json::String(val) => {
                    val.push_to_lua(lua)
                },
                Json::Null => {
                    ().push_to_lua(lua)
                },
                Json::Boolean(val) => {
                    val.push_to_lua(lua)
                },
                Json::Array(vals) => {
                    push_iter(lua, vals.into_iter()
                              .map(|j| JsonWrap(j)))
                },
                Json::Object(map_l) => {
                    let map: BTreeMap<String, Json> = map_l;

                    let len = map.len();

                    unsafe {
                        hlua_ffi::lua_newtable(lua.as_mut_lua().0);
                    }

                    for (key, value) in map.into_iter() {
                        push_table_field(&mut lua, key, JsonWrap(value));
                    }
                    PushGuard { lua: lua, size: 1 }
                }
            }
        }
    }
}
