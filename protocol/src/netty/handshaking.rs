use crate::*;

pub mod serverbound;

parsing_tree! {
    handshaking_sb_custom handshaking_sb_tree crate::netty::handshaking::serverbound::;
    0x00 => {
        0..=760 => Handshake0::<'a>,
    }
}
handshaking_sb_custom! {
    pub enum SbHandshaking<'a> {
        #(#PacketName(#PacketTypeLt),)
    }
}
impl<'a> SbHandshaking<'a> {
    pub fn parse(id: i32, pv: i32, data: &'a [u8]) -> Result<Self, decode::Error> {
        let mut cursor = std::io::Cursor::new(data);
        handshaking_sb_tree! {
            id, pv,
            {<#PacketTypeLt as Decode>::decode(&mut cursor).map(SbHandshaking::#PacketName)},
            {Err(decode::Error::InvalidId)}
        }
    }
}
