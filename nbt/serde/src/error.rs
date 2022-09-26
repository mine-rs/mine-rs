use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("an error occurred while decoding: {0}")]
    Decode(#[from] miners_encoding::decode::Error),
    #[error("an error occurred while encoding: {0}")]
    Encode(#[from] miners_encoding::encode::Error),
    #[error("a serde error occurred: {0}")]
    Serde(String),
    #[error("no compound tag at root")]
    NonCompoundRoot,
}
impl serde::ser::Error for Error {
    fn custom<T>(msg:T) -> Self where T:std::fmt::Display {
        Self::Serde(msg.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;