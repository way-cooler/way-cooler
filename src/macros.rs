//! Warning: extreme macros

/// This macro creates structs that implement `ToTable` and `FromTable`.
///
/// # Examples:
///
/// ```rust
/// lua_convertible! {
///     #[derive(Debug)]
///     // #[attribtue]
///     struct Point {
///         x: i32,
///         y: i32
///     }
/// }
/// ```
#[macro_export]
macro_rules! lua_convertible {
    (  $(#[$attr:meta])*
       struct $name:ident { $($fname:ident : $ftype:ty),+  }  ) => {

        $(#[$attr])*
        pub struct $name {
            $($fname: $ftype),+
        }

        impl $crate::convert::ToTable for $name {
            fn to_table(self) -> hlua::any::AnyLuaValue {
                hlua::any::AnyLuaValue::LuaArray(vec![
                    $(  (hlua::any::AnyLuaValue::LuaString(stringify!($fname).to_string()),
                         self.$fname.to_table())  ),+
                ])
            }
        }

        impl $crate::convert::FromTable for $name {
            #[allow(unused_variables)]
            fn from_table(decoder: $crate::convert::LuaDecoder) ->
                $crate::convert::ConvertResult<$name> {
                $(  let (decoder, $fname) =
                    try!(decoder.read_field(stringify!($fname).to_string())); )+

                Ok($name {
                    $( $fname: $fname ),+
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::convert::{ToTable, FromTable, LuaDecoder};
    use hlua;
    lua_convertible! {
        #[derive(Debug, Clone, PartialEq)]
        struct Point {
            x: f32,
            y: f32
        }
    }

    #[test]
    fn test_lua_convertible() {
        let point = Point { x: 0f32, y: 0f32 };
        let lua_point = point.clone().to_table();
        let maybe_point = Point::from_table(LuaDecoder::new(lua_point));
        assert!(maybe_point.is_ok());
        let parsed_point = maybe_point.unwrap();
        assert_eq!(parsed_point, point);
    }
}
