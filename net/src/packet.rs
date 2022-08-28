use futures::{AsyncRead, AsyncWrite, AsyncWriteExt};
use std::io::{Result, Read};
use flate2::read::ZlibEncoder;
use flate2::Compression;

pub struct RawPacket<'a> {
    pub id: i32,
    pub data: &'a [u8]
}

impl<'a> RawPacket<'a> {
    pub async fn unpack<R: AsyncRead + Unpin>(reader: &mut R, bufs: &mut (Vec<u8>, Vec<u8>), threshold: i32) -> Result<RawPacket<'a>>{
        if threshold >= 0 {
            RawPacket::unpack_with_compression(reader, bufs, threshold).await
        } else {
            RawPacket::unpack_without_compression(reader, &mut bufs.0, threshold).await
        }
    }

    pub async fn unpack_with_compression<R: AsyncRead + Unpin>(reader: &mut R, bufs: &mut (Vec<u8>, Vec<u8>), threshold: i32) -> Result<RawPacket<'a>>{
        todo!()
    }

    pub async fn unpack_without_compression<R: AsyncRead + Unpin>(reader: &mut R, buf: &mut Vec<u8>, threshold: i32) -> Result<RawPacket<'a>>{
        todo!()
    }

    pub async fn pack<W: AsyncWrite + Unpin>(self, writer: &mut W, bufs: &mut (Vec<u8>, Vec<u8>), threshold: i32) -> Result<()> {
        
        if threshold >= 0 {
            self.pack_with_compression(writer, bufs, threshold).await
        } else {
            self.pack_without_compression(writer, &mut bufs.0).await
        }
    }

    async fn pack_with_compression<W: AsyncWrite + Unpin>(self, writer: &mut W, bufs: &mut (Vec<u8>, Vec<u8>), threshold: i32) -> Result<()> {
        bufs.0.truncate(0);
        bufs.1.truncate(0);
        write_varint(self.id, &mut bufs.0).await?;
        bufs.0.write_all(self.data).await?;
        let data_len = bufs.0.len();
        if data_len < threshold as usize {
            write_varint(0, &mut bufs.1).await?;
            bufs.1.write_all(&mut bufs.0).await?;
            write_varint(bufs.1.len() as i32, writer).await?;
            writer.write_all(&bufs.1).await?;
        } else {
            //TODO: Use unblock here
            let mut encoder = ZlibEncoder::new(bufs.0.as_slice(), Compression::default());
            write_varint(data_len as i32, &mut bufs.1).await?;
            encoder.read_to_end(&mut bufs.1)?;
            write_varint(bufs.1.len() as i32, writer).await?;
            writer.write_all(&bufs.1).await?;
        }
        Ok(())
    }

    async fn pack_without_compression<W: AsyncWrite + Unpin>(self, writer: &mut W, buf: &mut Vec<u8>) -> Result<()> {
        buf.truncate(0);
        write_varint(self.id, buf).await?;
        buf.write_all(self.data).await?;
        write_varint(buf.len() as i32, writer).await?;
        writer.write_all(buf).await?;
        todo!()
    }
}

async fn write_varint<W>(mut num: i32, writer: &mut W) -> Result<()>
where
    W: AsyncWrite + Unpin,
{
    loop {
        let next_val = (num as u32 >> 7) as i32;
        if next_val == 0 {
            writer.write_all(&[num as u8]).await?;
            break;
        }
        writer.write_all(&[num as u8 | 0x80]).await?;
        num = next_val;
    }
    Ok(())
}

async fn read_varint<R: Unpin + AsyncRead>(reader: &mut R) -> Result<i32> {
    todo!()
}
