use crate::*;

mod bool;
mod cow;
mod num;
mod slice;
mod str;
mod string;
mod vec;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    IntError(#[from] std::num::TryFromIntError),
    // StringTooLong,
    // InvalidCount,
}

pub trait Encode {
    fn encode(&self, writer: &mut impl Write) -> Result<()>;
}
