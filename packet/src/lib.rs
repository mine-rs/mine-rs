use miners_encoding::encode;

pub trait Packet {
    fn id_for_version(&self, version: i32) -> Option<i32>;
    fn encode_for_version(
        &self,
        version: i32,
        writer: &mut impl std::io::Write,
    ) -> Option<encode::Result<()>>;
}

pub trait PacketExt: Packet {
    fn exists_in_version(&self, version: i32) -> bool {
        self.id_for_version(version).is_some()
    }
}
impl<T: Packet> PacketExt for T {}

pub trait DynPacket<W: std::io::Write> {
    fn dyn_id_for_version(&self, version: i32) -> Option<i32>;
    fn dyn_encode_for_version(&self, version: i32, writer: &mut W) -> Option<encode::Result<()>>;
    fn type_name(&self) -> &'static str;
}

impl<T: Packet, W: std::io::Write> DynPacket<W> for T {
    fn dyn_id_for_version(&self, version: i32) -> Option<i32> {
        Packet::id_for_version(self, version)
    }
    fn dyn_encode_for_version(&self, version: i32, writer: &mut W) -> Option<encode::Result<()>> {
        Packet::encode_for_version(self, version, writer)
    }
    fn type_name(&self) -> &'static str {
        std::any::type_name::<T>()
    }
}
