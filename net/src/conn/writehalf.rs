use crate::{
    encoding::{EncodedData, PacketEncodeExt},
    packing::Compression,
    writer::Writer,
};
use flate2::Compress;
use futures_lite::AsyncWrite;
use std::io;

pub struct WriteHalf<W> {
    writer: Writer<W>,
    compression: Option<Compression>,
}

impl<W> WriteHalf<W> {
    pub fn new(inner: W) -> WriteHalf<W> {
        WriteHalf {
            writer: Writer::new(inner),
            compression: None,
        }
    }

    pub fn enable_encryption(&mut self, encryptor: cfb8::Encryptor<aes::Aes128>) {
        self.writer.enable_encryption(encryptor)
    }

    pub(super) fn enable_compression(&mut self, threshold: i32) {
        self.compression = Some(Compression {
            threshold: threshold as u32,
            zlib: Compress::new(flate2::Compression::fast(), true),
        })
    }
}

impl<W> WriteHalf<W>
where
    W: AsyncWrite + Unpin,
{
    pub async fn write<'encoded>(&mut self, encoded: EncodedData) -> io::Result<()> {
        let packed = encoded.split_pack(self.compression.as_mut());
        self.writer.write(packed).await
    }
    pub async fn flush(&mut self) -> io::Result<()> {
        self.writer.flush().await
    }
}

impl<W> WriteHalf<W>
where
    W: AsyncWrite + Unpin,
{
    pub async fn write_packet<P>(
        &mut self,
        version: miners_version::ProtocolVersion,
        mut packet: P,
    ) -> miners_encoding::encode::Result<()>
    where
        P: miners_packet::Packet,
    {
        if let Some(res) = packet.encode_packet(version) {
            match res {
                Ok(encoded) => Ok(self.write(encoded).await?),
                Err(e) => Err(e),
            }
        } else {
            #[cfg(debug_assertions)]
            panic!(
                "tried to write packet of type {0} in mismatching protocol version {version}",
                std::any::type_name::<P>(),
            );
        }
    }
}
