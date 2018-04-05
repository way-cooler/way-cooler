//! Warning: extreme macros

/// Create a keypress using fewer keystrokes. Provides a custom panic method.
#[macro_export]
macro_rules! keypress {
    ($modifier:expr, $key:expr) => {
        $crate::keys::KeyPress::from_key_names(&[$modifier],
                                 $key)
            .expect(concat!("Unable to create keypress from macro with ",
                            $modifier, " and ", $key))
    };
}

macro_rules! impl_objectable {
    ($WrapperType: ident, $StateType: ty) => {
        impl <'lua> Objectable<'lua, $WrapperType<'lua>, $StateType> for $WrapperType<'lua> {
            fn _wrap(object: Object<'lua>) -> $WrapperType {
                $WrapperType(object)
            }

            fn get_object(&self) -> $crate::rlua::Result<$StateType> {
                Ok(self.0.object.borrow_mut::<$StateType>()?.clone())
            }

            fn get_object_mut(&mut self) -> $crate::rlua::Result<::std::cell::RefMut<$StateType>> {
                Ok(self.0.object.borrow_mut::<$StateType>()?)
            }
        }
    }
}

/// Inner try for use within a LocalKey with call.
///
/// We use an `Option<T>` to get the return value to "escape",
/// this simply lets you keep try mechanics with it.
///
/// Simply specify the "out" variable and then write the expression.
macro_rules! inner_try {
    ($i: ident, $b: block) => {
        $i = ::std::option::Option::Some((|| $b)())
    }
}
