use crate::*;
use attrs::Var;

impl<'dec> Decode<'dec> for &'dec [u8] {
    fn decode(cursor: &mut Cursor<&'dec [u8]>) -> decode::Result<Self> {
        let Var(count) = <Var<u32>>::decode(cursor)?;
        let pos = cursor.position() as usize;
        let slice = cursor
            .get_ref()
            .get(pos..pos + count as usize)
            .ok_or(decode::Error::UnexpectedEndOfSlice)?;
        cursor.set_position(pos as u64 + count as u64);
        Ok(slice)
    }
}
