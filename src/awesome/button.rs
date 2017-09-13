use std::fmt::{self, Display, Formatter};
use rlua::{self, Table, Lua, UserData, UserDataMethods};
use super::object;

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
    fn add_methods(methods: &mut UserDataMethods<Self>) {/*TODO Does this need anything?*/}
}

/// Makes a new button stored in a table beside its signals
pub fn new(lua: &Lua, num: u32) -> rlua::Result<Table> {
    let button = Button { num };
    let table = object::add_meta_methods(lua, button)?;
    table.get_metatable().unwrap().set("num", 1)?;
    Ok(table)
}


// TODODDDDDOOOOO
// FIXME
// work on making sure the filled in methods on "object.rs" is correct.
// then implement enough "class.rs" so that I can impl this function:


pub fn init(lua: &Lua) -> rlua::Result<()> {
    unimplemented!()
}


mod test {
    #[test]
    fn basic_test() {
        use rlua::Lua;
        use super::new;
        let lua = Lua::new();
        lua.globals().set("button0", new(&lua, 0).unwrap());
        lua.globals().set("button1", new(&lua, 1).unwrap());
        lua.eval(r#"
                 assert(button0.num == 1)
                 assert(button1.num == 1)
                 button0.connect_signal("test", function(button) button.num = 3 end)
                 button0.emit_signal("test")
                 assert(button1.num == 1)
                 assert(button0.num == 3)
"#,
        None).unwrap()
    }
}

