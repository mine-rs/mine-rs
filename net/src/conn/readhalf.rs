use core::slice;
use std::io::Write;
use std::pin::Pin;
use std::task::Poll;
use std::{fmt::Display, io};

use super::INITIAL_BUF_SIZE;

use crate::packet::RawPacket;
use aes::{
    cipher::{inout::InOutBuf, BlockDecryptMut, InvalidLength, KeyIvInit},
    Aes128,
};
use blocking::{unblock, Task};
use cfb8::Decryptor;
use flate2::Decompress;
use futures_lite::{io::BufReader, AsyncRead, AsyncReadExt};
use futures_lite::{ready, FutureExt};

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

fn decrypt_in_place(decryptor: &mut Decryptor<Aes128>, data: &mut [u8]) {
    let (chunks, rest) = InOutBuf::from(data).into_chunks();
    assert!(rest.is_empty());
    decryptor.decrypt_blocks_inout_mut(chunks);
}

struct Decryption {
    pub decryptor: Decryptor<Aes128>,
    pub task: Option<Task<(Decryptor<Aes128>, Vec<u8>, usize)>>,
    pub offset: usize,
    pub buf: Vec<u8>, 
}

// const AVG_PACKET_THRESHOLD: usize = 65536;

/// The reading half of a connection.
/// Returned from `Connection::split()`
pub struct ReadHalf<R> {
    decryption: Option<Decryption>,
    pub(super) compression: Option<Vec<u8>>,
    readbuf: Vec<u8>,
    reader: BufReader<R>,
    #[cfg(feature = "blocking")]
    blocking: Option<u32>,
}

impl<R: AsyncRead + Unpin> AsyncRead for ReadHalf<R> {
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
                    decryption.buf.clear();
                    decryption.buf.reserve(1024 * 8); // reserve 8 KiB
                    decryption.buf.resize(1024 * 8, 0);

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
        let decryptor = match  decryptor {
            None => None,
            Some(decryptor) => Some(
                Decryption {
                    buf: Vec::with_capacity(INITIAL_BUF_SIZE),
                    offset: 0,
                    decryptor,
                    task: None
                }
            )
        };
        Self {
            decryption: decryptor,
            compression,
            readbuf: Vec::with_capacity(INITIAL_BUF_SIZE),
            reader,
            #[cfg(feature = "blocking")]
            blocking: None,
        }
    }

    pub(super) fn enable_encryption(&mut self, key: &[u8]) -> Result<(), InvalidLength> {
        self.decryption = Some(
            Decryption {
                decryptor: Decryptor::new_from_slices(key, key)?,
                buf: Vec::with_capacity(INITIAL_BUF_SIZE),
                offset: 0,
                task: None,
            }
        );

        Ok(())
    }

    #[cfg(feature = "blocking")]
    /// sets the threshold which determines if to offload
    /// packet decryption using cfb8/aes128 to the threadpool
    pub fn set_blocking_threshold(&mut self, threshold: Option<u32>) {
        self.blocking = threshold;
    }

    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.readbuf.clear();
        self.readbuf.shrink_to(min_capacity);
        if let Some(comp_buf) = &mut self.compression {
            comp_buf.clear();
            comp_buf.shrink_to(min_capacity);
        }
    }
}
/*
impl<R> ReadHalf<R>
where
    R: AsyncRead + Unpin,
{
    pub async fn read_raw_packet(&mut self) -> io::Result<RawPacket<'_>> {
        let mut data = if let Some(decryptor) = &mut self.decryptor {
            // entire packet is encrypted

            // read encrypted packet length

            let len = read_encrypted_varint_async(&mut self.reader, decryptor.0).await?;

            verify_len(len)?;

            // reserve enough space

            self.readbuf.reserve(len as usize);

            // get the ref

            // @rob9315 Please add a safety comment here
            #[allow(clippy::undocumented_unsafe_blocks)]
            let readslice =
                unsafe { slice::from_raw_parts_mut(self.readbuf.as_mut_ptr(), len as usize) };

            // read into the slice

            self.reader.read_exact(readslice).await?;

            // decrypt read data

            #[cfg(feature = "blocking")]
            match self.blocking {
                Some(threshold) if len > threshold => {
                    let mut xchanged_buf = vec![];

                    // exchange workbuf with replacement buf

                    std::mem::swap::<Vec<_>>(&mut self.readbuf, &mut xchanged_buf);

                    // run encryption on threadpool

                    let mut cloned_decryptor = decryptor.clone();

                    let slice_start = readslice.as_mut_ptr() as usize;
                    let (cloned_decryptor, mut xchanged_buf) = blocking::unblock(move || {
                        // SAFETY: This is save because the buffer it is pointing to is owned by this closure.
                        let xchangedbuf_slice = unsafe {
                            slice::from_raw_parts_mut(slice_start as *mut u8, len as usize)
                        };

                        decrypt_in_place(&mut cloned_decryptor, xchangedbuf_slice);
                        (cloned_decryptor, xchanged_buf)
                    })
                    .await;

                    // swap the workbuf back into place, delete the temporary replacement

                    std::mem::swap::<Vec<_>>(&mut self.readbuf, &mut xchanged_buf);
                    drop(xchanged_buf);

                    // update the encryptor

                    *decryptor = cloned_decryptor
                }
                _ => {
                    decrypt_in_place(decryptor, readslice);
                }
            }

            #[cfg(not(feature = "blocking"))]
            decrypt_in_place(decryptor, readslice);

            // data at readslice is now not encrypted anymore

            &*readslice
        } else {
            // packet is not encrypted

            // read packet length

            let len = read_varint_async(&mut self.reader).await?;

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

            let data_len = read_varint(&mut data)?;

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
*/
