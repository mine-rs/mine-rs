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
