use std::num::{
    NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize, NonZeroU128,
    NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize,
};

use crate::ToStatic;

macro_rules! nonzero_impl {
    ($($nz:ident $(;)?)*) => {
        $(impl ToStatic for $nz {
            type Static = $nz;
            fn to_static(&self) -> Self::Static {
                *self
            }
            fn into_static(self) -> Self::Static {
                self
            }
        })*
    };
}

nonzero_impl!(
    NonZeroU8 NonZeroI8;
    NonZeroU16 NonZeroI16;
    NonZeroU32 NonZeroI32;
    NonZeroU64 NonZeroI64;
    NonZeroU128 NonZeroI128;
    NonZeroUsize NonZeroIsize;
);
