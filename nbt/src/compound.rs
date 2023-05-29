use std::ops::{Deref, DerefMut};

use crate::*;

#[derive(Default, Debug, Clone, PartialEq)]
#[repr(transparent)]
pub struct Compound<'a>(HashMap<Cow<'a, str>, Value<'a>>);

impl<'a> Deref for Compound<'a> {
    type Target = HashMap<Cow<'a, str>, Value<'a>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> DerefMut for Compound<'a> {    
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> Compound<'a> {
    pub const fn new(map: HashMap<Cow<'a, str>, Value<'a>>) -> Self {
        Compound(map)
    }
    pub fn into_map(self) -> HashMap<Cow<'a, str>, Value<'a>> {
        self.0
    }
}

#[cfg(feature = "to_static")]
impl<'a> ToStatic for Compound<'a> {
    type Static = Compound<'static>;
    fn to_static(&self) -> Self::Static {
        Compound(self.0.to_static())
    }
    fn into_static(self) -> Self::Static {
        Compound(self.0.into_static())
    }
}

impl<'a> Encode for Compound<'a> {
    fn encode(&self, writer: &mut impl std::io::Write) -> miners_encoding::encode::Result<()> {
        for (name, value) in self.0.iter() {
            let key = Mutf8::from(name);
            match value {
                Value::Byte(byte) => {
                    NbtTag::Byte.encode(writer)?;
                    key.encode(writer)?;
                    byte.encode(writer)?;
                }
                Value::Short(short) => {
                    NbtTag::Short.encode(writer)?;
                    key.encode(writer)?;
                    short.encode(writer)?;
                }
                Value::Int(int) => {
                    NbtTag::Int.encode(writer)?;
                    key.encode(writer)?;
                    int.encode(writer)?;
                }
                Value::Long(long) => {
                    NbtTag::Long.encode(writer)?;
                    key.encode(writer)?;
                    long.encode(writer)?;
                }
                Value::Float(float) => {
                    NbtTag::Float.encode(writer)?;
                    key.encode(writer)?;
                    float.encode(writer)?;
                }
                Value::Double(double) => {
                    NbtTag::Double.encode(writer)?;
                    key.encode(writer)?;
                    double.encode(writer)?;
                }
                Value::ByteArray(bytearray) => {
                    NbtTag::ByteArray.encode(writer)?;
                    key.encode(writer)?;
                    <&Counted<_, i32>>::from(bytearray).encode(writer)?;
                }
                Value::String(string) => {
                    NbtTag::String.encode(writer)?;
                    key.encode(writer)?;
                    Mutf8::from(string).encode(writer)?;
                }
                Value::List(list) => {
                    NbtTag::List.encode(writer)?;
                    key.encode(writer)?;
                    list.encode(writer)?;
                }
                Value::Compound(compound) => {
                    NbtTag::Compound.encode(writer)?;
                    key.encode(writer)?;
                    compound.encode(writer)?;
                }
                Value::IntArray(intarray) => {
                    NbtTag::IntArray.encode(writer)?;
                    key.encode(writer)?;
                    <&Counted<_, i32>>::from(intarray).encode(writer)?;
                }
                Value::LongArray(longarray) => {
                    NbtTag::LongArray.encode(writer)?;
                    key.encode(writer)?;
                    <&Counted<_, i32>>::from(longarray).encode(writer)?;
                }
            }
        }
        NbtTag::End.encode(writer)
    }
}
impl<'dec: 'a, 'a> Decode<'dec> for Compound<'a> {
    fn decode(cursor: &mut std::io::Cursor<&'dec [u8]>) -> decode::Result<Self> {
        let mut this = HashMap::default();
        loop {
            let tag = match NbtTag::decode(cursor) {
                Ok(NbtTag::End) => break Ok(Compound(this)),
                Err(decode::Error::Io(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    break Ok(Compound(this))
                }
                Err(e) => return Err(e),
                Ok(tag) => tag,
            };
            use std::collections::hash_map::Entry;
            let key = Mutf8::decode(cursor)?.into_inner();
            let entry = match this.entry(key) {
                Entry::Occupied(_) => {
                    return Err(decode::Error::Custom("duplicate key in compound"))
                }
                Entry::Vacant(entry) => entry,
            };
            let value = match tag {
                // Safety: This is safe because this case is handled earlier.
                NbtTag::End => unsafe { unreachable_unchecked() },
                NbtTag::Byte => Value::Byte(i8::decode(cursor)?),
                NbtTag::Short => Value::Short(i16::decode(cursor)?),
                NbtTag::Int => Value::Int(i32::decode(cursor)?),
                NbtTag::Long => Value::Long(i64::decode(cursor)?),
                NbtTag::Float => Value::Float(f32::decode(cursor)?),
                NbtTag::Double => Value::Double(f64::decode(cursor)?),
                NbtTag::ByteArray => {
                    Value::ByteArray(<Counted<Cow<[u8]>, i32>>::decode(cursor)?.inner)
                }
                NbtTag::String => Value::String(Mutf8::decode(cursor)?.into_inner()),
                NbtTag::List => Value::List(List::decode(cursor)?),
                NbtTag::Compound => Value::Compound(Compound::decode(cursor)?),
                NbtTag::IntArray => {
                    Value::IntArray(<Counted<Vec<i32>, i32>>::decode(cursor)?.inner)
                }
                NbtTag::LongArray => {
                    Value::LongArray(<Counted<Vec<i64>, i32>>::decode(cursor)?.inner)
                }
            };
            entry.insert(value);
        }
    }
}
