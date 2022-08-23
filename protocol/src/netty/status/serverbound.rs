use crate::netty::{ProtocolRead, ProtocolWrite, ReadError, WriteError};
use protocol_derive::Protocol;

#[derive(Protocol)]
pub struct Request0 {}

pub use super::Ping0;
