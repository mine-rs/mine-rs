use crate::*;
use std::io::{Read, Write};
use Cursor;

/// Minecraft's VarInt
///
/// https://wiki.vg/VarInt_And_VarLong
#[repr(transparent)]
pub struct Var<T>(pub(crate) T);

impl<T> From<T> for Var<T> {
    fn from(t: T) -> Self {
        Var(t)
    }
}

impl<T> Var<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

/// given the number of bits this function returns
/// how many bytes a Var will take up
pub const fn var_size(bits: u32) -> usize {
    (bits as usize * 8 + 6) / 7
}

macro_rules! impl_var {
    (@impl $num:ident $unum:ident) => {
        impl<'dec> Decode<'dec> for Var<$num> {
            fn decode(cursor: &mut Cursor<&'dec [u8]>) -> decode::Result<Self> {
                let mut value = 0;
                let mut current_byte = [0];
                for i in 0..var_size($num::BITS) {
                    cursor.read_exact(&mut current_byte)?;
                    value += ((current_byte[0] & 0x7f) as $unum) << (i * 7);
                    if (current_byte[0] & 0x80) == 0x00 {
                        break;
                    }
                }
                Ok(Var(value as $num))
            }
        }
        impl Encode for Var<$num> {
            fn encode(&self, writer: &mut impl Write) -> encode::Result<()> {
                let Var(mut value) = self;
                loop {
                    let next_byte = (value as $unum >> 7) as $num;
                    if next_byte == 0 {
                        writer.write_all(&[value as u8])?;
                        break;
                    }
                    writer.write_all(&[value as u8 | 0x80])?;
                    value = next_byte;
                }
                Ok(())
            }
        }
    };
    ($($inum:ident, $unum:ident),*) => {$(
        impl_var!(@impl $inum $unum);
        impl_var!(@impl $unum $unum);
    )*};
}

impl_var! {
    i8, u8,
    i16, u16,
    i32, u32,
    i64, u64,
    i128, u128
}

#[cfg(test)]
macro_rules! tests {
    ($len:literal; $($read:ident $write:ident $typ:ident;)*) => {$(
        #[test]
        fn $read() {
            for (num, res) in TESTS {
                let mut cursor = Cursor::new(*res);
                let read_res = Var::<$typ>::decode(&mut cursor);
                assert_eq!(
                    cursor.position() as usize,
                    res.len(),
                    "did not read all of the data"
                );
                match read_res {
                    Ok(Var(res)) => assert_eq!(*num as $typ, res, "numbers weren't equal"),
                    Err(_) => panic!("failed parsing"),
                }
            }
        }
        #[test]
        fn $write() {
            for (num, res) in TESTS {
                let mut buf = [0u8; $len];
                let mut writebuf = &mut buf[..];

                let write_res = Var(*num as $typ).encode(&mut writebuf);
                let leftover = writebuf.len();

                assert!(write_res.is_ok(), "tried to write more data than it should");
                assert_eq!(*res, &buf[0..$len - leftover], "buffers weren't equal")
            }
        }
    )*};
}

#[cfg(test)]
mod varint {
    use super::*;

    const TESTS: &[(i32, &[u8])] = &[
        (0, &[0x00]),
        (1, &[0x01]),
        (127, &[0x7f]),
        (128, &[0x80, 1]),
        (255, &[0xff, 0x01]),
        (25565, &[0xdd, 0xc7, 0x01]),
        (2097151, &[0xff, 0xff, 0x7f]),
        (2147483647, &[0xff, 0xff, 0xff, 0xff, 0x07]),
        (-1, &[0xff, 0xff, 0xff, 0xff, 0x0f]),
        (-2147483648, &[0x80, 0x80, 0x80, 0x80, 0x08]),
    ];
    tests! {5;
        read_i32 write_i32 i32;
        read_u32 write_u32 u32;
    }
}

#[cfg(test)]
mod varlong {
    use super::*;

    const TESTS: &[(i64, &[u8])] = &[
        (0, &[0x00]),
        (1, &[0x01]),
        (127, &[0x7f]),
        (128, &[0x80, 1]),
        (255, &[0xff, 0x01]),
        (25565, &[0xdd, 0xc7, 0x01]),
        (2097151, &[0xff, 0xff, 0x7f]),
        (2147483647, &[0xff, 0xff, 0xff, 0xff, 0x07]),
        (
            9223372036854775807,
            &[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f],
        ),
        (
            -1,
            &[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01],
        ),
        (
            -2147483648,
            &[0x80, 0x80, 0x80, 0x80, 0xf8, 0xff, 0xff, 0xff, 0xff, 0x01],
        ),
        (
            -9223372036854775808,
            &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
        ),
    ];
    tests! {10;
        read_i64 write_i64 i64;
        read_u64 write_u64 u64;
    }
}
