pub mod block;
pub mod inv;

pub enum Error {}

pub type Result<T> = std::result::Result<T, Error>;

