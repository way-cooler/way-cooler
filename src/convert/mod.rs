use hlua::any::AnyLuaValue;
use hlua::any::AnyLuaValue::*;

use std::ops::Deref;
use std::iter::Iterator;

/// Represents types which can be serialized into a Lua table (AnyLuaValue).
///
/// If you wish to continue using the object after serialization, consider
/// using `impl<a> ToTable for &'a T`, which will take an `&T` as its `self`.
pub trait ToTable {
    /// Write this value into an AnyLuaValue
    fn to_table(self) -> AnyLuaValue;
}

/// Represents types which can be serialized from a Lua table (AnyLuaValue).
///
/// For convenience, this method takes in a `LuaDecoder` (obtained from
/// `LuaDecoder::new(AnyLuaValue)`). See methods on `LuaDecoder`.
pub trait FromTable {
    /// Attempt to parse this value from the decoder.
    fn from_table(decoder: &mut LuaDecoder) -> ConvertResult<Self>
        where Self: Sized;
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
impl<'a, T> ToTable for &'a T where T: Copy {
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

impl ToTable for () {
    fn to_table(self) -> AnyLuaValue {
        LuaNil
    }
}

/// Errors a converter can run into
pub enum ConverterError {
    /// The type of value present did not match the one expected
    UnexpectedType(AnyLuaValue),
    /// The table index expected did not exist
    MissingTableIndex(String),
    /// The table index present was not valid
    InvalidTableIndex(AnyLuaValue)
}

pub type ConvertResult<T> = Result<T, ConverterError>;

/// Can decode values with a FromTable
pub struct LuaDecoder {
    val: AnyLuaValue
}

impl LuaDecoder {
    pub fn new(val: AnyLuaValue) -> LuaDecoder {
        LuaDecoder { val: val }
    }

    pub fn get_u32(&self) -> ConvertResult<u32> {
        unimplemented!()
    }

    pub fn get_i32(&self) -> ConvertResult<i32> {
        unimplemented!()
    }

    pub fn get_f64(&self) -> ConvertResult<i32> {
        unimplemented!()
    }

    pub fn get_string(&self) -> ConvertResult<String> {
        unimplemented!()
    }

    pub fn read_field<T, F>(&self, func: F) -> ConvertResult<T> {
        unimplemented!()
    }
}
