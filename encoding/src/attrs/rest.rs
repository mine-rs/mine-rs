use crate::{decode, encode};
use crate::{Decode, Encode};
use std::io::Cursor;

#[repr(transparent)]
pub struct Rest<T: ?Sized>(T);

impl<T> From<T> for Rest<T> {
    fn from(v: T) -> Self {
        Self(v)
    }
}

impl<T: ?Sized> From<&T> for &Rest<T> {
    fn from(inner: &T) -> Self {
        // SAFETY: This is ok because Counted is #[repr(transparent)]
        unsafe { &*(inner as *const T as *const Rest<T>) }
    }
}

impl<T: ?Sized> AsRef<T> for Rest<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T: AsRef<[u8]>> Encode for Rest<T> {
    fn encode(&self, writer: &mut impl std::io::Write) -> encode::Result<()> {
        writer.write_all(self.0.as_ref()).map_err(From::from)
    }
}

impl<T> Rest<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<'dec, T: TryFrom<&'dec [u8]>> Decode<'dec> for Rest<T>
where
    decode::Error: From<<T as TryFrom<&'dec [u8]>>::Error>,
{
    fn decode(cursor: &mut Cursor<&'dec [u8]>) -> decode::Result<Self> {
        let pos = cursor.position() as usize;
        let slice = cursor
            .get_ref()
            .get(pos..)
            .ok_or(decode::Error::UnexpectedEndOfSlice)?;
        cursor.set_position(cursor.get_ref().len() as u64);
        Ok(Self(slice.try_into()?))
    }
}

/*
impl<'dec> Decode<'dec> for Rest<Cow<'dec, [u8]>> {
    fn decode(cursor: &mut Cursor<&'dec [u8]>) -> decode::Result<Self> {
        let pos = cursor.position() as usize;
        let slice = cursor
            .get_ref()
            .get(pos..)
            .ok_or(decode::Error::UnexpectedEndOfSlice)?;
        cursor.set_position(cursor.get_ref().len() as u64);
        Ok(Cow::Borrowed(slice).into())
    }
}

impl<'dec> Decode<'dec> for Rest<Cow<'dec, str>> {
    fn decode(cursor: &mut Cursor<&'dec [u8]>) -> decode::Result<Self> {
        let pos = cursor.position() as usize;
        let slice = cursor
            .get_ref()
            .get(pos..)
            .ok_or(decode::Error::UnexpectedEndOfSlice)?;
        cursor.set_position(cursor.get_ref().len() as u64);
        let str = std::str::from_utf8(slice)?;
        Ok(Cow::Borrowed(str).into())
    }
}

impl<'dec> Decode<'dec> for &Rest<[u8]> {
    fn decode(cursor: &mut Cursor<&'dec [u8]>) -> decode::Result<Self> {
        let pos = cursor.position() as usize;
        let slice = cursor
            .get_ref()
            .get(pos..)
            .ok_or(decode::Error::UnexpectedEndOfSlice)?;
        cursor.set_position(cursor.get_ref().len() as u64);
        Ok(slice.into())
    }
}

impl<'dec> Decode<'dec> for &Rest<str> {
    fn decode(cursor: &mut Cursor<&'dec [u8]>) -> decode::Result<Self> {
        let pos = cursor.position() as usize;
        let slice = cursor
            .get_ref()
            .get(pos..)
            .ok_or(decode::Error::UnexpectedEndOfSlice)?;
        cursor.set_position(cursor.get_ref().len() as u64);
        let str = std::str::from_utf8(slice)?;
        Ok(str.into())
    }
}

impl<'dec> Decode<'dec> for Rest<String> {
    fn decode(cursor: &mut Cursor<&'dec [u8]>) -> decode::Result<Self> {
        let str = <&Rest::<str>>::decode(cursor)?.as_ref();
        Ok(Rest::from(str.to_owned()))
    }
}
*/

#[test]
fn test() {
    let bytes = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let mut reader = Cursor::new(bytes.as_slice());
    #[allow(clippy::unwrap_used)]
    let new_bytes = Rest::<Vec<u8>>::decode(&mut reader).unwrap().into_inner();
    assert_eq!(bytes, new_bytes)
}
