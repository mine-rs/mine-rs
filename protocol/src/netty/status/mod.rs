use crate::*;

use protocol_derive::Protocol;

pub mod clientbound;
pub mod serverbound;

#[derive(Protocol)]
pub struct Ping0 {
    pub time: i64,
}
