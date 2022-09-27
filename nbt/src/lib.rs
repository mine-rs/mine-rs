pub mod compound;
pub mod list;
pub mod macros;
pub mod tag;
pub mod value;

pub use compound::Compound;
pub use list::List;
pub use tag::NbtTag;
pub use value::Value;

pub(crate) use miners_encoding::{attrs::Counted, decode, Decode, Encode};
#[cfg(feature = "to_static")]
pub(crate) use miners_to_static::ToStatic;
pub(crate) use std::{borrow::Cow, collections::BTreeMap, hint::unreachable_unchecked};

#[repr(transparent)]
pub(crate) struct Mutf8<'a>(Cow<'a, str>);
impl<'dec, 'a> Decode<'dec> for Mutf8<'a>
where
    'dec: 'a,
{
    fn decode(cursor: &mut std::io::Cursor<&'dec [u8]>) -> decode::Result<Self> {
        let bytes = &<&Counted<[u8], u16>>::decode(cursor)?.inner;
        mutf8::decode(bytes)
            .map_err(|_| decode::Error::Custom("invalid mutf8 in nbt"))
            .map(Mutf8)
    }
}
impl<'a> Encode for Mutf8<'a> {
    fn encode(&self, writer: &mut impl std::io::Write) -> miners_encoding::encode::Result<()> {
        <&Counted<Cow<[u8]>, u16>>::from(&mutf8::encode(&self.0)).encode(writer)
    }
}
impl<'a> Mutf8<'a> {
    #[allow(clippy::ptr_arg)]
    fn from<'string>(string: &'string Cow<'a, str>) -> &'string Mutf8<'a> {
        // SAFETY: Mutf8 is #[repr(transparent)]
        unsafe { std::mem::transmute(string) }
    }
}
