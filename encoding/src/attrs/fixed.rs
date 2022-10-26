use crate::*;
use std::marker::PhantomData;

/// Minecraft's Fixed Point Numbers
///
/// https://wiki.vg/Protocol#Fixed-point_numbers
pub struct Fixed<const PRECISION: u8, Protocol, Logical> {
    pub(crate) logical: Logical,
    _marker: PhantomData<Protocol>,
}

impl<const PRECISION: u8, P, L> From<&L> for Fixed<PRECISION, P, L>
where
    L: Copy,
{
    fn from(logical: &L) -> Self {
        Self {
            logical: *logical,
            _marker: PhantomData,
        }
    }
}

impl<const PRECISION: u8, P, L> From<L> for Fixed<PRECISION, P, L> {
    fn from(logical: L) -> Self {
        Self {
            logical,
            _marker: PhantomData,
        }
    }
}

impl<const PRECISION: u8, P, L> Fixed<PRECISION, P, L> {
    pub fn into_inner(self) -> L {
        self.logical
    }
}

macro_rules! impl_fixed {
    ($([$($Protocol:ident),*]: $Logical:ident),+) => {
        $($(
            impl<'dec, const PRECISION: u8> Decode<'dec> for Fixed<PRECISION, $Protocol, $Logical> {
                fn decode(cursor: &mut Cursor<&'dec [u8]>) -> decode::Result<Self> {
                    let logical = $Protocol::decode(cursor)? as $Logical / (1 << (PRECISION as usize)) as $Logical;
                    Ok(Self { logical, _marker: PhantomData })
                }
            }
            impl<const PRECISION: u8> Encode for Fixed<PRECISION, $Protocol, $Logical> {
                fn encode(&self, writer: &mut impl Write) -> encode::Result<()> {
                    let Self { logical, .. } = self;
                    ((logical * (1 << (PRECISION as usize)) as $Logical) as $Protocol).encode(writer)
                }
            }
        )*)*
    };
}

impl_fixed! {
    [i8, u8, i16, u16]: f32,
    [i8, u8, i16, u16, i32, u32]: f64
}

// TODO: add tests
