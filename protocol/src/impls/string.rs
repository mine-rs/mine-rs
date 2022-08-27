use crate::{attrs::Var, *};
use std::{borrow::Cow, io::Read};

// &str

impl<'a> ProtocolRead<'a> for &'a str {
    fn read(cursor: &mut std::io::Cursor<&'a [u8]>) -> Result<Self, ReadError> {
        let len = <Var<i32> as ProtocolRead>::read(cursor)?.0;
        let pos = cursor.position() as usize;
        let end = pos + len as usize;
        let slice = cursor
            .get_ref()
            .get(pos..end)
            .ok_or(ReadError::ReadPastEnd)?;
        let s = std::str::from_utf8(slice)?;
        cursor.set_position(end as u64);
        Ok(s)
    }
}
impl<'a> ProtocolWrite for &'a str {
    fn write(self, writer: &mut impl std::io::Write) -> Result<(), WriteError> {
        let len = self.as_bytes().len();
        let var_len = len
            .try_into()
            .map(Var)
            .map_err(|_| WriteError::StringTooLong)?;
        <Var<i32> as ProtocolWrite>::write(var_len, writer)?;
        writer.write_all(self.as_bytes())?;
        Ok(())
    }
    #[inline(always)]
    fn size_hint() -> usize {
        1
    }
}

// Cow<str>

impl<'a> ProtocolRead<'a> for Cow<'a, str> {
    fn read(cursor: &mut ::std::io::Cursor<&'a [u8]>) -> Result<Self, ReadError> {
        let len = <Var<i32> as ProtocolRead>::read(cursor)?.0;
        let pos = cursor.position() as usize;
        let end = pos + len as usize;
        let slice = cursor
            .get_ref()
            .get(pos..end)
            .ok_or(ReadError::ReadPastEnd)?;
        let s = std::str::from_utf8(slice)?;
        cursor.set_position(end as u64);
        Ok(Cow::Borrowed(s))
    }
}
impl<'a> ProtocolWrite for Cow<'a, str> {
    fn write(self, writer: &mut impl std::io::Write) -> Result<(), WriteError> {
        let len = self.as_bytes().len();
        let var_len = len
            .try_into()
            .map(Var)
            .map_err(|_| WriteError::StringTooLong)?;
        <Var<i32> as ProtocolWrite>::write(var_len, writer)?;
        writer.write_all(self.as_bytes())?;
        Ok(())
    }
    #[inline(always)]
    fn size_hint() -> usize {
        1
    }
}

// String

impl<'a> ProtocolRead<'a> for String {
    fn read(cursor: &mut ::std::io::Cursor<&[u8]>) -> Result<Self, ReadError> {
        let len = <Var<i32>>::read(cursor)?.0;
        let mut buf = vec![0u8; len as usize];
        cursor.read_exact(&mut buf[..])?;
        Ok(String::from_utf8(buf)?)
    }
}
impl ProtocolWrite for String {
    fn write(self, writer: &mut impl std::io::Write) -> Result<(), WriteError> {
        let len = self.as_bytes().len();
        let var_len = len
            .try_into()
            .map(Var)
            .map_err(|_| WriteError::StringTooLong)?;
        <Var<i32>>::write(var_len, writer)?;
        writer.write_all(self.as_bytes())?;
        Ok(())
    }
    #[inline(always)]
    fn size_hint() -> usize {
        1
    }
}
