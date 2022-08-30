use super::*;
use std::{borrow::Cow, marker::PhantomData};

pub struct Count<T, C> {
    pub inner: T,
    _marker: PhantomData<C>,
}

impl<T, C> Count<T, C> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            _marker: PhantomData,
        }
    }
}

macro_rules! impl_count {
    ($($num:ident),*) => {$(
        // generic Vec<T>

        impl<'a, T> ProtocolRead<'a> for Count<Vec<T>, $num>
        where
            T: ProtocolRead<'a>,
        {
            fn read(cursor: &mut std::io::Cursor<&'a [u8]>) -> Result<Self, ReadError> {
                let len = $num::read(cursor)? as usize;
                let slice = (0..len)
                    .map(|_| <T as ProtocolRead>::read(cursor))
                    .collect::<Result<_, _>>()?;

                Ok(Count {
                    inner: slice,
                    _marker: PhantomData,
                })
            }
        }
        impl<T> ProtocolWrite for Count<Vec<T>, $num>
        where
            T: ProtocolWrite,
        {
            fn write(self, writer: &mut impl std::io::Write) -> Result<(), WriteError> {
                $num::write(self.inner.len().try_into()?, writer)?;
                for item in self.inner {
                    item.write(writer)?;
                }
                Ok(())
            }

            fn size_hint() -> usize {
                $num::size_hint()
            }
        }

        // &[u8]

        impl<'a> ProtocolRead<'a> for Count<&'a [u8], $num> {
            fn read(cursor: &mut std::io::Cursor<&'a [u8]>) -> Result<Self, ReadError> {
                let len = $num::read(cursor)? as usize;
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
        impl<'a> ProtocolWrite for Count<&'a [u8], $num> {
            fn write(self, writer: &mut impl std::io::Write) -> Result<(), WriteError> {
                let len: $num = self.inner.len().try_into()?;
                $num::write(len, writer)?;
                writer.write_all(self.inner)?;
                Ok(())
            }

            fn size_hint() -> usize {
                $num::size_hint()
            }
        }

        // Cow<[u8]>

        impl<'a> ProtocolRead<'a> for Count<Cow<'a, [u8]>, $num> {
            fn read(cursor: &'_ mut std::io::Cursor<&'a [u8]>) -> Result<Self, ReadError> {
                let len = $num::read(cursor)? as usize;
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
        impl<'a> ProtocolWrite for Count<Cow<'a, [u8]>, $num> {
            fn write(self, writer: &mut impl std::io::Write) -> Result<(), WriteError> {
                let len: $num = self.inner.len().try_into()?;
                $num::write(len, writer)?;
                writer.write_all(&self.inner)?;
                Ok(())
            }

            fn size_hint() -> usize {
                $num::size_hint()
            }
        }

        // &str

        impl<'a> ProtocolRead<'a> for Count<&'a str, $num> {
            fn read(cursor: &mut ::std::io::Cursor<&'a [u8]>) -> Result<Self, ReadError> {
                let len = $num::read(cursor)? as usize;
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
        impl<'a> ProtocolWrite for Count<&'a str, $num> {
            fn write(self, writer: &mut impl std::io::Write) -> Result<(), WriteError> {
                let slice = self.inner.as_bytes();
                let len: $num = slice.len().try_into()?;
                $num::write(len, writer)?;
                writer.write_all(slice)?;
                Ok(())
            }

            fn size_hint() -> usize {
                $num::size_hint()
            }
        }

        // Cow<str>

        impl<'a> ProtocolRead<'a> for Count<Cow<'a, str>, $num> {
            fn read(cursor: &mut ::std::io::Cursor<&'a [u8]>) -> Result<Self, ReadError> {
                let len = $num::read(cursor)? as usize;
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
        impl<'a> ProtocolWrite for Count<Cow<'a, str>, $num> {
            fn write(self, writer: &mut impl std::io::Write) -> Result<(), WriteError> {
                let slice = self.inner.as_bytes();
                let len: $num = slice.len().try_into()?;
                $num::write(len, writer)?;
                writer.write_all(slice)?;
                Ok(())
            }

            fn size_hint() -> usize {
                $num::size_hint()
            }
        }
    )*};
}

impl_count! {
    u8, u16, u32, u64, u128,
    i8, i16, i32, i64, i128
}
