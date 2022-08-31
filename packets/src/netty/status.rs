use crate::*;

use miners_packets_derive::Protocol;

pub mod clientbound;
pub mod serverbound;

#[derive(Protocol)]
pub struct Ping0 {
    pub time: i64,
}

packets! {
    status_cb_custom status_cb_tree clientbound::;
    0x00 => {
        0..=760 => Response0::<'a>,
    },
    0x01 => {
        0..=760 => Ping0,
    }
}
status_cb_custom! {
    pub enum CbStatus<'a> {
        #(#PacketName(#PacketType),)
    }
}
impl<'a> CbStatus<'a> {
    pub fn parse(id: i32, pv: i32, data: &'a [u8]) -> Result<Self, decode::Error> {
        let mut cursor = std::io::Cursor::new(data);
        status_cb_tree! {
            id, pv,
            {<#PacketType as Decode>::decode(&mut cursor).map(CbStatus::#PacketName)},
            {Err(decode::Error::InvalidId)}
        }
    }
}

packets! {
    status_sb_custom status_sb_tree serverbound::;
    0x00 => {
        0..=760 => Request0,
    },
    0x01 => {
        0..=760 => Ping0,
    }
}
status_sb_custom! {
    pub enum SbStatus {
        #(#PacketName(#PacketType),)
    }
}
impl SbStatus {
    pub fn parse(id: i32, pv: i32, data: &[u8]) -> Result<Self, decode::Error> {
        let mut cursor = std::io::Cursor::new(data);
        status_sb_tree! {
            id, pv,
            {<#PacketType as Decode>::decode(&mut cursor).map(SbStatus::#PacketName)},
            {Err(decode::Error::InvalidId)}
        }
    }
}
