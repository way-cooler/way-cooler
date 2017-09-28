//! Warning: extreme macros

/// Creates a struct and implements `ToJson` and `Decodeable` from
/// rustc_serialize.
#[macro_export]
macro_rules! json_convertible {
    (  $(#[$attr:meta])*
       struct $name:ident { $($fname:ident : $ftype:ty),+  }  ) => {

        $(#[$attr])*
        pub struct $name {
            $($fname: $ftype),+
        }

        impl ::rustc_serialize::json::ToJson for $name {
            fn to_json(&self) -> ::rustc_serialize::json::Json {
                let mut tree = ::std::collections::BTreeMap::new();
                $( tree.insert(stringify!($fname).to_string(),
                                self.$fname.to_json()); )+
                ::rustc_serialize::json::Json::Object(tree)
            }
        }

        impl ::rustc_serialize::Decodable for $name {
            fn decode<D: ::rustc_serialize::Decoder>(d: &mut D) -> Result<$name, D::Error> {
                $( let $fname = try!(d.read_struct_field(
                    stringify!($fname), 0usize,
                    |f| ::rustc_serialize::Decodable::decode(f))); )+

                Ok($name {
                    $( $fname: $fname ),+
                })
            }
        }
    }
}
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

/// Return from a test method if DUMMY_RUSTWLC is defined.
#[cfg(test)]
macro_rules! require_rustwlc {
    () => {
        if cfg!(test) {
            return;
        }
    }
}

/// Create a dbus interface object.
///
/// Given the path and name of a dbus interface
/// and a series of methods with type ipc::DBusResult, this macro
/// generates a big function `setup(&mut DBusFactory) -> DBusObjPath`
/// which will call the `Factory`'s `add_*` methods properly.
///
/// Currently limited to one dbus interface per invocation and requires setting
/// the name of the outputs.
macro_rules! dbus_interface {
    ( path: $obj_path:expr; name: $obj_name:expr;
      $(fn $fn_name:ident($($in_name:ident : $in_ty:ty),*)
                          -> $out_name:ident : DBusResult< $out_ty_inner:ty > { $($inner:tt)* })+ ) => {
        #[warn(dead_code)]
        #[allow(unused_mut)]
        pub fn setup(factory: &mut $crate::ipc::DBusFactory) -> $crate::ipc::DBusObjPath {
            return factory.object_path($obj_path, ()).introspectable()
                .add(factory.interface($obj_name, ())
                     $(
                         .add_m(factory.method(stringify!($fn_name), (),
                                move |msg| {
                                    let mut args_iter = msg.msg.iter_init();
                                    $(
                                        let $in_name: $in_ty = try!(args_iter.read::<$in_ty>());
                                    )*
                                    let result = $fn_name($($in_name),*);
                                    match result {
                                        Ok(value) => {
                                            let dbus_return = msg.msg.method_return().append(value);
                                            return Ok(vec![dbus_return])
                                        },
                                        Err(err) => {
                                            return Err(err)
                                        }
                                    }
                                })
                                .outarg::<$out_ty_inner, _>(stringify!($out_name))
                                $(
                                    .inarg::<$in_ty, _>(stringify!($in_name))
                                )*
                            )
                     )*
                );
        }
        $(
            #[allow(non_snake_case)]
            #[warn(dead_code)]
            fn $fn_name( $($in_name: $in_ty),* )
                         -> $crate::ipc::DBusResult<$out_ty_inner> {
                $($inner)*
            }
        )*
    };
}

macro_rules! map(
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = ::std::collections::HashMap::new();
            $(
                m.insert($key, $value);
            )+
                m
        }
    };
);

macro_rules! impl_objectable {
    ($WrapperType: ident, $StateType: ty) => {
        impl <'lua> Objectable<'lua, $WrapperType<'lua>, $StateType> for $WrapperType<'lua> {
            fn _wrap(table: Table<'lua>) -> $WrapperType {
                $WrapperType(table)
            }

            fn get_table(&self) -> Table<'lua> {
                self.0.clone()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rustc_serialize::Decodable;
    use rustc_serialize::json::{Decoder, ToJson};

    json_convertible! {
        #[derive(Debug, Clone, PartialEq)]
        struct Rectangle {
            height: u32,
            width: u32
        }
    }

    #[test]
    fn require_rustwlc() {
        require_rustwlc!();
        // If we're here we can use rustwlc.
        // If we tried to get a view or something it'd fail though.
        let _ = keypress!("Ctrl", "p");
    }

    #[test]
    fn json_convertible() {
        let rect = Rectangle { height: 1u32, width: 2u32 };
        let json_rect = rect.to_json();
        let maybe_rect = Rectangle::decode(&mut Decoder::new(json_rect));
        let parsed_rect = maybe_rect.expect("Unable to parse rectangle!");
        assert_eq!(parsed_rect, rect);
    }

    #[test]
    #[allow(deprecated)]
    fn keypress() {
        require_rustwlc!();
        use super::super::keys::KeyPress;
        #[allow(deprecated)]
        use std::hash::{SipHasher, Hash};

        let press = KeyPress::from_key_names(&["Ctrl"], "p")
            .expect("Unable to construct regular keypress");
        let press_macro = keypress!("Ctrl", "p");
        let mut hasher = SipHasher::new();
        assert!(press.hash(&mut hasher) == press_macro.hash(&mut hasher),
                "Hashes do not match");
        assert_eq!(press, press_macro);
    }
}
