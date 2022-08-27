use crate::*;
use std::io::Read;

impl ProtocolRead<'_> for bool {
    fn read(cursor: &mut ::std::io::Cursor<&[u8]>) -> Result<Self, ReadError> {
        let mut id = [0u8; 1];
        cursor.read_exact(&mut id)?;
        Ok(id[0] != 0)
    }
}
impl ProtocolWrite for bool {
    fn write(self, writer: &mut impl std::io::Write) -> Result<(), WriteError> {
        writer.write_all(&[self as u8])?;
        Ok(())
    }
    #[inline(always)]
    fn size_hint() -> usize {
        1
    }
}
