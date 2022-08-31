use crate::*;

macro_rules! impl_num {
    ($($num:ident),*) => {$(
        impl Encode for $num {
            fn encode(&self, writer: &mut impl Write) -> encode::Result<()> {
                Ok(writer.write_all(&$num::to_be_bytes(*self))?)
            }
        }
    )*};
}
impl_num! {
    u8, u16, u32, u64, u128,
    i8, i16, i32, i64, i128,
    f32, f64
}
