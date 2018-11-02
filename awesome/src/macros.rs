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
