use crate::*;

use protocol_derive::Protocol;
use std::borrow::Cow;

#[derive(Protocol)]
pub struct Response0<'a> {
    // todo! json thing
    pub data: Cow<'a, str>,
}

pub use super::Ping0;

protocol_derive::packets! {
    status_cb_custom status_cb_tree;
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
    pub fn parse(id: i32, pv: i32, data: &'a [u8]) -> Result<Self, ReadError> {
        let mut cursor = std::io::Cursor::new(data);
        status_cb_tree! {
            id, pv,
            {<#PacketType as ProtocolRead>::read(&mut cursor).map(CbStatus::#PacketName)},
            {Err(ReadError::InvalidProtocolVersionIdCombination)}
        }
    }
}
