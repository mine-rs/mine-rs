use crate::*;

#[derive(Default)]
#[repr(transparent)]
pub struct Compound<'a>(BTreeMap<Cow<'a, str>, Value<'a>>);

impl<'a> Compound<'a> {
    pub fn new(map: BTreeMap<Cow<'a, str>, Value<'a>>) -> Self {
        Compound(map)
    }
    pub fn into_map(self) -> BTreeMap<Cow<'a, str>, Value<'a>> {
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
            u16::try_from(name.as_bytes().len())?.encode(writer)?;
            writer.write_all(name.as_bytes())?;
            value.encode(writer)?;
        }
        NbtTag::End.encode(writer)
    }
}
impl<'dec, 'a> Decode<'dec> for Compound<'a>
where
    'dec: 'a,
{
    fn decode(cursor: &mut std::io::Cursor<&'dec [u8]>) -> decode::Result<Self> {
        let mut this = BTreeMap::default();
        loop {
            let tag = match NbtTag::decode(cursor)? {
                NbtTag::End => break Ok(Compound(this)),
                tag => tag,
            };
            let key = Cow::Borrowed(&<&Counted<str, u16>>::decode(cursor)?.inner);
            use std::collections::btree_map::Entry;
            let entry = match this.entry(key) {
                Entry::Occupied(_) => {
                    return Err(decode::Error::Custom("duplicate key in compound"))
                }
                Entry::Vacant(entry) => entry,
            };
            let value = match tag {
                NbtTag::End => unsafe { unreachable_unchecked() },
                NbtTag::Byte => Value::Byte(i8::decode(cursor)?),
                NbtTag::Short => Value::Short(i16::decode(cursor)?),
                NbtTag::Int => Value::Int(i32::decode(cursor)?),
                NbtTag::Long => Value::Long(i64::decode(cursor)?),
                NbtTag::Float => Value::Float(f32::decode(cursor)?),
                NbtTag::Double => Value::Double(f64::decode(cursor)?),
                NbtTag::ByteArray => Value::ByteArray(Cow::decode(cursor)?),
                NbtTag::String => Value::String(Mutf8::decode(cursor)?.0),
                NbtTag::List => Value::List(List::decode(cursor)?),
                NbtTag::Compound => Value::Compound(Compound::decode(cursor)?),
                NbtTag::IntArray => Value::IntArray(Vec::decode(cursor)?),
                NbtTag::LongArray => Value::LongArray(Vec::decode(cursor)?),
            };
            entry.insert(value);
        }
    }
}
