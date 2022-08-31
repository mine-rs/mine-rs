use crate::*;

impl<'dec> Decode<'dec> for &'dec str {
    fn decode(cursor: &mut Cursor<&'dec [u8]>) -> decode::Result<Self> {
        let slice = <&[u8]>::decode(cursor)?;
        Ok(std::str::from_utf8(slice)?)
    }
}
