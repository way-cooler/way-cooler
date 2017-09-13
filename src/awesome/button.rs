use std::fmt::{self, Display, Formatter};
use rlua::{self, Table, Lua, UserData, UserDataMethods};
use super::{class, object};

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
    meta.set("__index", lua.create_function(object::default_index))?;
    meta.set("signals", lua.create_table())?;
    // yep the value I have doesn't matter
    // this won't happen for realz, just for testing.
    meta.set("num", 1).unwrap();
    meta.set("__tostring", lua.create_function(|_, button_table: Table| {
        Ok(format!("{}", button_table.get::<_, Button>("data")?))
    }))?;
    button_table.set_metatable(Some(meta));
    Ok(button_table)
}


pub fn init(lua: &Lua) -> rlua::Result<()> {
    unimplemented!()
}


mod test {
    #[test]
    fn basic_test() {
        let lua = Lua::new();
        lua.globals().set("button0", new(&lua, 0).unwrap());
        lua.globals().set("button1", new(&lua, 1).unwrap());
        lua.eval(r#"
                 print(button0)
                 print(button0.num)
                 print(button1.num)
                 button0.connect_signal("test", function(button) button.num = 3 end)
                 button0.emit_signal("test")
                 print(button1.num)
                 print(button0.num)
"#,
        None).unwrap()
    }
}

