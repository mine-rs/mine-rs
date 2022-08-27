use super::*;
use std::marker::PhantomData;

pub struct Fixed<const PRECISION: u8, Protocol, Logical> {
    pub data: Logical,
    _marker: PhantomData<Protocol>,
}
impl<const PRECISION: u8, A, B> Fixed<PRECISION, A, B> {
    #[allow(clippy::self_named_constructors)]
    pub fn fixed(b: B) -> Self {
        Self {
            data: b,
            _marker: PhantomData,
        }
    }
}

macro_rules! fixed {
    ($([$($prot:ident),*]: $logical:ident),+) => {
        $($(
            impl<const PRECISION: u8> ProtocolRead<'_> for Fixed<PRECISION, $prot, $logical> {
                fn read(cursor: &'_ mut std::io::Cursor<&[u8]>) -> Result<Self, ReadError> {
                    let data = $prot::read(cursor)? as $logical / (1 << (PRECISION as usize)) as $logical;
                    Ok(Self {
                        data,
                        _marker: PhantomData,
                    })
                }
            }
            impl<const PRECISION: u8>  ProtocolWrite for Fixed<PRECISION, $prot, $logical> {
                fn write(self, writer: &mut impl std::io::Write) -> Result<(), WriteError> {
                    let Self {
                        data,
                        _marker: PhantomData,
                    } = self;
                    ((data * (1 << (PRECISION as usize)) as $logical) as $prot).write(writer)
                }
                fn size_hint() -> usize {
                    $prot::size_hint()
                }
            }
            impl<const PRECISION: u8> ProtocolRead<'_> for Fixed<PRECISION, Var<$prot>, $logical> {
                fn read(cursor: &'_ mut std::io::Cursor<&[u8]>) -> Result<Self, ReadError> {
                    let data = <Var<$prot>>::read(cursor)?.0 as $logical / (1 << (PRECISION as usize)) as $logical;
                    Ok(Self {
                        data,
                        _marker: PhantomData,
                    })
                }
            }
            impl<const PRECISION: u8>  ProtocolWrite for Fixed<PRECISION, Var<$prot>, $logical> {
                fn write(self, writer: &mut impl std::io::Write) -> Result<(), WriteError> {
                    let Self {
                        data,
                        _marker: PhantomData,
                    } = self;
                    Var((data * (1 << (PRECISION as usize)) as $logical) as $prot).write(writer)
                }
                fn size_hint() -> usize {
                    <Var<$prot> as ProtocolWrite>::size_hint()
                }
            }
        )*)*
    };
}

fixed! {
    [i8, u8, i16, u16]: f32,
    [i8, u8, i16, u16, i32, u32]: f64
}
