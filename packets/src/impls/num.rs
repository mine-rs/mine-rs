use crate::*;
use core::mem::size_of;
use std::io::Read;

macro_rules! impl_num {
    ($($num:ident),*) => {$(
        impl ProtocolRead<'_> for $num {
            fn read(cursor: &mut ::std::io::Cursor<&[u8]>) -> Result<Self, ReadError> {
                let mut buf = [0u8; size_of::<$num>()];
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
        impl ToStatic for $num {
            type Static = $num;
            fn to_static(&self) -> Self::Static {
                *self
            }
            fn into_static(self) -> Self::Static {
                self
            }
        }
    )*};
}
impl_num! {
    u8, u16, u32, u64, u128,
    i8, i16, i32, i64, i128,
    f32, f64
}
