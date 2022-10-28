#[cfg(feature = "packet")]
use crate::encoding::Encoder;
use crate::{encoding::EncodedData, packing::Compression, writer::Writer};
use futures_lite::AsyncWrite;
use miners_packet::DynPacket;
use std::io;

pub struct WriteHalf<W> {
    writer: Writer<W>,
    compression: Option<Compression>,
    compress_buf: Vec<u8>,
}

const DEFAULT_COMPRESS_BUF_CAPACITY: usize = 4096;

impl<W> WriteHalf<W> {
    pub fn new(inner: W) -> WriteHalf<W> {
        WriteHalf {
            writer: Writer::new(inner),
            compression: None,
            compress_buf: Vec::with_capacity(DEFAULT_COMPRESS_BUF_CAPACITY),
        }
    }
    pub fn new_with_capacity(inner: W, capacity: u32) -> WriteHalf<W> {
        WriteHalf {
            writer: Writer::new(inner),
            compression: None,
            compress_buf: Vec::with_capacity(capacity as usize),
        }
    }
    pub fn enable_encryption(&mut self, encryptor: cfb8::Encryptor<aes::Aes128>) {
        self.writer.enable_encryption(encryptor)
    }
}

impl<W> WriteHalf<W>
where
    W: AsyncWrite + Unpin,
{
    pub async fn write<'encoded>(&mut self, encoded: EncodedData<'encoded>) -> io::Result<()> {
        let packed = encoded.split_pack(self.compression.as_mut(), &mut self.compress_buf);
        self.writer.write(packed).await
    }
    pub async fn flush(&mut self) -> io::Result<()> {
        self.writer.flush().await
    }
}

#[cfg(feature = "packet")]
impl<W> WriteHalf<W>
where
    W: AsyncWrite + Unpin,
{
    pub async fn write_packet<P>(
        &mut self,
        version: i32,
        packet: P,
        encoder: &mut Encoder,
    ) -> miners_encoding::encode::Result<()>
    where
        P: miners_packet::Packet,
    {
        if let Some(res) = encoder.encode_packet(version, packet) {
            match res {
                Ok(encoded) => Ok(self.write(encoded).await?),
                Err(e) => Err(e),
            }
        } else {
            #[cfg(debug_assertions)]
            eprintln!(
                "tried to write packet of type {0} in mismatching protocol version {version}",
                std::any::type_name::<P>(),
            );
            Ok(())
        }
    }
    pub async fn write_dyn_packet(
        &mut self,
        version: i32,
        packet: &dyn DynPacket<Vec<u8>>,
        encoder: &mut Encoder,
    ) -> miners_encoding::encode::Result<()> {
        if let Some(res) = encoder.encode_dyn_packet(version, packet) {
            match res {
                Ok(encoded) => Ok(self.write(encoded).await?),
                Err(e) => Err(e),
            }
        } else {
            #[cfg(debug_assertions)]
            eprintln!(
                "tried to write packet of type `{}` in mismatching protocol version {version}", packet.type_name()
            );

            Ok(())
        }
    }
}
