use crate::attrs::*;
use crate::errors::InvalidEnumId;
use crate::*;

use protocol_derive::Protocol;
use std::borrow::Cow;

#[derive(Protocol)]
/// this packet as the first one is actually pretty controversial,
/// for 13w41a protocol_version was actually varint, starting 13w42a
/// it is now ushort
pub struct Handshake0<'a> {
    #[varint]
    pub protocol_version: i32,
    pub server_address: Cow<'a, str>,
    pub server_port: u16,
    pub next_state: NextState0,
}
#[derive(Protocol)]
#[varint]
pub enum NextState0 {
    Status = 1,
    Login = 2,
}

protocol_derive::packets! {
    handshaking_sb_custom handshaking_sb_tree;
    0x00 => {
        0..=760 => Handshake0::<'a>,
    }
}
handshaking_sb_custom! {
    pub enum SbHandshaking<'a> {
        #(#PacketName(#PacketType),)
    }
}
impl<'a> SbHandshaking<'a> {
    pub fn parse(id: i32, pv: i32, data: &'a [u8]) -> Result<Self, ReadError> {
        let mut cursor = std::io::Cursor::new(data);
        handshaking_sb_tree! {
            id, pv,
            {<#PacketType as ProtocolRead>::read(&mut cursor).map(SbHandshaking::#PacketName)},
            {Err(ReadError::InvalidProtocolVersionIdCombination)}
        }
    }
}
