use protocol_derive::Protocol;
use std::borrow::Cow;
use std::io::Read;
use std::mem::{size_of, MaybeUninit};
use std::str::Utf8Error;
use std::string::FromUtf8Error;

#[derive(Protocol, Debug)]
pub struct Handshake0<'a> {
    #[varint]
    pub protocol_version: i32,
    // #[string(255)]
    pub server_address: Cow<'a, str>,
    pub server_port: u16,
    pub next_state: NextState0,
}

#[derive(Protocol, Debug)]
pub struct EntityAnimation0 {
    entity_id: i32,
    animation: Animation0,
}

#[derive(Protocol, Debug)]
#[varint]
pub enum Animation0 {
    None = 0,
    SwingArm,
    Damage,
    LeaveBed,
    EatFood,
    Crit,
    MagicCrit,
    Unknown = 102,
    Crouch,
    Uncrouch,
}

#[test]
fn testsss() {
    let cursor = &[2u8, 0, 0, 0, 1];
    let mut cursor = std::io::Cursor::new(&cursor[..]);
    let hs = Handshake0::read(&mut cursor);
    println!("{hs:?}");

    let hs = if let Ok(hs) = hs {
        hs
    } else {
        panic!("failed to parse")
    };

    let capacity = Handshake0::size_hint();
    eprintln!("allocating {} bytes for Handshake0", capacity);
    let mut buf = Vec::with_capacity(capacity);
    let res = Handshake0::write(hs, &mut std::io::Cursor::new(&mut buf));

    eprintln!("{:?}", buf);

    assert!(res.is_ok())
}

#[derive(Protocol, Debug)]
#[varint]
pub enum NextState0 {
    Status = 1,
    Login = 2,
}
impl TryFrom<i32> for NextState0 {
    type Error = InvalidEnumId;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Ok(match value {
            1 => Self::Status,
            2 => Self::Login,
            _ => Err(InvalidEnumId)?,
        })
    }
}

pub trait ProtocolRead<'read>: Sized {
    fn read(cursor: &'_ mut ::std::io::Cursor<&'read [u8]>) -> Result<Self, ReadError>;
}
pub trait ProtocolWrite {
    fn write(self, writer: &mut impl std::io::Write) -> Result<(), WriteError>;
    fn size_hint() -> usize;
}

pub enum WriteError {
    IoError(std::io::Error),
    StringTooLong,
}
impl From<std::io::Error> for WriteError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}

pub struct InvalidEnumId;
#[derive(Debug)]
#[non_exhaustive]
pub enum ReadError {
    IoError(std::io::Error),
    InvalidEnumId,
    Utf8Error(Utf8Error),
    FromUtf8Error(FromUtf8Error),
}
impl From<std::io::Error> for ReadError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}
impl From<InvalidEnumId> for ReadError {
    fn from(_: InvalidEnumId) -> Self {
        Self::InvalidEnumId
    }
}
impl From<Utf8Error> for ReadError {
    fn from(e: Utf8Error) -> Self {
        Self::Utf8Error(e)
    }
}
impl From<FromUtf8Error> for ReadError {
    fn from(e: FromUtf8Error) -> Self {
        Self::FromUtf8Error(e)
    }
}

macro_rules! impl_num {
    ($($num:ident),*) => {$(
        impl ProtocolRead<'_> for $num {
            fn read(cursor: &mut ::std::io::Cursor<&[u8]>) -> Result<Self, ReadError> {
                let mut buf: [u8; size_of::<$num>()] = unsafe { MaybeUninit::uninit().assume_init() };
                cursor.read_exact(&mut buf)?;
                Ok($num::from_be_bytes(buf))
            }
        }
        impl ProtocolWrite for $num {
            fn write(self, writer: &mut impl std::io::Write) -> Result<(), WriteError> {
                Ok(writer.write_all(&$num::to_be_bytes(self))?)
            }
            #[inline(always)]
            fn size_hint() -> usize {
                size_of::<$num>()
            }
        }
    )*};
}
impl_num! {
    u8, u16, u32, u64, u128,
    i8, i16, i32, i64, i128,
    f32, f64
}

macro_rules! impl_var_num {
    ($($num:ident, $unum:ident),*) => {$(
        impl ProtocolRead<'_> for Var<$num> {
            fn read(cursor: &mut ::std::io::Cursor<&[u8]>) -> Result<Self, ReadError> {
                let mut val = 0;
                let mut cur_val = [0];
                for i in 0..var_size::<{ $num::BITS }>() {
                    cursor.read_exact(&mut cur_val)?;
                    val += ((cur_val[0] & 0x7f) as $unum) << (i * 7);
                    if (cur_val[0] & 0x80) == 0x00 {
                        break;
                    }
                }
                Ok(Var(val as $num))
            }
        }
        impl ProtocolWrite for Var<$num> {
            fn write(self, writer: &mut impl std::io::Write) -> Result<(), WriteError> {
                let Var(mut int) = self;
                loop {
                    let next_val = (int as $unum >> 7) as $num;
                    if next_val == 0 {
                        writer.write_all(&[int as u8])?;
                        break;
                    }
                    writer.write_all(&[int as u8 | 0x80])?;
                    int = next_val;
                }
                Ok(())
            }
            #[inline(always)]
            fn size_hint() -> usize {
                1
            }
        }
        impl ProtocolRead<'_> for Var<$unum> {
            fn read(cursor: &mut ::std::io::Cursor<&[u8]>) -> Result<Self, ReadError> {
                let mut val = 0;
                let mut cur_val = [0];
                for i in 0..var_size::<{ $unum::BITS }>() {
                    cursor.read_exact(&mut cur_val)?;
                    val += ((cur_val[0] & 0x7f) as $unum) << (i * 7);
                    if (cur_val[0] & 0x80) == 0x00 {
                        break;
                    }
                }
                Ok(Var(val))
            }
        }
        impl ProtocolWrite for Var<$unum> {
            fn write(self, writer: &mut impl std::io::Write) -> Result<(), WriteError> {
                let Var(mut int) = self;
                loop {
                    let next_val = (int as $unum >> 7) as $unum;
                    if next_val == 0 {
                        writer.write_all(&[int as u8])?;
                        break;
                    }
                    writer.write_all(&[int as u8 | 0x80])?;
                    int = next_val;
                }
                Ok(())
            }
            #[inline(always)]
            fn size_hint() -> usize {
                1
            }
        }
    )*};
}
impl_var_num! {
    i8, u8,
    i16, u16,
    i32, u32,
    i64, u64,
    i128, u128
}

#[repr(transparent)]
struct Var<T>(pub T);

const fn var_size<const BITS: u32>() -> usize {
    (BITS as usize * 8 + 6) / 7
}

impl<'a> ProtocolRead<'a> for String {
    fn read(cursor: &mut ::std::io::Cursor<&[u8]>) -> Result<Self, ReadError> {
        let len = <Var<i32> as ProtocolRead>::read(cursor)?.0;
        let mut buf = unsafe { MaybeUninit::new(Vec::with_capacity(len as usize)).assume_init() };
        unsafe { buf.set_len(len as usize) };
        cursor.read_exact(&mut buf[..])?;
        Ok(String::from_utf8(buf)?)
    }
}
impl<'a> ProtocolRead<'a> for Cow<'a, str> {
    fn read(cursor: &mut ::std::io::Cursor<&'a [u8]>) -> Result<Self, ReadError> {
        let len = <Var<i32> as ProtocolRead>::read(cursor)?.0;
        let pos = cursor.position() as usize;
        let end = pos + len as usize;
        let s = std::str::from_utf8(&cursor.get_ref()[pos..end])?;
        cursor.set_position(end as u64);
        Ok(Cow::Borrowed(s))
    }
}
impl ProtocolWrite for String {
    fn write(self, writer: &mut impl std::io::Write) -> Result<(), WriteError> {
        let len = self
            .as_bytes()
            .len()
            .try_into()
            .map(Var)
            .map_err(|_| WriteError::StringTooLong)?;
        <Var<i32> as ProtocolWrite>::write(len, writer)?;
        writer.write_all(self.as_bytes())?;
        Ok(())
    }
    #[inline(always)]
    fn size_hint() -> usize {
        1
    }
}
impl<'a> ProtocolRead<'a> for &'a str {
    fn read(cursor: &mut std::io::Cursor<&'a [u8]>) -> Result<Self, ReadError> {
        let len = <Var<i32> as ProtocolRead>::read(cursor)?.0;
        let pos = cursor.position() as usize;
        let s = std::str::from_utf8(&cursor.get_ref()[pos..pos + len as usize])?;
        Ok(s)
    }
}
impl<'a> ProtocolWrite for Cow<'a, str> {
    fn write(self, writer: &mut impl std::io::Write) -> Result<(), WriteError> {
        let len = self
            .as_bytes()
            .len()
            .try_into()
            .map(Var)
            .map_err(|_| WriteError::StringTooLong)?;
        <Var<i32> as ProtocolWrite>::write(len, writer)?;
        writer.write_all(self.as_bytes())?;
        Ok(())
    }
    #[inline(always)]
    fn size_hint() -> usize {
        1
    }
}

impl ProtocolRead<'_> for bool {
    fn read(cursor: &mut ::std::io::Cursor<&[u8]>) -> Result<Self, ReadError> {
        let mut id = [0];
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
