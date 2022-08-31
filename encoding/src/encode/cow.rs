use crate::*;
use std::borrow::Cow;

impl<'enc, T> Encode for Cow<'enc, [T]>
where
    [T]: Encode,
    [T]: ToOwned,
{
    fn encode(&self, writer: &mut impl Write) -> encode::Result<()> {
        self.as_ref().encode(writer)
    }
}

impl<'enc> Encode for Cow<'enc, str> {
    fn encode(&self, writer: &mut impl Write) -> encode::Result<()> {
        self.as_bytes().encode(writer)
    }
}

impl<'enc, T> Encode for Cow<'enc, T>
where
    T: Encode,
    T: ToOwned,
{
    fn encode(&self, writer: &mut impl Write) -> encode::Result<()> {
        self.as_ref().encode(writer)
    }
}
