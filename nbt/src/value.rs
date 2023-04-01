use std::borrow::Cow;

use crate::{List, Compound};

#[derive(Clone, Debug, PartialEq)]
pub enum Value<'a> {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Cow<'a, [u8]>),
    String(Cow<'a, str>),
    List(List<'a>),
    Compound(Compound<'a>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}
macro_rules! from {
    ($($case:ident $ufrom:ident $ifrom:ident;)+) => {$(
        impl<'a> From<$ifrom> for Value<'a> {
            fn from(val: $ifrom) -> Self {
                Value::$case(val)
            }
        }
        impl<'a> From<$ufrom> for Value<'a> {
            fn from(val: $ufrom) -> Self {
                Value::$case(val as $ifrom)
            }
        }
    )+};
    ($($case:ident $from:ty;)+) => {$(
        impl<'a> From<$from> for Value<'a> {
            fn from(val: $from) -> Self {
                Value::$case(val)
            }
        }
    )+}
}
from! {
    Byte u8 i8;
    Short u16 i16;
    Int u32 i32;
    Long u64 i64;
}
from! {
    Float f32;
    Double f64;
    ByteArray Cow<'a, [u8]>;
    String Cow<'a, str>;
    List List<'a>;
    Compound Compound<'a>;
    IntArray Vec<i32>;
    LongArray Vec<i64>;
}
impl<'a> From<&'a str> for Value<'a> {
    fn from(val: &'a str) -> Self {
        Value::String(val.into())
    }
}
impl<'a> From<String> for Value<'a> {
    fn from(val: String) -> Self {
        Value::String(val.into())
    }
}

#[cfg(feature = "to_static")]
impl<'a> ::miners_to_static::ToStatic for Value<'a> {
    type Static = Value<'static>;
    fn to_static(&self) -> Self::Static {
        match self {
            Self::Byte(byte) => Value::Byte(*byte),
            Self::Short(short) => Value::Short(*short),
            Self::Int(int) => Value::Int(*int),
            Self::Long(long) => Value::Long(*long),
            Self::Float(float) => Value::Float(*float),
            Self::Double(double) => Value::Double(*double),
            Self::ByteArray(bytearray) => Value::ByteArray(bytearray.to_static()),
            Self::String(string) => Value::String(string.to_static()),
            Self::List(list) => Value::List(list.to_static()),
            Self::Compound(compound) => Value::Compound(compound.to_static()),
            Self::IntArray(intarray) => Value::IntArray(intarray.to_static()),
            Self::LongArray(longarray) => Value::LongArray(longarray.to_static()),
        }
    }
    fn into_static(self) -> Self::Static {
        match self {
            Self::Byte(byte) => Value::Byte(byte),
            Self::Short(short) => Value::Short(short),
            Self::Int(int) => Value::Int(int),
            Self::Long(long) => Value::Long(long),
            Self::Float(float) => Value::Float(float),
            Self::Double(double) => Value::Double(double),
            Self::ByteArray(bytearray) => Value::ByteArray(bytearray.into_static()),
            Self::String(string) => Value::String(string.into_static()),
            Self::List(list) => Value::List(list.into_static()),
            Self::Compound(compound) => Value::Compound(compound.into_static()),
            Self::IntArray(intarray) => Value::IntArray(intarray.into_static()),
            Self::LongArray(longarray) => Value::LongArray(longarray.into_static()),
        }
    }
}
