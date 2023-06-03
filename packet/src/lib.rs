use miners_encoding::encode;
use miners_version::ProtocolVersion;

pub struct RawPacket<'a> {
    pub id: i32,
    pub data: &'a [u8]
}

impl<'a> RawPacket<'a> {
    pub fn new(id: i32, data: &'a [u8]) -> Self {
        Self {
            id,
            data,
        }
    }
}

impl<'a> From<(i32, &'a [u8])> for RawPacket<'a> {
    fn from(v: (i32, &'a [u8])) -> Self {
        Self::new(v.0, v.1)
    }
}

impl<'a> Into<(i32, &'a [u8])> for RawPacket<'a> {
    fn into(self) -> (i32, &'a [u8]) {
        (self.id, self.data)
    }
}

pub trait Packet {
    fn id_for_version(&self, version: ProtocolVersion) -> Option<i32>;
    fn encode_for_version(
        &self,
        version: ProtocolVersion,
        writer: &mut impl std::io::Write,
    ) -> Option<encode::Result<()>>;
}

impl<T: Packet> Packet for &T {
    fn id_for_version(&self, version: ProtocolVersion) -> Option<i32> {
        (*self).id_for_version(version)
    }

    fn encode_for_version(
        &self,
        version: ProtocolVersion,
        writer: &mut impl std::io::Write,
    ) -> Option<encode::Result<()>> {
        (*self).encode_for_version(version, writer)
    }
}

pub trait PacketExt: Packet {
    fn exists_in_version(&self, version: ProtocolVersion) -> bool {
        self.id_for_version(version).is_some()
    }
}
impl<T: Packet> PacketExt for T {}
