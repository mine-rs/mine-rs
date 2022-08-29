use super::*;
use std::io::Read;

#[repr(transparent)]
pub struct Var<T>(pub T);

const fn var_size<const BITS: u32>() -> usize {
    (BITS as usize * 8 + 6) / 7
}

macro_rules! impl_var_num {
    ($num:ident $unum:ident) => {
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
    };
    ($($num:ident, $unum:ident),*) => {$(
        impl_var_num!{$num $unum}
        impl_var_num!{$unum $unum}
    )*};
}
impl_var_num! {
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
                let read_res = Var::<$typ>::read(&mut cursor);
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

                let write_res = Var(*num as $typ).write(&mut writebuf);
                let leftover = writebuf.len();

                assert!(write_res.is_ok(), "tried to write more data than it should");
                assert_eq!(*res, &buf[0..$len - leftover], "buffers weren't equal")
            }
        }
    )*};
}
#[cfg(test)]
mod varint {
    use std::io::Cursor;

    use super::*;
    use crate::*;
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
    use std::io::Cursor;

    use super::*;
    use crate::*;
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
