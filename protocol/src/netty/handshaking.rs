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
    impl<'a> Packet for SbHandshaking<'a> {
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
    // impl<'a> ToStatic for SbHandshaking<'a> {
    //     type Static = SbHandshaking<'static>;
    //     fn into_static(self) -> Self::Static {
    //         match self {#(Self::#PacketName(#packet_name) => Self::Static::#PacketName(#packet_name.into_static()),)}
    //     }
    //     fn to_static(&self) -> Self::Static {
    //         match self {#(Self::#PacketName(#packet_name) => Self::Static::#PacketName(#packet_name.to_static()),)}
    //     }
    // }
    // impl<'a> Encode for SbHandshaking<'a> {
    //     fn encode(&self, writer: &mut impl std::io::Write) -> encode::Result<()> {
    //         match self {#(Self::#PacketName(#packet_name) => #packet_name.encode(writer),)}
    //     }
    // }
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
