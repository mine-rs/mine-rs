#![deny(clippy::unwrap_used, clippy::expect_used)]

pub mod attrs;
mod errors;
mod impls;
pub mod netty;

pub use errors::{ReadError, WriteError};

pub trait ProtocolRead<'read>: Sized {
    fn read(cursor: &'_ mut ::std::io::Cursor<&'read [u8]>) -> Result<Self, ReadError>;
}
pub trait ProtocolWrite {
    fn write(self, writer: &mut impl std::io::Write) -> Result<(), WriteError>;
    fn size_hint() -> usize;
}
