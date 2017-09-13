//! Utility methods and constructors for Lua objects

use std::fmt::Display;
use std::convert::From;
use rlua::{self, Lua, Table, MetaMethod, AnyUserData, UserData,
           UserDataMethods};

/// All Lua objects can be cast to this.
pub struct Object<'lua> {
    table: Table<'lua>
}


// TODO Move this to TryFrom and check that data exists and is
// the right type.
// Can't just yet cause TryFrom is still nightly...

impl <'lua> From<Table<'lua>> for Object<'lua> {
    fn from(table: Table<'lua>) -> Self {
        Object { table }
    }
}

impl <'lua> Object<'lua> {
    pub fn signals(&self) -> rlua::Table {
        self.table.get::<_, Table>("signals")
            .expect("Object table did not have signals defined!")
    }
}


pub fn add_meta_methods<T: UserData + Display>(methods: &mut UserDataMethods<T>) {
    methods.add_meta_method(MetaMethod::ToString, |lua, obj: &T, _: ()| {
        Ok(lua.create_string(&format!("{}", obj)))
    });

    methods.add_method_mut("connect_signal", connect_signal_simple);

    // TODO Add {connect,disconnect,emit}_signal methods
}


fn connect_signal_simple<T: UserData>(lua: &Lua, this: &mut T,
                                      (name, function): (String, rlua::Function))
                                      -> rlua::Result<()> {
    panic!()
    // get this.signals (e.g SIGNALS ON THE OBJECT INSTANCE)
    // then associate the name with the function
    // e.g this.signals.update(name, function)
}
