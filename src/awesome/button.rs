use std::fmt::{self, Display, Formatter};
use std::default::Default;
use rlua::{self, Table, Lua, UserData, AnyUserData, UserDataMethods, ToLua,
           Value};
use super::object::{self, Object, Objectable};

#[derive(Clone, Debug)]
pub struct ButtonState {
    num: u32
}

#[derive(Clone, Debug)]
pub struct Button<'lua>(Table<'lua>);

impl <'lua> ToLua<'lua> for Button<'lua> {
    fn to_lua(self, lua: &'lua Lua) -> rlua::Result<Value<'lua>> {
        self.0.to_lua(lua)
    }
}

impl <'lua> Objectable<'lua, Button<'lua>, ButtonState> for Button<'lua> {
    fn _wrap(table: Table<'lua>) -> Button {
        Button(table)
    }

    fn get_table(self) -> Table<'lua> {
        self.0
    }
}

impl Display for ButtonState {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Button: {:p}", self)
    }
}

impl Default for ButtonState {
    fn default() -> Self {
        ButtonState { num: 0 }
    }
}

impl UserData for ButtonState {
    fn add_methods(methods: &mut UserDataMethods<Self>) {/*TODO Does this need anything?*/}
}

/// Makes a new button stored in a table beside its signals
pub fn new(lua: &Lua, num: u32) -> rlua::Result<Object> {
    let object = Object::new::<ButtonState>(lua)?;
    let button = Button::cast(object)?;
    button.0.get_metatable().unwrap().set("num", 1)?;
    Ok(Object::to_object(button))
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

