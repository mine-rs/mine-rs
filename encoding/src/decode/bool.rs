use crate::*;

impl<'dec> Decode<'dec> for bool {
    fn decode(cursor: &mut Cursor<&'dec [u8]>) -> decode::Result<Self> {
        let mut id = [0u8; 1];
        cursor.read_exact(&mut id)?;
        Ok(id[0] != 0)
    }
}
