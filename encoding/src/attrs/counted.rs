use crate::*;
use std::borrow::Cow;
use std::marker::PhantomData;

#[repr(transparent)]
pub struct Counted<T: ?Sized, C> {
    _marker: PhantomData<C>,
    pub inner: T,
}

impl<T, C> From<T> for Counted<T, C> {
    fn from(inner: T) -> Self {
        Self {
            inner,
            _marker: PhantomData,
        }
    }
}

impl<T: ?Sized, C> From<&T> for &Counted<T, C> {
    fn from(inner: &T) -> Self {
        // SAFETY: This is ok because Counted is #[repr(transparent)]
        unsafe { std::mem::transmute(inner) }
    }
}

impl<T: ?Sized, C> AsRef<T> for Counted<T, C> {
    fn as_ref(&self) -> &T {
        &self.inner
    }
}

impl<T: Sized, C> Counted<T, C> {
    pub fn into_inner(self) -> T {
        self.inner
    }
}

macro_rules! impl_count {($($num:ident),*) => {$(
    impl<'dec, T> Decode<'dec> for Counted<Vec<T>, $num>
    where
        T: Decode<'dec>,
    {
        fn decode(cursor: &mut Cursor<&'dec [u8]>) -> decode::Result<Self> {
            let count = $num::decode(cursor)?;
            let vec: Result<Vec<T>, _> = (0..count).map(|_| T::decode(cursor)).collect();
            Ok(Counted::from(vec?))
        }
    }
    impl<T> Encode for Counted<Vec<T>, $num>
    where
        T: Encode,
    {
        fn encode(&self, writer: &mut impl Write) -> encode::Result<()> {
            let vec = self.as_ref();
            $num::try_from(vec.len())?.encode(writer)?;
            vec.iter().try_for_each(|item| item.encode(writer))
        }
    }

    // cow

    // str

    impl<'dec> Decode<'dec> for Counted<Cow<'dec, str>, $num> {
        fn decode(cursor: &mut Cursor<&'dec [u8]>) -> decode::Result<Self> {
            let slice = &<&Counted<[u8], $num>>::decode(cursor)?.inner;
            let str = std::str::from_utf8(slice)?;
            Ok(Cow::Borrowed(str).into())
        }
    }
    impl<'enc> Encode for Counted<Cow<'enc, str>, $num> {
        fn encode(&self, writer: &mut impl Write) -> encode::Result<()> {
            let str = self.as_ref().as_ref();
            let slice = str.as_bytes();
            $num::try_from(slice.len())?.encode(writer)?;
            writer.write_all(slice)?;
            Ok(())
        }
    }

    // [u8]

    impl<'dec> Decode<'dec> for Counted<Cow<'dec, [u8]>, $num> {
        fn decode(cursor: &mut Cursor<&'dec [u8]>) -> decode::Result<Self> {
            let slice = &<&Counted<[u8], $num>>::decode(cursor)?.inner;
            Ok(Cow::Borrowed(slice).into())
        }
    }
    impl<'enc> Encode for Counted<Cow<'enc, [u8]>, $num> {
        fn encode(&self, writer: &mut impl Write) -> encode::Result<()> {
            let cow = self.as_ref();
            $num::try_from(cow.len())?.encode(writer)?;
            writer.write_all(cow)?;
            Ok(())
        }
    }

    // [f32]

    impl<'dec> Decode<'dec> for Counted<Cow<'dec, [f32]>, $num> {
        fn decode(cursor: &mut Cursor<&'dec [u8]>) -> decode::Result<Self> {
            let slice = &<&Counted<[f32], $num>>::decode(cursor)?.inner;
            Ok(Cow::Borrowed(slice).into())
        }
    }
    impl<'enc> Encode for Counted<Cow<'enc, [f32]>, $num> {
        fn encode(&self, writer: &mut impl Write) -> encode::Result<()> {
            let cow = self.as_ref();
            <&Counted<[f32], $num>>::from(cow.as_ref()).encode(writer)
        }
    }

    // [f64]

    impl<'dec> Decode<'dec> for Counted<Cow<'dec, [f64]>, $num> {
        fn decode(cursor: &mut Cursor<&'dec [u8]>) -> decode::Result<Self> {
            let slice = &<&Counted<[f64], $num>>::decode(cursor)?.inner;
            Ok(Cow::Borrowed(slice).into())
        }
    }
    impl<'enc> Encode for Counted<Cow<'enc, [f64]>, $num> {
        fn encode(&self, writer: &mut impl Write) -> encode::Result<()> {
            let cow = self.as_ref();
            <&Counted<[f64], $num>>::from(cow.as_ref()).encode(writer)
        }
    }

    // slices

    // str

    impl<'dec> Decode<'dec> for &Counted<str, $num> {
        fn decode(cursor: &mut Cursor<&'dec [u8]>) -> decode::Result<Self> {
            let slice = &<&Counted<[u8], $num>>::decode(cursor)?.inner;
            let str = std::str::from_utf8(slice)?;
            Ok(str.into())
        }
    }
    impl Encode for Counted<str, $num> {
        fn encode(&self, writer: &mut impl Write) -> encode::Result<()> {
            let str = self.as_ref();
            let slice = str.as_bytes();
            $num::try_from(slice.len())?.encode(writer)?;
            writer.write_all(slice)?;
            Ok(())
        }
    }

    // [u8]

    impl<'dec> Decode<'dec> for &Counted<[u8], $num> {
        fn decode(cursor: &mut Cursor<&'dec [u8]>) -> decode::Result<Self> {
            let count = $num::decode(cursor)?;
            let pos = cursor.position() as usize;
            let slice = cursor
                .get_ref()
                .get(pos..pos + count as usize)
                .ok_or(decode::Error::UnexpectedEndOfSlice)?;
            cursor.set_position(pos as u64 + count as u64);
            Ok(slice.into())
        }
    }

    impl Encode for Counted<[u8], $num> {
        fn encode(&self, writer: &mut impl Write) -> encode::Result<()> {
            let slice = self.as_ref();
            $num::try_from(slice.len())?.encode(writer)?;
            writer.write_all(slice)?;
            Ok(())
        }
    }

    // [f32]

    impl<'dec> Decode<'dec> for &Counted<[f32], $num> {
        fn decode(cursor: &mut Cursor<&'dec [u8]>) -> decode::Result<Self> {
            let len = $num::decode(cursor)? as usize * core::mem::size_of::<f32>();
            let pos = cursor.position();
            let data = cursor
                .get_ref()
                .get(pos as usize + len)
                .ok_or(decode::Error::UnexpectedEndOfSlice)? as *const u8
                as *const f32;
            let slice = unsafe { std::slice::from_raw_parts(data, len) };
            cursor.set_position(pos + len as u64);
            Ok(slice.into())
        }
    }
    impl Encode for &Counted<[f32], $num> {
        fn encode(&self, writer: &mut impl Write) -> encode::Result<()> {
            let slice = self.as_ref();
            $num::try_from(slice.len())?.encode(writer)?;
            writer.write_all(unsafe {
                let data = slice.as_ptr() as *const u8;
                let len = slice.len() * core::mem::size_of::<f32>();
                std::slice::from_raw_parts(data, len)
            })?;
            Ok(())
        }
    }

    // [f64]

    impl<'dec> Decode<'dec> for &Counted<[f64], $num> {
        fn decode(cursor: &mut Cursor<&'dec [u8]>) -> decode::Result<Self> {
            let len = $num::decode(cursor)? as usize * core::mem::size_of::<f64>();
            let pos = cursor.position();
            let data = cursor
                .get_ref()
                .get(pos as usize + len)
                .ok_or(decode::Error::UnexpectedEndOfSlice)? as *const u8
                as *const f64;
            let slice = unsafe { std::slice::from_raw_parts(data, len) };
            cursor.set_position(pos + len as u64);
            Ok(slice.into())
        }
    }
    impl Encode for &Counted<[f64], $num> {
        fn encode(&self, writer: &mut impl Write) -> encode::Result<()> {
            let slice = self.as_ref();
            $num::try_from(slice.len())?.encode(writer)?;
            writer.write_all(unsafe {
                let data = slice.as_ptr() as *const u8;
                let len = slice.len() * core::mem::size_of::<f64>();
                std::slice::from_raw_parts(data, len)
            })?;
            Ok(())
        }
    }

    impl<'dec> Decode<'dec> for Counted<String, $num> {
        fn decode(cursor: &mut Cursor<&'dec [u8]>) -> decode::Result<Self> {
            let str = <&Counted::<str, $num>>::decode(cursor)?.as_ref();
            Ok(Counted::from(str.to_owned()))
        }
    }
    impl Encode for Counted<String, $num> {
        fn encode(&self, writer: &mut impl Write) -> encode::Result<()> {
            let string = self.as_ref();
            let slice = string.as_bytes();
            $num::try_from(slice.len())?.encode(writer)?;
            writer.write_all(slice)?;
            Ok(())
        }
    }
)*};}

impl_count! {
    u8, u16, u32, u64, u128,
    i8, i16, i32, i64, i128
}

// TODO: add tests
