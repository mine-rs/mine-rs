use crate::*;

pub mod clientbound;
pub mod serverbound;

parsing_tree! {
    login_cb_custom login_cb_tree crate::netty::login::clientbound::;
    0x00 => {
        0..=12 => Disconnect0<'a>,
        13..=384 => Disconnect0<'a>,
        // 385..=390 => _385,
    },
    0x01 => {
        0..=18 => EncryptionResponse0<'a>,
        19..=384 => EncryptionResponse19<'a>,
        // 385..=390 => _385,
    },
    0x02 => {
        0..=4 => Success0<'a>,
        5 => Success5<'a>,
        6..=13 => Success0<'a>,
        14..=384 => Success5<'a>,
        // 385..=390 => _385,
        // 391..=706 => _391,
        // 707..=758 => _707,
        // 759..=760 => _759,
        // 1073741825..=1073741905 => _1073741825,
    },
    0x03 => {
        27..=384 => SetCompression27,
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
        #(#PacketName(#PacketTypeLt),)
    }
}
impl<'a> CbLogin<'a> {
    pub fn parse(id: i32, pv: i32, data: &'a [u8]) -> Result<Self, decode::Error> {
        let mut cursor = std::io::Cursor::new(data);
        login_cb_tree! {
            id, pv,
            {<#PacketTypeLt as Decode>::decode(&mut cursor).map(CbLogin::#PacketName)},
            {Err(decode::Error::InvalidId)}
        }
    }
}

parsing_tree! {
    login_sb_custom login_sb_tree crate::netty::login::serverbound::;
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
        19..=384 => EncryptionRequest19<'a>,
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
        #(#PacketName(#PacketTypeLt),)
    }
}
impl<'a> SbLogin<'a> {
    pub fn parse(id: i32, pv: i32, data: &'a [u8]) -> Result<Self, decode::Error> {
        let mut cursor = std::io::Cursor::new(data);
        login_sb_tree! {
            id, pv,
            {<#PacketTypeLt as Decode>::decode(&mut cursor).map(SbLogin::#PacketName)},
            {Err(decode::Error::InvalidId)}
        }
    }
}
