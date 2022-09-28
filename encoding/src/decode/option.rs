use crate::*;

impl<'dec, T> Decode<'dec> for Option<T> where T: Decode<'dec> {
    fn decode(cursor: &mut Cursor<&'dec [u8]>) -> decode::Result<Self> {
        Ok(if bool::decode(cursor)? {
            Some(T::decode(cursor)?)
        } else {
            None
        })
    }
}
