//! way-cooler registry.

use std::collections::HashMap;
use std::sync::RwLock;

lazy_static! {
    /// Registry variable for the registry
    static ref REGISTRY: RwLock<HashMap<String, RegistryValue>> =
        RwLock::new(HashMap::new());
}

#[derive(Debug, Clone)]
pub enum RegistryValue {
    Integer(i32),
    Float(f32),
    Boolean(bool),
    String(String)
}

pub fn get<'a>(name: &String) -> Option<RegistryValue> {
    let mut val: Option<RegistryValue> = None;
    {
        let reg = REGISTRY.read().unwrap();
        val = reg.get(name).cloned();
    }
    val
}
