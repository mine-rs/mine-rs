use crate::{ProtocolRead, ReadError};

pub mod serverbound;

packets_derive::packets! {
    handshaking_sb_custom handshaking_sb_tree serverbound::;
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
