pub mod clientbound;
pub mod serverbound;
use crate::netty::{ProtocolRead, ProtocolWrite, ReadError, WriteError};
use protocol_derive::Protocol;

#[derive(Protocol)]
pub struct Ping0 {
    pub time: i64,
}
