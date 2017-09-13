use std::fmt::{self, Display, Formatter};
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
        let obj: Object<'lua> = self.0.into();
        obj.to_lua(lua)
    }
}

impl <'lua> Objectable<'lua, Button<'lua>> for Button<'lua> {
    fn cast(obj: Object<'lua>) -> rlua::Result<Button> {
        let data = obj.table.get::<_, AnyUserData>("data")?;
        if data.is::<ButtonState>() {
            Ok(Button(obj.table))
        } else {
            use rlua::Error;
            Err(Error::RuntimeError("Could not cast object to button".into()))
        }
    }
}

impl Display for ButtonState {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Button: {:p}", self)
    }
}

impl UserData for ButtonState {
    fn add_methods(methods: &mut UserDataMethods<Self>) {/*TODO Does this need anything?*/}
}

/// Makes a new button stored in a table beside its signals
pub fn new(lua: &Lua, num: u32) -> rlua::Result<Button> {
    let button = ButtonState { num };
    let object = Object::to_object(lua, button)?;
    object.table.get_metatable().unwrap().set("num", 1)?;
    Button::cast(object)
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

