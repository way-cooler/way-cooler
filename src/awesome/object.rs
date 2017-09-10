//! Utility methods and constructors for Lua objects

use std::fmt::Display;
use rlua::{self, Lua, MetaMethod, AnyUserData, UserData, UserDataMethods};

pub fn add_meta_methods<T: UserData<'static> + Display>(methods: &mut UserDataMethods<'static, T>) {
    methods.add_meta_method(MetaMethod::ToString, |lua, obj: &T, _: ()| {
        Ok(lua.create_string(&format!("{}", obj)))
    });

    methods.add_method_mut("connect_signal", connect_signal_simple);

    // TODO Add {connect,disconnect,emit}_signal methods
}


fn connect_signal_simple<T: UserData<'static>>(lua: &Lua, this: &mut T,
                                      (name, function): (String, rlua::Function))
                                      -> rlua::Result<()> {
    panic!()
    // get this.signals (e.g SIGNALS ON THE OBJECT INSTANCE)
    // then associate the name with the function
    // e.g this.signals.update(name, function)
}
