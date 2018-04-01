use std::fmt::{self, Display, Formatter};
use std::default::Default;
use rlua::{self, Table, Lua, UserData, ToLua, Value, AnyUserData, UserDataMethods};
use super::object::{self, Object, Objectable};
use super::signal;
use super::property::Property;
use super::class::{self, Class};
use wlroots::events::key_events::Key;
use xcb::ffi::xproto::xcb_button_t;

#[derive(Clone, Debug)]
pub struct ButtonState {
    button: xcb_button_t,
    modifiers: Vec<Key>
}

#[derive(Clone, Debug)]
pub struct Button<'lua>(Object<'lua>);

impl Display for ButtonState {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Button: {:p}", self)
    }
}

impl Default for ButtonState {
    fn default() -> Self {
        ButtonState {
            button: xcb_button_t::default(),
            modifiers: Vec::new()
        }
    }
}

impl UserData for ButtonState {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        object::default_add_methods(methods);
    }
}

impl <'lua> Button<'lua> {
    fn new(lua: &'lua Lua, args: rlua::Table) -> rlua::Result<Object<'lua>> {
        let class = class::class_setup(lua, "button")?;
        Ok(Button::allocate(lua, class)?
           .handle_constructor_argument(args)?
           .build())
    }

    pub fn button(&self) -> rlua::Result<Value<'lua>> {
        let button = self.state()?;
        Ok(Value::Integer(button.button as _))
    }

    pub fn set_button(&mut self, new_val: xcb_button_t) -> rlua::Result<()> {
        let mut button = self.get_object_mut()?;
        button.button = new_val;
        Ok(())
    }

    pub fn modifiers(&self) -> rlua::Result<Vec<Key>> {
        let button = self.state()?;
        Ok(button.modifiers)
    }

    pub fn set_modifiers(&mut self, mods: Table<'lua>) -> rlua::Result<()> {
        use ::lua::mods_to_rust;
        let mut button = self.get_object_mut()?;
        button.modifiers = mods_to_rust(mods)?;
        Ok(())
    }
}

impl <'lua> ToLua<'lua> for Button<'lua> {
    fn to_lua(self, lua: &'lua Lua) -> rlua::Result<Value<'lua>> {
        self.0.to_lua(lua)
    }
}

impl_objectable!(Button, ButtonState);


pub fn init(lua: &Lua) -> rlua::Result<Class> {
    Class::builder(lua, "button", None)?
        .method("__call".into(),
                lua.create_function(|lua, args: rlua::Table|
                                    Button::new(lua, args))?)?
        .property(Property::new("button".into(),
                                Some(lua.create_function(set_button)?),
                                Some(lua.create_function(get_button)?),
                                Some(lua.create_function(set_button)?)))?
        .property(Property::new("modifiers".into(),
                                Some(lua.create_function(set_modifiers)?),
                                Some(lua.create_function(get_modifiers)?),
                                Some(lua.create_function(set_modifiers)?)))?
        .save_class("button")?
        .build()
}

fn set_button<'lua>(lua: &'lua Lua, (obj, val): (AnyUserData<'lua>, Value<'lua>))
                    -> rlua::Result<Value<'lua>> {
    use rlua::Value::*;
    let mut button = Button::cast(obj.clone().into())?;
    match val {
        Number(num) => button.set_button(num as _)?,
        Integer(num) => button.set_button(num as _)?,
        _ => button.set_button(xcb_button_t::default())?
    }
    signal::emit_object_signal(lua,
                        obj.into(),
                        "property::button".into(),
                        val)?;
    Ok(Value::Nil)
}

fn get_button<'lua>(_: &'lua Lua, obj: AnyUserData<'lua>)
                    -> rlua::Result<Value<'lua>> {
    Button::cast(obj.into())?.button()
}

fn set_modifiers<'lua>(lua: &'lua Lua, (obj, modifiers): (AnyUserData<'lua>, Table<'lua>))
                       -> rlua::Result<Value<'lua>> {
    let mut button = Button::cast(obj.clone().into())?;
    button.set_modifiers(modifiers.clone())?;
    signal::emit_object_signal(lua,
                        obj.into(),
                        "property::modifiers".into(),
                        modifiers)?;
    Ok(Value::Nil)
}

fn get_modifiers<'lua>(lua: &'lua Lua, obj: AnyUserData<'lua>)
                    -> rlua::Result<Value<'lua>> {
    use ::lua::mods_to_lua;
    mods_to_lua(lua, &Button::cast(obj.into())?.modifiers()?).map(Value::Table)
}

#[cfg(test)]
mod test {
    use rlua::{AnyUserData, Lua};
    use super::super::button::{self, Button};
    use super::super::object;

    #[test]
    fn button_object_test() {
        let lua = Lua::new();
        button::init(&lua).unwrap();
        lua.globals().set("button0", Button::new(&lua, lua.create_table().unwrap()).unwrap())
            .unwrap();
        lua.globals().set("button1", Button::new(&lua, lua.create_table().unwrap()).unwrap())
            .unwrap();
        lua.eval(r#"
assert(button0.button == 0)
assert(button1.button == 0)
button0:connect_signal("test", function(button) button.button = 3 end)
button0:emit_signal("test")
assert(button1.button == 0)
assert(button0.button == 3)
"#, None).unwrap()
    }

    #[test]
    fn button_class_test() {
        let lua = Lua::new();
        button::init(&lua).unwrap();
        lua.eval(r#"
a_button = button{}
assert(a_button.button == 0)
a_button:connect_signal("test", function(button) button.button = 2 end)
a_button:emit_signal("test")
assert(a_button.button == 2)
"#, None).unwrap()
    }

    #[test]
    fn button_property_test() {
        let lua = Lua::new();
        let button_class = button::init(&lua).unwrap();
        use rlua::{self, Table, ToLua};
        let mut count = -1;
        if let rlua::Value::UserData(any) = button_class.to_lua(&lua).unwrap() {
            count = any.get_user_value::<Table>().unwrap().get::<_, Table>("properties").unwrap().len().unwrap();
        }
        assert_eq!(count, 2);
        lua.eval(r#"
a_button = button{}
assert(a_button.button == 0)
a_button.button = 5
assert(a_button.button == 5)
"#, None).unwrap()
    }

    #[test]
    fn button_remove_signal_test() {
        let lua = Lua::new();
        button::init(&lua).unwrap();
        lua.eval(r#"
button0 = button{}
assert(button0.button == 0)
button0:connect_signal("test", function(button) button.button = 3 end)
button0:emit_signal("test")
assert(button0.button == 3)
button0.button = 0
button0:disconnect_signal("test")
button0:emit_signal("test")
assert(button0.button == 0)
"#, None).unwrap()
    }

    #[test]
    fn button_emit_signal_multiple_args() {
        let lua = Lua::new();
        button::init(&lua).unwrap();
        lua.globals().set("a_button", Button::new(&lua, lua.create_table().unwrap()).unwrap())
            .unwrap();
        lua.eval(r#"
 assert(a_button.button == 0)
 a_button:connect_signal("test", function(button, num) button.button = num end)
 a_button:emit_signal("test", 5)
 assert(a_button.button == 5)
 a_button:emit_signal("test", 0)
 assert(a_button.button == 0)
 "#, None).unwrap()
    }

    #[test]
    fn button_modifiers_test() {
        use rustwlc::*;
        use self::button::Button;
        use self::object::Objectable;
        let lua = Lua::new();
        button::init(&lua).unwrap();
        lua.globals().set("a_button", Button::new(&lua, lua.create_table().unwrap()).unwrap())
            .unwrap();
        let button = Button::cast(lua.globals().get::<_, AnyUserData>("a_button")
                                  .unwrap().into()).unwrap();
        assert_eq!(button.modifiers().unwrap(), KeyMod::empty());
        lua.eval::<()>(r#"
a_button.modifiers = { "Caps" }
"#, None).unwrap();
        assert_eq!(button.modifiers().unwrap(), MOD_CAPS);
    }

    #[test]
    fn button_multiple_modifiers_test() {
        use rustwlc::*;
        use self::button::Button;
        use self::object::Objectable;
        let lua = Lua::new();
        button::init(&lua).unwrap();
        lua.globals().set("a_button", Button::new(&lua, lua.create_table().unwrap()).unwrap())
            .unwrap();
        let button = Button::cast(lua.globals().get::<_, AnyUserData>("a_button")
                                  .unwrap().into()).unwrap();
        assert_eq!(button.modifiers().unwrap(), KeyMod::empty());
        lua.eval::<()>(r#"
a_button.modifiers = { "Caps", "Mod2" }
"#, None).unwrap();
        assert_eq!(button.modifiers().unwrap(), MOD_CAPS | MOD_MOD2);
    }

    #[test]
    /// Tests that setting the button index property updates the
    /// callback for all instances of button
    fn button_index_property() {
        let lua = Lua::new();
        button::init(&lua).unwrap();
        lua.globals().set("a_button", Button::new(&lua, lua.create_table().unwrap()).unwrap())
            .unwrap();
        lua.eval::<()>(r#"
hit = false
button.set_index_miss_handler(function(button)
    hit = true
    return 5
end)
assert(not hit)
a = button.button
assert(a ~= 5)
assert(not hit)
a = a_button.aoeu
assert(hit)
assert(a == 5)
a = nil
hit = false
a = button{}.aoeu
assert(hit)
assert(a == 5)
"#, None).unwrap()
    }

    #[test]
    fn button_modifiers_signal_test() {
        let lua = Lua::new();
        button::init(&lua).unwrap();
        lua.eval::<()>(r#"
a_button = button{}
hit = false
a_button:connect_signal("property::modifiers", function(button) hit = true end)
a_button:emit_signal("property::modifiers")
assert(hit)
hit = false
assert(not hit)
a_button.button = nil
assert(not hit)
a_button.modifiers = { "Caps", "Mod2" }
assert(hit)
"#, None).unwrap()
    }

    #[test]
    fn button_button_signal_test() {
        let lua = Lua::new();
        button::init(&lua).unwrap();
        lua.eval::<()>(r#"
a_button = button{}
hit = false
a_button:connect_signal("property::button", function(button) hit = true end)
a_button:emit_signal("property::button")
assert(hit)
hit = false
assert(not hit)
a_button.button = nil
assert(hit)
"#, None).unwrap()
    }

    #[test]
    fn button_test_valid() {
        let lua = Lua::new();
        button::init(&lua).unwrap();
        lua.eval::<()>(r#"
a_button = button{}
assert(a_button.valid)
"#, None).unwrap();
    }
}

