//! Module for implementing convert's ToTable.

use hlua::any::AnyLuaValue;
use hlua::any::AnyLuaValue::*;

/// Represents types which can be serialized into a Lua table (AnyLuaValue).
///
/// If you wish to continue using the object after serialization, consider
/// using `impl<a> ToTable for &'a T`, which will take an `&T` as its `self`.
pub trait ToTable {
    /// Write this value into an AnyLuaValue
    fn to_table(self) -> AnyLuaValue;
}

// Implementation for standard numeric types
macro_rules! numeric_impl {
    ($($typ:ty), +) => {
        $(impl ToTable for $typ {
            fn to_table(self) -> AnyLuaValue {
                LuaNumber(self as f64)
            }
        })+
    }
}

numeric_impl!(usize, isize);
numeric_impl!(i8, i16, i32);
numeric_impl!(u8, u16, u32);
numeric_impl!(f32, f64);

// Implementation for &Ts which use Copy syntax
impl<'a, T> ToTable for &'a T where T: Copy + ToTable {
    fn to_table(self) -> AnyLuaValue {
        self.clone().to_table()
    }
}

impl ToTable for bool {
    fn to_table(self) -> AnyLuaValue {
        LuaBoolean(self)
    }
}

impl ToTable for String {
    fn to_table(self) -> AnyLuaValue {
        LuaString(self)
    }
}

impl<T: ToTable> ToTable for Option<T> {
    fn to_table(self) -> AnyLuaValue {
        match self {
            Some(val) => val.to_table(),
            None => LuaNil
        }
    }
}

impl<T: ToTable> ToTable for Vec<T> {
    fn to_table(self) -> AnyLuaValue {
        LuaArray(self.into_iter().enumerate()
            .map(|(ix, val)| {
                (LuaNumber(ix as f64), val.to_table())
            }).collect())
    }
}

impl ToTable for () {
    fn to_table(self) -> AnyLuaValue {
        LuaNil
    }
}
