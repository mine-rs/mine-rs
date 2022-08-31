use crate::*;

macro_rules! impl_num {
    ($($num:ident),*) => {$(
        impl<'dec> Decode<'dec> for $num {
            fn decode(cursor: &mut Cursor<&'dec [u8]>) -> decode::Result<Self> {
                let mut buf = [0u8; core::mem::size_of::<$num>()];
                cursor.read_exact(&mut buf)?;
                Ok($num::from_be_bytes(buf))
            }
        }
    )*};
}
impl_num! {
    u8, u16, u32, u64, u128,
    i8, i16, i32, i64, i128,
    f32, f64
}
