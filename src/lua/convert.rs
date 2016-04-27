use hlua::any::AnyLuaValue;
use hlua::any::AnyLuaValue::*;

use std::ops::Deref;
use std::iter::Iterator;

pub trait ToTable {
    fn to_table(&self) -> AnyLuaValue;
}

pub trait FromTable {
    fn from_table(decoder: &mut LuaDecoder) -> Self;
}

macro_rules! numeric_impl {
    ($($typ:ty), +) => {
        $(impl ToTable for $typ {
            fn to_table(&self) -> AnyLuaValue {
                LuaNumber(*self as f64)
            }
        })+
    }
}

numeric_impl!(usize, isize);
numeric_impl!(i8, i16, i32);
numeric_impl!(u8, u16, u32);
numeric_impl!(f32, f64);

impl ToTable for bool {
    fn to_table(&self) -> AnyLuaValue {
        LuaBoolean(*self)
    }
}

impl ToTable for String {
    fn to_table(&self) -> AnyLuaValue {
        LuaString((*self).clone())
    }
}

impl<T: ToTable> ToTable for Option<T> {
    fn to_table(&self) -> AnyLuaValue {
        match self {
            &Some(ref val) => val.to_table(),
            &None => LuaNil
        }
    }
}

impl ToTable for () {
    fn to_table(&self) -> AnyLuaValue {
        LuaNil
    }
}

/// Errors a converter can run into
enum ConverterError {
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

/*

$name =>


struct $name { ($field : $type, )+ }


( ($meta )+ (struct $name { ($field : $type, )+ }); )+


 */
