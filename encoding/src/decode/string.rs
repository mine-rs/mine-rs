use crate::*;

impl<'dec> Decode<'dec> for String {
    fn decode(cursor: &mut Cursor<&'dec [u8]>) -> decode::Result<Self> {
        let str = <&str>::decode(cursor)?;
        Ok(str.to_owned())
    }
}
