use std::borrow::Cow;

use crate::*;

/// Modified UTF-8
///
/// a modified version of the utf-8 standard by java,
/// used in miners-protocol in form of nbt, can also be
/// found in java classfiles
///
/// mutf8-strings are length-prefixed in form of a u16
/// some characters like "\0" are represented in form of
/// multi-byte sequences
#[repr(transparent)]
pub struct Mutf8<T: ?Sized>(T);

impl<T: ?Sized> Mutf8<T> {
    pub fn from(t: &T) -> &Self {
        // SAFETY: this is safe because Mutf8 is #[repr(transparent)]
        unsafe { std::mem::transmute(t) }
    }
}
impl<T> Mutf8<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl Encode for Mutf8<str> {
    fn encode(&self, writer: &mut impl Write) -> encode::Result<()> {
        to_mutf8(&self.0, writer)
    }
}

impl<'dec: 'a, 'a> Decode<'dec> for Mutf8<Cow<'a, str>> {
    fn decode(cursor: &mut Cursor<&'dec [u8]>) -> decode::Result<Self> {
        let string = from_mutf8(cursor)?;
        Ok(Mutf8(string))
    }
}
impl<'a> Encode for Mutf8<Cow<'a, str>> {
    fn encode(&self, writer: &mut impl Write) -> encode::Result<()> {
        to_mutf8(&self.0, writer)
    }
}

impl<'dec> Decode<'dec> for Mutf8<String> {
    fn decode(cursor: &mut Cursor<&'dec [u8]>) -> decode::Result<Self> {
        let string = from_mutf8(cursor)?;
        Ok(Mutf8(string.to_string()))
    }
}
impl Encode for Mutf8<String> {
    fn encode(&self, writer: &mut impl Write) -> encode::Result<()> {
        to_mutf8(&self.0, writer)
    }
}

fn from_mutf8<'dec>(cursor: &mut Cursor<&'dec [u8]>) -> decode::Result<Cow<'dec, str>> {
    let len = u16::decode(cursor)?;
    let pos = cursor.position();
    let slice = &cursor.get_ref()[pos as usize..pos as usize + len as usize];
    let string = mutf8::decode(slice)?;
    cursor.set_position(pos + len as u64);
    Ok(string)
}
fn to_mutf8(data: &str, writer: &mut impl Write) -> encode::Result<()> {
    let mutf8 = mutf8::encode(data);
    (mutf8.len() as u16).encode(writer)?;
    writer.write_all(&mutf8)?;
    Ok(())
}

// TODO: add tests
