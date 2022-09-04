use core::slice;
use std::pin::Pin;
use std::task::Poll;
use std::{fmt::Display, io};

use super::INITIAL_BUF_SIZE;

use crate::packet::RawPacket;
use aes::{
    cipher::{inout::InOutBuf, BlockDecryptMut, InvalidLength, KeyIvInit},
    Aes128,
};
use cfb8::Decryptor;
use flate2::Decompress;
use futures_lite::{io::BufReader, AsyncRead, AsyncReadExt};
use futures_lite::ready;

/// The maximum packet length, 8 MiB
const MAX_PACKET_LENGTH: u32 = 1024 * 1024 * 8;

#[inline]
fn verify_len(len: u32) -> std::io::Result<()> {
    if len > MAX_PACKET_LENGTH {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "The data length exceeds the maximum packet length! {len} > {MAX_PACKET_LENGTH}"
            ),
        ))
    } else {
        Ok(())
    }
}


// const AVG_PACKET_THRESHOLD: usize = 65536;

/// The reading half of a connection.
/// Returned from `Connection::split()`
pub struct ReadHalf<R> {
    decryptor: Option<Decryptor<Aes128>>,
    pub(super) compression: Option<Vec<u8>>,
    readbuf: Vec<u8>,
    reader: BufReader<R>,
    //#[cfg(feature = "blocking")]
    //blocking: Option<u32>,
}

impl<R: AsyncRead + Unpin> AsyncRead for ReadHalf<R> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> std::task::Poll<io::Result<usize>> {
        let this = self.get_mut();
        match &mut this.decryptor {
            None => Pin::new(&mut this.reader).poll_read(cx, buf),
            Some(decryptor) => {
                let n = ready!(Pin::new(&mut this.reader).poll_read(cx, buf))?;
                let (chunks, _rest) = InOutBuf::from(buf).into_chunks();
                decryptor.decrypt_blocks_inout_mut(chunks);
                Poll::Ready(Ok(n))
            }
        }
}
    /*
    this is really complicated
    all for using unblock
    fuck unblock
    we ain't doing this
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> std::task::Poll<io::Result<usize>> {
        let this = self.get_mut();
        match &mut this.decryption {
            None => Pin::new(&mut this.reader).poll_read(cx, buf),
            Some(decryption) => {
                // If task is None, that means we haven't spawned the task yet, so we spawn it.
                if decryption.task.is_none() {
                    // Clear the decryption buffer.
                    //decryption.buf.clear();
                    decryption.buf.fill(0);
                    
                    //decryption.buf.resize(1024 * 8, 0);

                    // Read the data to the decryption buffer.
                    let n = ready!(Pin::new(&mut this.reader).poll_read(cx, &mut decryption.buf))?;
                    
                    // We use std::mem::take because we can't pass the decryption buf directly.
                    let mut owned_decryption_buf = std::mem::take(&mut decryption.buf);
                    // We use clone because we can't pass the decryptor directly.
                    let mut owned_decryptor = decryption.decryptor.clone();

                    // Create and store the task.
                    decryption.task = Some(unblock(move || -> (Decryptor<Aes128>, Vec<u8>, usize) {
                        // Decryption
                        let (chunks, _rest) =
                            InOutBuf::from(&mut owned_decryption_buf[0..n]).into_chunks();
                        owned_decryptor.decrypt_blocks_inout_mut(chunks);
                        (owned_decryptor, owned_decryption_buf, n)
                    }));
                }
                //This should be fine because if the task is None, we set it to Some in the above if clause.
                #[allow(clippy::unwrap_used)]
                let task = decryption.task.as_mut().unwrap();

                
                let (decryptor, decryption_buf, n) = ready!(task.poll(cx));
                let len = buf.len();

                let n = if len < n {
                    buf.copy_from_slice(&decryption_buf[decryption.offset..len]);
                    // Put the decryptor and buf back.
                    decryption.buf = decryption_buf;
                    decryption.decryptor = decryptor;
                    // Set the offset
                    decryption.offset = len;
                    // Return the amount of bytes read to the buf supplied.
                    len

                } else {
                    buf.copy_from_slice(&decryption_buf[decryption.offset..n]);
                    // Put the decryptor and buf back.
                    decryption.buf = decryption_buf;
                    decryption.decryptor = decryptor;
                    // Set the task to None.
                    decryption.task = None;
                    // Return the amount of bytes read to the buf supplied
                    n - decryption.offset
                };
                Poll::Ready(Ok(n))
            }
        }
    }
    */
}

#[derive(Debug)]
struct PacketLengthTooLarge;
impl std::error::Error for PacketLengthTooLarge {}
impl Display for PacketLengthTooLarge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("packet length too large")
    }
}

impl<R> ReadHalf<R> {
    pub(super) fn new(
        decryptor: Option<Decryptor<Aes128>>,
        compression: Option<Vec<u8>>,
        reader: BufReader<R>,
    ) -> Self {
        Self {
            decryptor,
            compression,
            readbuf: Vec::with_capacity(INITIAL_BUF_SIZE),
            reader,
            //#[cfg(feature = "blocking")]
            //blocking: None,
        }
    }

    pub(super) fn enable_encryption(&mut self, key: &[u8]) -> Result<(), InvalidLength> {
        self.decryptor = Some(Decryptor::new_from_slices(key, key)?);
        Ok(())
    }

    //#[cfg(feature = "blocking")]
    /// sets the threshold which determines if to offload
    /// packet decryption using cfb8/aes128 to the threadpool
    //pub fn set_blocking_threshold(&mut self, threshold: Option<u32>) {
    //    self.blocking = threshold;
    //}

    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.readbuf.clear();
        self.readbuf.shrink_to(min_capacity);
        if let Some(comp_buf) = &mut self.compression {
            comp_buf.clear();
            comp_buf.shrink_to(min_capacity);
        }
    }
}


impl<R> ReadHalf<R>
where
    R: AsyncRead + Unpin,
{
    pub async fn read_raw_packet(&mut self) -> io::Result<RawPacket<'_>> {
        let mut data =  {
            // read packet length

            let len = read_varint(self).await?;

            verify_len(len)?;

            // reserve enough space

            self.readbuf.reserve(len as usize);

            // get the ref

            // @rob9315 please add a safety comment here
            #[allow(clippy::undocumented_unsafe_blocks)]
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

            let data_len = read_varint(&mut data).await?;

            if data_len != 0 {
                // compression was used

                // ensure uncompressed len is valid
                verify_len(data_len)?;

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

            let id = read_varint(&mut data).await? as i32;

            Ok(RawPacket { id, data })
        } else {
            // no compression

            // read packet id from data

            let id = read_varint(&mut data).await? as i32;

            Ok(RawPacket { id, data })
        }
    }
}

async fn read_varint<R>(reader: &mut R) -> io::Result<u32>
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