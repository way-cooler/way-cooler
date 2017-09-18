use std::fmt::{self, Display, Formatter};
use std::default::Default;
use rlua::{self, Table, Lua, UserData, ToLua, Value};
use super::object::{Object, Objectable};
use super::property::Property;
use super::class::{self, Class};
#[allow(deprecated)]
use rustwlc::xkb::Keysym;
use rustwlc::types::KeyMod;

#[derive(Clone, Debug)]
pub struct ButtonState {
    button: Option<u32>,
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

    pub fn set_button(&self, new_val: Option<u32>) -> rlua::Result<()> {
        let mut button = self.0.get::<_, ButtonState>("data")?;
        button.button = new_val;
        self.0.set("data", button)?;
        Ok(())
    }

    pub fn modifiers(&self) -> rlua::Result<KeyMod> {
        let button = self.0.get::<_, ButtonState>("data")?;
        Ok(button.modifiers)
    }

    pub fn set_modifiers(&self, mods: Table<'lua>) -> rlua::Result<()> {
        use ::lua::mods_to_rust;
        let mut button = self.0.get::<_, ButtonState>("data")?;
        button.modifiers = mods_to_rust(mods)?;
        self.0.set("data", button)?;
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

// TODO Try to see if I can make this pass in an Object,
// or even better a Button

fn set_button<'lua>(_: &'lua Lua, (table, val): (Table, Value))
                    -> rlua::Result<Value<'lua>> {
    use rlua::Value::*;
    let button = Button::cast(table.into())?;
    match val {
        Number(num) => button.set_button(Some(num as _))?,
        Integer(num) => button.set_button(Some(num as _))?,
        _ => button.set_button(None)?
    }
    Ok(Value::Nil)
}

fn get_button<'lua>(_: &'lua Lua, table: Table<'lua>)
                    -> rlua::Result<Value<'lua>> {
    Button::cast(table.into())?.button()
}

fn set_modifiers<'lua>(_: &'lua Lua, (table, modifiers): (Table, Table))
                       -> rlua::Result<Value<'lua>> {
    let button = Button::cast(table.into())?;
    button.set_modifiers(modifiers)?;
    Ok(Value::Nil)
}

fn get_modifiers<'lua>(lua: &'lua Lua, table: Table<'lua>)
                    -> rlua::Result<Value<'lua>> {
    use ::lua::mods_to_lua;
    mods_to_lua(lua, Button::cast(table.into())?.modifiers()?).map(Value::Table)
}

#[cfg(test)]
mod test {
    use rlua::{self, Table, Lua};
    use super::super::button;
    use super::super::object;

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
"#, None).unwrap()
    }

    #[test]
    fn button_class_test() {
        let lua = Lua::new();
        let button_class = button::init(&lua).unwrap();
        lua.globals().set("button", button_class).unwrap();
        lua.eval(r#"
a_button = button()
assert(a_button.button == nil)
a_button.connect_signal("test", function(button) button.button = 2 end)
a_button.emit_signal("test")
assert(a_button.button == 2)
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
 a_button.emit_signal("test", nil)
 assert(a_button.button == nil)
 "#, None).unwrap()
    }

    #[test]
    fn button_modifiers_test() {
        use rustwlc::*;
        use self::button::Button;
        use self::object::{Object, Objectable};
        let lua = Lua::new();
        button::init(&lua).unwrap();
        lua.globals().set("a_button", button::allocator(&lua).unwrap());
        let button = Button::cast(lua.globals().get::<_, Table>("a_button")
                                  .unwrap().into()).unwrap();
        assert_eq!(button.modifiers().unwrap(), KeyMod::empty());
        lua.eval::<()>(r#"
a_button.modifiers = { "Caps" }
"#, None).unwrap();
        assert_eq!(button.modifiers().unwrap(), MOD_CAPS);
    }
}

