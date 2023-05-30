pub mod compound;
pub mod list;
pub mod macros;
pub mod tag;
pub mod value;

pub use compound::Compound;
pub use list::List;
pub use tag::NbtTag;
pub use value::Value;

pub(crate) use miners_encoding::attrs::{Counted, Mutf8};
pub(crate) use miners_encoding::{decode, Decode, Encode};
#[cfg(feature = "to_static")]
pub(crate) use miners_to_static::ToStatic;
use std::ops::{Deref, DerefMut};
pub(crate) use std::{borrow::Cow, collections::HashMap, hint::unreachable_unchecked};


#[derive(Debug, Clone)]
pub struct Nbt<'a> {
    pub name: Cow<'a, str>,
    pub data: Compound<'a>
}

impl<'a> Deref for Nbt<'a> {
    type Target = Compound<'a>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'a> DerefMut for Nbt<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<'a> ToStatic for Nbt<'a> {
    type Static = Nbt<'static>;

    fn to_static(&self) -> Self::Static {
        Self::Static {
            name: self.name.to_static(),
            data: self.data.to_static()
        }
    }

    fn into_static(self) -> Self::Static {
        Self::Static {
            name: self.name.into_static(),
            data: self.data.into_static()
        }
    }
}

impl<'dec> Decode<'dec> for Nbt<'dec> {
    fn decode(cursor: &mut std::io::Cursor<&'dec [u8]>) -> decode::Result<Self> {
        let tag = NbtTag::decode(cursor)?;
        if !matches!(tag, NbtTag::Compound) {
            return Err(miners_encoding::decode::Error::InvalidId);
        }
        let name = miners_encoding::attrs::Mutf8::decode(cursor)?.into_inner();
        let data = Compound::decode(cursor)?;
        Ok(
            Self {
                name,
                data
            }
        )
    }
}

impl<'a> Encode for Nbt<'a> {
    fn encode(&self, writer: &mut impl std::io::Write) -> miners_encoding::encode::Result<()> {
        NbtTag::Compound.encode(writer)?;
        miners_encoding::attrs::Mutf8::from(&self.name).encode(writer)?;
        self.data.encode(writer)?;
        todo!()
    }
}
