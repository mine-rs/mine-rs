use crate::ToStatic;

macro_rules! impl_num {
    ($($num:ident),*) => {$(
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
