//! Utility methods and constructors for Lua classes

use rlua::{self, MetaMethod, UserData, UserDataMethods};

pub fn add_meta_methods<T: UserData>(methods: &mut UserDataMethods<T>) {
    //methods.add_meta_method(MetaMethod::Index, |lua, obj, idx: rlua::Value| {
    //    // TODO "valid" and "data"
    //});
}
