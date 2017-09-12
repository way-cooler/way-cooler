use std::fmt::{self, Display, Formatter};
use rlua::{self, Table, Lua, UserData, UserDataMethods};
use super::{class, object};

use rustwlc::*;
#[allow(deprecated)]
use rustwlc::xkb::Keysym;

#[derive(Clone, Debug)]
pub struct Button {
    num: u32
}

impl Display for Button {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Button: {:p}", self)
    }
}

impl UserData for Button {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        object::add_meta_methods(methods);
        class::add_meta_methods(methods);
    }
}

/// Makes a new button stored in a table beside its signals
pub fn new(lua: &Lua, num: u32) -> rlua::Result<Table> {
    let button_table = lua.create_table();
    button_table.set("data", Button { num })?;
    let meta = lua.create_table();
    meta.set("__index", lua.create_function(default_index))?;
    meta.set("signals", lua.create_table());
    meta.set("__tostring", lua.create_function(|_, button_table: Table|
                                               Ok(format!("{}", button_table.get::<_, Button>("data").unwrap()))));
    button_table.set_metatable(Some(meta));
    Ok(button_table)
}

fn default_index<'lua>(lua: &'lua Lua, (button_table, index): (Table<'lua>, String))
                 -> rlua::Result<rlua::Value<'lua>> {
    if let Ok(val) = button_table.get::<_, rlua::Value>(index.clone()) {
        return Ok(val)
    }
    // TODO error handling
    let button = button_table.raw_get::<_, Button>("data").unwrap();
    match index.as_str() {
        "connect_signal" => {
            lua.globals().set("__temp", button_table);
            Ok(rlua::Value::Function(lua.create_function(|lua, val: rlua::Value| {
                let button_table = lua.globals().get::<_, Table>("__temp").unwrap();
                lua.globals().set("__temp", rlua::Value::Nil);
                let signals = button_table.get_metatable()
                    .expect("no meta")
                    .get::<_, Table>("signals")
                    .expect("signals was not a table");
                signals.set(signals.len().expect("No length"), val);
                Ok(())
            })))
        },
        "emit_signal" => {
            lua.globals().set("__temp", button_table);
            Ok(rlua::Value::Function(lua.create_function(|lua, val: rlua::Value| {
                let button_table = lua.globals().get::<_, Table>("__temp").unwrap();
                lua.globals().set("__temp", rlua::Value::Nil);
                let signals = button_table.get_metatable().unwrap().get::<_, Table>("signals").unwrap();
                signals.get::<_,rlua::Function>(0).unwrap().call::<_,()>(button_table);
                Ok(())
            })))
        },
        "num" => Ok(rlua::Value::Number(button.num as _)),
        // TODO Error here
        _ => Ok(rlua::Value::Nil)
    }
    // TODO special "valid" property
}

pub fn init(lua: &Lua) -> rlua::Result<()> {
    unimplemented!()
}


mod test {
    use rlua::*;
    use super::*;
    #[test]
    fn basic_test() {
        let lua = Lua::new();
        lua.globals().set("button0", new(&lua, 0).unwrap());
        lua.globals().set("button1", new(&lua, 1).unwrap());
        lua.eval(r#"
                 print(button0)
                 print(button0.num)
                 print(button1.num)
                 print(button0.connect_signal(function(button) button.num = 3 end))
                 print(button0.emit_signal())
                 print(button0.num)
"#,
        None).unwrap()
    }
}

