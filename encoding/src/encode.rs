use crate::*;

mod bool;
mod cow;
mod num;
mod option;
mod slice;
mod str;
mod string;
mod uuid;
mod vec;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    IntError(#[from] std::num::TryFromIntError),
    #[error("{0}")]
    Custom(&'static str),
    // StringTooLong,
    // InvalidCount,
}

pub trait Encode {
    fn encode(&self, writer: &mut impl Write) -> Result<()>;
}
