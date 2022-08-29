use core::slice;
use std::{fmt::Display, io};

use aes::{
    cipher::{inout::InOutBuf, BlockDecryptMut},
    Aes128,
};
use cfb8::Decryptor;
use flate2::Decompress;
use futures::{io::BufReader, AsyncRead, AsyncReadExt};
use crate::packet::RawPacket;

// const AVG_PACKET_THRESHOLD: usize = 65536;

pub struct ReadHalf<R> {
    decryptor: Option<Decryptor<Aes128>>,
    compression: Option<Vec<u8>>,
    readbuf: Vec<u8>,
    reader: BufReader<R>,
}

//pub struct RawPacket<'a> {
//    pub id: i32,
//    pub data: &'a [u8],
//}

#[derive(Debug)]
struct PacketLengthTooLarge;
impl std::error::Error for PacketLengthTooLarge {}
impl Display for PacketLengthTooLarge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("packet length too large")
    }
}

impl<R> ReadHalf<R>
where
    R: AsyncRead + Unpin,
{
    // todo! add a method for truncating the internal buffers
    pub fn new(decryptor: Option<Decryptor<Aes128>>, compression: Option<Vec<u8>>, reader: BufReader<R>) -> Self {
        Self {
            decryptor,
            compression,
            readbuf: Vec::new(),
            reader
        }
    }

    pub async fn read_raw_packet(&mut self) -> io::Result<RawPacket<'_>> {
        let mut data = if let Some(decryptor) = &mut self.decryptor {
            // entire packet is encrypted

            // read encrypted packet length

            let len = read_encrypted_varint_async(&mut self.reader, decryptor).await?;

            // todo! ensure read len is valid

            // reserve enough space

            self.readbuf.reserve(len as usize);

            // get the ref

            let readslice =
                unsafe { slice::from_raw_parts_mut(self.readbuf.as_mut_ptr(), len as usize) };

            // read into the slice

            self.reader.read_exact(readslice).await?;

            // decrypt read data

            decrypt_in_place(decryptor, readslice);

            // data at readslice is now not encrypted anymore

            &*readslice
        } else {
            // packet is not encrypted

            // read packet length

            let len = read_varint_async(&mut self.reader).await?;

            // todo! ensure read len is valid

            // reserve enough space

            self.readbuf.reserve(len as usize);

            // get the ref

            let readslice =
                unsafe { slice::from_raw_parts_mut(self.readbuf.as_mut_ptr(), len as usize) };

            // read into the slice

            self.reader.read_exact(readslice).await?;

            &*readslice
        };

        if let Some(comp_buf) = &mut self.compression {
            // compression is enabled. Packet looks like this:
            //
            // Length              | VarInt          | how many bytes follow
            // Uncompressed Length | VarInt          | 0 if below threshold, otherwise the uncompressed length
            // Data                | (VarInt, &[u8]) | packet id and data, zlib-compressed

            // clear the compression buffer

            comp_buf.clear();

            // read the uncompressed data length

            let data_len = read_varint(&mut data)?;

            if data_len != 0 {
                // compression was used

                // ensure uncompressed len is valid

                // todo! add a max size check so we don't try to alloc 4gb
                // when a malformed packet is received

                // reserve enough space to inflate data

                comp_buf.reserve(data_len as usize);

                // inflate data

                let mut zlib = Decompress::new(true);
                zlib.decompress_vec(data, comp_buf, flate2::FlushDecompress::Finish)?;

                // update the location of the read data

                data = &comp_buf[..];
            } else {
                // compression wasn't used because the packet is below the threshold
            }

            // read packet id from data

            let id = read_varint(&mut data)? as i32;

            Ok(RawPacket { id, data })
        } else {
            // no compression

            // read packet id from data

            let id = read_varint(&mut data)? as i32;

            Ok(RawPacket { id, data })
        }
    }
}

async fn read_varint_async<R>(reader: &mut R) -> io::Result<u32>
where
    R: AsyncRead + Unpin,
{
    let mut val = 0;
    let mut cur_val = [0];
    for i in 0..5 {
        reader.read_exact(&mut cur_val).await?;
        val += ((cur_val[0] & 0x7f) as u32) << (i * 7);
        if (cur_val[0] & 0x80) == 0x00 {
            break;
        }
    }
    Ok(val)
}

fn decrypt_in_place(decryptor: &mut Decryptor<Aes128>, data: &mut [u8]) {
    let (chunks, rest) = InOutBuf::from(data).into_chunks();
    assert!(rest.is_empty());
    decryptor.decrypt_blocks_inout_mut(chunks);
}

async fn read_encrypted_varint_async<R>(
    reader: &mut R,
    decryptor: &mut Decryptor<Aes128>,
) -> io::Result<u32>
where
    R: AsyncRead + Unpin,
{
    let mut val = 0;
    let mut cur_val = [0];
    for i in 0..5 {
        reader.read_exact(&mut cur_val).await?;
        decrypt_in_place(decryptor, &mut cur_val);
        val += ((cur_val[0] & 0x7f) as u32) << (i * 7);
        if (cur_val[0] & 0x80) == 0x00 {
            break;
        }
    }
    Ok(val)
}
fn read_varint(reader: &mut &[u8]) -> io::Result<u32> {
    let mut val = 0;
    let mut cur_val = [0];
    for i in 0..5 {
        std::io::Read::read_exact(reader, &mut cur_val)?;
        val += ((cur_val[0] & 0x7f) as u32) << (i * 7);
        if (cur_val[0] & 0x80) == 0x00 {
            break;
        }
    }
    Ok(val)
}
