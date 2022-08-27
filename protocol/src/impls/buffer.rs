use std::borrow::Cow;

use crate::{attrs::Var, *};

impl<'a> ProtocolRead<'a> for &'a [u8] {
    fn read(cursor: &mut std::io::Cursor<&'a [u8]>) -> Result<Self, ReadError> {
        let len = <Var<u32>>::read(cursor)?.0;
        let pos = cursor.position() as usize;
        let end = pos + len as usize;
        let bytes = &cursor
            .get_ref()
            .get(pos..end)
            .ok_or(ReadError::ReadPastEnd)?;
        cursor.set_position(end as u64);
        Ok(bytes)
    }
}

impl ProtocolWrite for &[u8] {
    fn write(self, writer: &mut impl std::io::Write) -> Result<(), WriteError> {
        Var(self.len() as u32).write(writer)?;
        Ok(writer.write_all(self)?)
    }

    fn size_hint() -> usize {
        1
    }
}

impl<'a> ProtocolRead<'a> for Cow<'a, [u8]> {
    fn read(cursor: &mut std::io::Cursor<&'a [u8]>) -> Result<Self, ReadError> {
        let len = <Var<u32>>::read(cursor)?.0;
        let pos = cursor.position() as usize;
        let end = pos + len as usize;
        let bytes = &cursor
            .get_ref()
            .get(pos..end)
            .ok_or(ReadError::ReadPastEnd)?;
        cursor.set_position(end as u64);
        Ok(Cow::Borrowed(bytes))
    }
}

impl ProtocolWrite for Cow<'_, [u8]> {
    fn write(self, writer: &mut impl std::io::Write) -> Result<(), WriteError> {
        Var(self.len() as u32).write(writer)?;
        Ok(writer.write_all(&self)?)
    }

    fn size_hint() -> usize {
        1
    }
}
