use super::*;
use std::{borrow::Cow, marker::PhantomData};

pub struct Count<T, C> {
    pub inner: T,
    _marker: PhantomData<C>,
}

// generic Vec<T>

impl<'a, T, C> ProtocolRead<'a> for Count<Vec<T>, C>
where
    C: Into<usize>,
    C: ProtocolRead<'a>,
    T: ProtocolRead<'a>,
{
    fn read(cursor: &'_ mut std::io::Cursor<&'a [u8]>) -> Result<Self, ReadError> {
        let len: usize = <C as ProtocolRead>::read(cursor)?.into();
        let slice = (0..len)
            .map(|_| <T as ProtocolRead>::read(cursor))
            .collect::<Result<_, _>>()?;

        Ok(Count {
            inner: slice,
            _marker: PhantomData,
        })
    }
}
impl<T, C> ProtocolWrite for Count<Vec<T>, C>
where
    C: TryFrom<usize>,
    WriteError: From<<C as TryFrom<usize>>::Error>,
    C: ProtocolWrite,
    T: ProtocolWrite,
{
    fn write(self, writer: &mut impl std::io::Write) -> Result<(), WriteError> {
        C::write(self.inner.len().try_into()?, writer)?;
        for item in self.inner {
            item.write(writer)?;
        }
        Ok(())
    }

    fn size_hint() -> usize {
        C::size_hint()
    }
}

// &[u8]

impl<'a, C> ProtocolRead<'a> for Count<&'a [u8], C>
where
    C: Into<usize>,
    C: ProtocolRead<'a>,
{
    fn read(cursor: &'_ mut std::io::Cursor<&'a [u8]>) -> Result<Self, ReadError> {
        let len: usize = C::read(cursor)?.into();
        let pos = cursor.position() as usize;
        let end = pos + len as usize;
        let slice = &cursor
            .get_ref()
            .get(pos..end)
            .ok_or(ReadError::ReadPastEnd)?;
        cursor.set_position(end as u64);
        Ok(Count {
            inner: slice,
            _marker: PhantomData,
        })
    }
}
impl<'a, C> ProtocolWrite for Count<&'a [u8], C>
where
    C: TryFrom<usize>,
    WriteError: From<<C as TryFrom<usize>>::Error>,
    C: ProtocolWrite,
{
    fn write(self, writer: &mut impl std::io::Write) -> Result<(), WriteError> {
        let len: C = self.inner.len().try_into()?;
        C::write(len, writer)?;
        writer.write_all(self.inner)?;
        Ok(())
    }

    fn size_hint() -> usize {
        C::size_hint()
    }
}

// Cow<[u8]>

impl<'a, C> ProtocolRead<'a> for Count<Cow<'a, [u8]>, C>
where
    C: Into<usize>,
    C: ProtocolRead<'a>,
{
    fn read(cursor: &'_ mut std::io::Cursor<&'a [u8]>) -> Result<Self, ReadError> {
        let len: usize = C::read(cursor)?.into();
        let pos = cursor.position() as usize;
        let end = pos + len as usize;
        let slice = &cursor
            .get_ref()
            .get(pos..end)
            .ok_or(ReadError::ReadPastEnd)?;
        cursor.set_position(end as u64);
        Ok(Count {
            inner: Cow::Borrowed(slice),
            _marker: PhantomData,
        })
    }
}
impl<'a, C> ProtocolWrite for Count<Cow<'a, [u8]>, C>
where
    C: TryFrom<usize>,
    WriteError: From<<C as TryFrom<usize>>::Error>,
    C: ProtocolWrite,
{
    fn write(self, writer: &mut impl std::io::Write) -> Result<(), WriteError> {
        let len: C = self.inner.len().try_into()?;
        C::write(len, writer)?;
        writer.write_all(&self.inner)?;
        Ok(())
    }

    fn size_hint() -> usize {
        C::size_hint()
    }
}

// &str

impl<'a, C> ProtocolRead<'a> for Count<&'a str, C>
where
    C: Into<usize>,
    C: ProtocolRead<'a>,
{
    fn read(cursor: &mut ::std::io::Cursor<&'a [u8]>) -> Result<Self, ReadError> {
        let len: usize = C::read(cursor)?.into();
        let pos = cursor.position() as usize;
        let end = pos + len as usize;
        let slice = cursor
            .get_ref()
            .get(pos..end)
            .ok_or(ReadError::ReadPastEnd)?;
        let string = std::str::from_utf8(slice)?;
        cursor.set_position(end as u64);
        Ok(Count {
            inner: string,
            _marker: PhantomData,
        })
    }
}
impl<'a, C> ProtocolWrite for Count<&'a str, C>
where
    C: TryFrom<usize>,
    WriteError: From<<C as TryFrom<usize>>::Error>,
    C: ProtocolWrite,
{
    fn write(self, writer: &mut impl std::io::Write) -> Result<(), WriteError> {
        let slice = self.inner.as_bytes();
        let len: C = slice.len().try_into()?;
        C::write(len, writer)?;
        writer.write_all(slice)?;
        Ok(())
    }

    fn size_hint() -> usize {
        C::size_hint()
    }
}

// Cow<str>

impl<'a, C> ProtocolRead<'a> for Count<Cow<'a, str>, C>
where
    C: Into<usize>,
    C: ProtocolRead<'a>,
{
    fn read(cursor: &mut ::std::io::Cursor<&'a [u8]>) -> Result<Self, ReadError> {
        let len: usize = C::read(cursor)?.into();
        let pos = cursor.position() as usize;
        let end = pos + len as usize;
        let slice = cursor
            .get_ref()
            .get(pos..end)
            .ok_or(ReadError::ReadPastEnd)?;
        let string = std::str::from_utf8(slice)?;
        cursor.set_position(end as u64);
        Ok(Count {
            inner: Cow::Borrowed(string),
            _marker: PhantomData,
        })
    }
}
impl<'a, C> ProtocolWrite for Count<Cow<'a, str>, C>
where
    C: TryFrom<usize>,
    WriteError: From<<C as TryFrom<usize>>::Error>,
    C: ProtocolWrite,
{
    fn write(self, writer: &mut impl std::io::Write) -> Result<(), WriteError> {
        let slice = self.inner.as_bytes();
        let len: C = slice.len().try_into()?;
        C::write(len, writer)?;
        writer.write_all(slice)?;
        Ok(())
    }

    fn size_hint() -> usize {
        C::size_hint()
    }
}
