use std::borrow::Cow;

use miners_encoding::{attrs::Counted, Encode};
#[cfg(feature = "to_static")]
use miners_to_static::ToStatic;

use crate::{compound::Compound, list::List, tag::NbtTag};

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

#[cfg(feature = "to_static")]
impl<'a> ToStatic for Value<'a> {
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

impl<'a> Encode for Value<'a> {
    fn encode(&self, writer: &mut impl std::io::Write) -> miners_encoding::encode::Result<()> {
        match self {
            Self::Byte(byte) => {
                NbtTag::Byte.encode(writer)?;
                byte.encode(writer)
            }
            Self::Short(short) => {
                NbtTag::Short.encode(writer)?;
                short.encode(writer)
            }
            Self::Int(int) => {
                NbtTag::Int.encode(writer)?;
                int.encode(writer)
            }
            Self::Long(long) => {
                NbtTag::Long.encode(writer)?;
                long.encode(writer)
            }
            Self::Float(float) => {
                NbtTag::Float.encode(writer)?;
                float.encode(writer)
            }
            Self::Double(double) => {
                NbtTag::Double.encode(writer)?;
                double.encode(writer)
            }
            Self::ByteArray(bytearray) => {
                NbtTag::ByteArray.encode(writer)?;
                <&Counted<_, i32>>::from(bytearray).encode(writer)
            }
            Self::String(string) => {
                NbtTag::String.encode(writer)?;
                <&Counted<_, u16>>::from(string).encode(writer)
            }
            Self::List(list) => {
                NbtTag::List.encode(writer)?;
                list.encode(writer)
            }
            Self::Compound(compound) => {
                NbtTag::Compound.encode(writer)?;
                compound.encode(writer)
            }
            Self::IntArray(intarray) => {
                NbtTag::IntArray.encode(writer)?;
                <&Counted<_, i32>>::from(intarray).encode(writer)
            }
            Self::LongArray(longarray) => {
                NbtTag::LongArray.encode(writer)?;
                <&Counted<_, i32>>::from(longarray).encode(writer)
            }
        }
    }
}
