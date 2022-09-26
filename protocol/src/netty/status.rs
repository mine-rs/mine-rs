use crate::*;

pub mod clientbound;
pub mod serverbound;

parsing_tree! {
    status_cb_custom status_cb_tree crate::netty::status::clientbound::;
    0x00 => {
        0..=760 => Response0::<'a>,
    },
    0x01 => {
        0..=760 => Ping0,
    }
}
status_cb_custom! {
    pub enum CbStatus<'a> {
        #(#PacketName(#PacketTypeLt),)
    }
    impl<'a> Packet for CbStatus<'a> {
        fn id_for_version(&self, version: i32) -> Option<i32> {
            match self {#(Self::#PacketName(#packet_name) => #packet_name.id_for_version(version),)}
        }
        fn encode_for_version(
            &self,
            version: i32,
            writer: &mut impl std::io::Write,
        ) -> Option<encode::Result<()>> {
            match self {#(Self::#PacketName(#packet_name) => #packet_name.encode_for_version(version, writer),)}
        }
    }
}
impl<'a> CbStatus<'a> {
    pub fn parse(id: i32, pv: i32, data: &'a [u8]) -> Result<Self, decode::Error> {
        let mut cursor = std::io::Cursor::new(data);
        status_cb_tree! {
            id, pv,
            {<#PacketTypeLt as Decode>::decode(&mut cursor).map(CbStatus::#PacketName)},
            {Err(decode::Error::InvalidId)}
        }
    }
}

parsing_tree! {
    status_sb_custom status_sb_tree crate::netty::status::serverbound::;
    0x00 => {
        0..=760 => Request0,
    },
    0x01 => {
        0..=760 => Ping0,
    }
}
status_sb_custom! {
    pub enum SbStatus {
        #(#PacketName(#PacketTypeLt),)
    }
    impl Packet for SbStatus {
        fn id_for_version(&self, version: i32) -> Option<i32> {
            match self {#(Self::#PacketName(#packet_name) => #packet_name.id_for_version(version),)}
        }
        fn encode_for_version(
            &self,
            version: i32,
            writer: &mut impl std::io::Write,
        ) -> Option<encode::Result<()>> {
            match self {#(Self::#PacketName(#packet_name) => #packet_name.encode_for_version(version, writer),)}
        }
    }
}
impl SbStatus {
    pub fn parse(id: i32, pv: i32, data: &[u8]) -> Result<Self, decode::Error> {
        let mut cursor = std::io::Cursor::new(data);
        status_sb_tree! {
            id, pv,
            {<#PacketTypeLt as Decode>::decode(&mut cursor).map(SbStatus::#PacketName)},
            {Err(decode::Error::InvalidId)}
        }
    }
}
