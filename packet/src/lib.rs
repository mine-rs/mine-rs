use miners_encoding::encode;

pub trait Packet {
    fn id_for_version(&self, version: i32) -> Option<i32>;
    fn encode_for_version(
        &self,
        version: i32,
        writer: &mut (impl std::io::Write + ?Sized),
    ) -> Option<encode::Result<()>>;
}

pub trait PacketExt: Packet {
    fn exists_in_version(&self, version: i32) -> bool {
        self.id_for_version(version).is_some()
    }
}
impl<T: Packet> PacketExt for T {}
