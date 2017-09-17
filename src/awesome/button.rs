use std::fmt::{self, Display, Formatter};
use std::default::Default;
use rlua::{self, Table, Lua, UserData, AnyUserData, UserDataMethods, ToLua,
           Value};
use super::object::{self, Object, Objectable};
use super::property::Property;
use super::class::{self, Class};

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
pub fn allocator(lua: &Lua) -> rlua::Result<Object> {
    let meta = lua.create_table();
    // TODO remove
    meta.set("num", 1)?;
    Ok(Button::new(lua)?
       .add_to_meta(meta)?
       .build())
}

pub fn new<'lua>(lua: &'lua Lua, _table: Table<'lua>)
                 -> rlua::Result<Object<'lua>> {
    allocator(lua)
}


pub fn init(lua: &Lua) -> rlua::Result<Class> {
    // TODO Add properties to class
    let class = Class::new(lua, Some(allocator), None, None)?;
    class
        .method(&lua, "__call".into(), lua.create_function(new))?
        .property(Property::new("button".into(),
                                Some(lua.create_function(set_button)),
                                Some(lua.create_function(get_button)),
                                Some(lua.create_function(set_button))))?
        .property(Property::new("modifiers".into(),
                                Some(lua.create_function(set_modifiers)),
                                Some(lua.create_function(get_modifiers)),
                                Some(lua.create_function(set_modifiers))))?
        .build()
}

fn set_button<'lua>(lua: &'lua Lua, button: Table<'lua>)
                    -> rlua::Result<Value<'lua>> {
    unimplemented!()
}

fn get_button<'lua>(lua: &'lua Lua, button: Table<'lua>)
                    -> rlua::Result<Value<'lua>> {
    unimplemented!()
}

fn set_modifiers<'lua>(lua: &'lua Lua, button: Table<'lua>)
                       -> rlua::Result<Value<'lua>> {
    unimplemented!()
}

fn get_modifiers<'lua>(lua: &'lua Lua, button: Table<'lua>)
                    -> rlua::Result<Value<'lua>> {
    unimplemented!()
}

mod test {
    use rlua::Lua;
    use super::super::button;

    #[test]
    fn button_object_test() {
        use super::allocator;
        let lua = Lua::new();
        lua.globals().set("button0", allocator(&lua).unwrap());
        lua.globals().set("button1", allocator(&lua).unwrap());
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

    #[test]
    fn button_class_test() {
        let lua = Lua::new();
        let button_class = button::init(&lua).unwrap();
        lua.globals().set("button", button_class).unwrap();
        lua.eval(r#"
a_button = button()
assert(a_button.num == 1)
a_button.connect_signal("test", function(button) button.num = 2 end)
a_button.emit_signal("test")
assert(a_button.num == 2)
"#, None).unwrap()
    }

    #[test]
    fn button_property_test() {
        let lua = Lua::new();
        let button_class = button::init(&lua).unwrap();
        assert_eq!(button_class.properties().unwrap().len().unwrap(), 2);
        lua.globals().set("button", button_class).unwrap();
        lua.eval(r#"
a_button = button()
print(a_button.button)
a_button.emit_signal("button")
button.emit_signal("button")
"#, None).unwrap()
    }
}

