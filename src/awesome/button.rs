use std::fmt::{self, Display, Formatter};
use std::default::Default;
use rlua::{self, Table, Lua, UserData, AnyUserData, UserDataMethods, ToLua,
           Value};
use super::object::{self, Object, Objectable};
use super::property::Property;
use super::class::{self, Class};
use rustwlc::xkb::Keysym;
use rustwlc::types::KeyMod;

#[derive(Clone, Debug)]
pub struct ButtonState {
    button: Option<i32>,
    modifiers: KeyMod
}

#[derive(Clone, Debug)]
pub struct Button<'lua>(Table<'lua>);

impl <'lua> Button<'lua> {
    pub fn button(&self) -> rlua::Result<Value<'lua>> {
        let button = self.0.get::<_, ButtonState>("data")?;
        Ok(button.button
            .map(|num| Value::Number(num as f64))
            .unwrap_or(Value::Nil))
    }

    pub fn set_button(&self, new_val: Option<i32>) -> rlua::Result<()> {
        let mut button = self.0.get::<_, ButtonState>("data")?;
        button.button = new_val;
        Ok(())
    }
}

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
        ButtonState {
            button: None,
            modifiers: KeyMod::empty()
        }
    }
}

impl UserData for ButtonState {}

/// Makes a new button stored in a table beside its signals
pub fn allocator(lua: &Lua) -> rlua::Result<Object> {
    let meta = lua.create_table();
    let class = class::button_class(lua)?;
    Ok(Button::new(lua, class)?
       .add_to_meta(meta)?
       .build())
}

pub fn new<'lua>(lua: &'lua Lua, _table: Table<'lua>)
                 -> rlua::Result<Object<'lua>> {
    allocator(lua)
}


pub fn init(lua: &Lua) -> rlua::Result<Class> {
    Class::new(lua, Some(allocator), None, None)?
        .method("__call".into(), lua.create_function(new))?
        .property(Property::new("button".into(),
                                Some(lua.create_function(set_button)),
                                Some(lua.create_function(get_button)),
                                Some(lua.create_function(set_button))))?
        .property(Property::new("modifiers".into(),
                                Some(lua.create_function(set_modifiers)),
                                Some(lua.create_function(get_modifiers)),
                                Some(lua.create_function(set_modifiers))))?
        .save_class("__button_class")?
        .build()
}

fn set_button<'lua>(lua: &'lua Lua, (table, num): (Table, i32))
                    -> rlua::Result<Value<'lua>> {
    let button = Button::cast(table.into())?;
    button.set_button(Some(num))?;
    Ok(Value::Nil)
}

fn get_button<'lua>(lua: &'lua Lua, table: Table<'lua>)
                    -> rlua::Result<Value<'lua>> {
    Button::cast(table.into())?.button()
}

fn set_modifiers<'lua>(lua: &'lua Lua, button: Table)
                       -> rlua::Result<Value<'lua>> {
    unimplemented!()
}

fn get_modifiers<'lua>(lua: &'lua Lua, button: Table)
                    -> rlua::Result<Value<'lua>> {
    unimplemented!()
}

mod test {
    use rlua::Lua;
    use super::super::button;

    #[test]
    fn button_object_test() {
        let lua = Lua::new();
        button::init(&lua).unwrap();
        lua.globals().set("button0", button::allocator(&lua).unwrap());
        lua.globals().set("button1", button::allocator(&lua).unwrap());
        lua.eval(r#"
 assert(button0.button == nil)
 assert(button1.button == nil)
 button0.connect_signal("test", function(button) button.button = 3 end)
 button0.emit_signal("test")
 assert(button1.button == nil)
 assert(button0.button == 3)
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
assert(a_button.button == nil)
a_button.connect_signal("test", function(button) button.num = 2 end)
a_button.emit_signal("test")
assert(a_button.num == 2)
"#, None).unwrap()
    }

    #[test]
    fn button_property_test() {
        let lua = Lua::new();
        button::init(&lua).unwrap();
        let button_class = button::init(&lua).unwrap();
        assert_eq!(button_class.properties().unwrap().len().unwrap(), 2);
        lua.globals().set("button", button_class).unwrap();
        lua.eval(r#"
a_button = button()
assert(a_button.button == nil)
a_button.button = 5
assert(a_button.button == 5)
"#, None).unwrap()
    }

    #[test]
    fn button_remove_signal_test() {
        let lua = Lua::new();
        let button_class = button::init(&lua).unwrap();
        lua.globals().set("button", button_class).unwrap();
        lua.eval(r#"
button0 = button()
assert(button0.button == nil)
button0.connect_signal("test", function(button) button.button = 3 end)
button0.emit_signal("test")
assert(button0.button == 3)
button0.button = 0
button0.disconnect_signal("test")
button0.emit_signal("test")
assert(button0.button == 0)
"#, None).unwrap()
    }

    #[test]
    fn button_emit_signal_multiple_args() {
        let lua = Lua::new();
        button::init(&lua).unwrap();
        lua.globals().set("a_button", button::allocator(&lua).unwrap());
        lua.eval(r#"
 assert(a_button.button == nil)
 a_button.connect_signal("test", function(button, num) button.button = num end)
 a_button.emit_signal("test", 5)
 assert(a_button.button == 5)
 a_button.emit_signal("test", -1)
 assert(a_button.button == -1)
 a_button.emit_signal("test", nil)
 assert(a_button.button == nil)
 "#, None).unwrap()
    }
}

