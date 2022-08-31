use crate::*;
use std::borrow::Cow;

impl<'dec, T> Decode<'dec> for Cow<'dec, [T]>
where
    &'dec [T]: Decode<'dec>,
    [T]: ToOwned,
{
    fn decode(cursor: &mut Cursor<&'dec [u8]>) -> decode::Result<Self> {
        let slice = <&[T]>::decode(cursor)?;
        Ok(Cow::Borrowed(slice))
    }
}

impl<'dec> Decode<'dec> for Cow<'dec, str> {
    fn decode(cursor: &mut Cursor<&'dec [u8]>) -> decode::Result<Self> {
        let slice = <&str>::decode(cursor)?;
        Ok(Cow::Borrowed(slice))
    }
}

impl<'dec, T> Decode<'dec> for Cow<'dec, T>
where
    &'dec T: Decode<'dec>,
    T: ToOwned,
{
    fn decode(cursor: &mut Cursor<&'dec [u8]>) -> decode::Result<Self> {
        let value = <&T>::decode(cursor)?;
        Ok(Cow::Borrowed(value))
    }
}
