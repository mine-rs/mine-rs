use std::borrow::Cow;

#[derive(Encoding, ToStatic, Debug)]
/// this packet as the first one is actually pretty controversial,
/// for 13w41a protocol_version was actually varint, starting 13w42a
/// it is now ushort
pub struct Handshake0<'a> {
    #[encoding(varint)]
    pub protocol_version: i32,
    pub server_address: Cow<'a, str>,
    pub server_port: u16,
    pub next_state: NextState0,
}
#[derive(Encoding, ToStatic, Debug)]
#[encoding(varint)]
pub enum NextState0 {
    Status = 1,
    Login = 2,
}
