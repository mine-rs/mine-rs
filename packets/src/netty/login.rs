use crate::{ProtocolRead, ReadError};

pub mod clientbound;
pub mod serverbound;

protocol_derive::packets! {
    login_cb_custom login_cb_tree clientbound::;
    0x00 => {
        0..=12 => Disconnect0<'a>,
        // 13..=384 => _13,
        // 385..=390 => _385,
    },
    0x01 => {
        0..=18 => EncryptionResponse0<'a>,
        // 19..=384 => _19,
        // 385..=390 => _385,
    },
    0x02 => {
        0..=4 => Success0<'a>,
        5 => Success5<'a>,
        // 6..=13 => _6,
        // 14..=384 => _14,
        // 385..=390 => _385,
        // 391..=706 => _391,
        // 707..=758 => _707,
        // 759..=760 => _759,
        // 1073741825..=1073741905 => _1073741825,
    },
    0x03 => {
        // 27..=384 => _27,
        // 385..=390 => _385,
        // 391..=760 => _391,
    },
    0x04 => {
        // 385..=390 => _385,
        // 391..=760 => _391,
    }
}
login_cb_custom! {
    pub enum CbLogin<'a> {
        #(#PacketName(#PacketType),)
    }
}
impl<'a> CbLogin<'a> {
    pub fn parse(id: i32, pv: i32, data: &'a [u8]) -> Result<Self, ReadError> {
        let mut cursor = std::io::Cursor::new(data);
        login_cb_tree! {
            id, pv,
            {<#PacketType as ProtocolRead>::read(&mut cursor).map(CbLogin::#PacketName)},
            {Err(ReadError::InvalidProtocolVersionIdCombination)}
        }
    }
}

protocol_derive::packets! {
    login_sb_custom login_sb_tree serverbound::;
    0x00 => {
        0..=384 => LoginStart0<'a>,
        // 385..=390 => _385,
        // 391..=758 => _391,
        // 759 => _759,
        // 760 => _760,
        // 1073741825..=1073741905 => _1073741825,
        // 1073741906..=1073741907 => _1073741906,
        // 1073741908..=1073741918 => _1073741908,
    },
    0x01 => {
        0..=18 => EncryptionRequest0<'a>,
        // 19..=384 => _19,
        // 385..=390 => _385,
        // 391..=758 => _391,
        // 759..=760 => _759,
        // 1073741825..=1073741905 => _1073741825,
    },
    0x02 => {
        // 385..=390 => _385,
        // 391..=758 => _391,
        // 759..=760 => _759,
        // 1073741825..=1073741906 => _1073741825,
    }
}
login_sb_custom! {
    pub enum SbLogin<'a> {
        #(#PacketName(#PacketType),)
    }
}
impl<'a> SbLogin<'a> {
    pub fn parse(id: i32, pv: i32, data: &'a [u8]) -> Result<Self, ReadError> {
        let mut cursor = std::io::Cursor::new(data);
        login_sb_tree! {
            id, pv,
            {<#PacketType as ProtocolRead>::read(&mut cursor).map(SbLogin::#PacketName)},
            {Err(ReadError::InvalidProtocolVersionIdCombination)}
        }
    }
}
