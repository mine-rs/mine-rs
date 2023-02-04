use miners_encoding::encode;
use miners_version::ProtocolVersion;

pub trait Packet {
    fn id_for_version(&self, version: ProtocolVersion,) -> Option<i32>;
    fn encode_for_version(
        &self,
        version: ProtocolVersion,
        writer: &mut impl std::io::Write,
    ) -> Option<encode::Result<()>>;
}

impl<T: Packet> Packet for &T {
    fn id_for_version(&self, version: ProtocolVersion,) -> Option<i32> {
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
    fn exists_in_version(&self, version: ProtocolVersion,) -> bool {
        self.id_for_version(version).is_some()
    }
}
impl<T: Packet> PacketExt for T {}
