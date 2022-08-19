use std::borrow::Cow;

use crate::netty::{InvalidEnumId, ProtocolRead, ProtocolWrite, ReadError, Var, WriteError};
use protocol_derive::Protocol;

#[derive(Protocol)]
/// this packet as the first one is actually pretty controversial,
/// for 13w41a protocol_version was actually varint, starting 13w42a
/// it is now ushort
struct Handshake0<'a> {
    #[varint]
    protocol_version: i32,
    server_address: Cow<'a, str>,
    server_port: u16,
    next_state: NextState0,
}
#[derive(Protocol)]
#[varint]
enum NextState0 {
    Status = 1,
    Login = 2,
}
