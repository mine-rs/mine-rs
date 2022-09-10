use core::slice;
use std::pin::Pin;
use std::task::Poll;
use std::{fmt::Display, io};

use crate::encoding::EncodedData;
use crate::helpers::encrypt;
use crate::DEFAULT_UNBLOCK_THRESHOLD;

use super::INITIAL_BUF_SIZE;

use aes::cipher::{InvalidLength, KeyIvInit};
use futures_lite::ready;
use futures_lite::{AsyncRead, AsyncReadExt};

pub struct RawPacket<'encoded> {
    id: i32,
    data: EncodedData<'encoded>,
}

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
    pub(super) compression: Option<Vec<u8>>,
    readbuf: Vec<u8>,
    reader: Reader<R>,
}

pub struct Reader<R> {
    reader: R,
    decryptor: Option<cfb8::Encryptor<aes::Aes128>>,
    #[cfg(feature = "workpool")]
    unblock_threshold: u32,
}

impl<R> Reader<R>
where
    R: AsyncRead + Unpin,
{
    async fn read(&mut self, buf: &mut Vec<u8>, len: u32) -> io::Result<()> {
        buf.reserve(len as usize);
        // SAFETY: this is safe because the additional length is forcefully allocated in the above line
        let slice = unsafe { slice::from_raw_parts_mut(buf.as_mut_ptr(), len as usize) };
        self.reader.read_exact(slice).await?;
        if let Some(decryptor) = &mut self.decryptor {
            #[cfg(feature = "workpool")]
            if len > self.unblock_threshold {
                let taken_buf = std::mem::take(buf);
                let decryptor_clone = decryptor.clone();
                // SAFETY: this is safe as we are specifying a length that was just written
                let (taken_buf, decryptor_clone) = unsafe {
                    crate::workpool::request_partial_encryption(
                        taken_buf,
                        len as usize,
                        decryptor_clone,
                    )
                    .await
                    .await
                    .unwrap()
                };
                *buf = taken_buf;
                *decryptor = decryptor_clone;
            } else {
                encrypt(slice, decryptor);
            }
            #[cfg(not(feature = "workpool"))]
            encrypt(slice, decryptor);
        }
        Ok(())
    }
}

impl<R: AsyncRead + Unpin> AsyncRead for Reader<R> {
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
                encrypt(buf, decryptor);
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
    pub(super) fn new(decryptor: Option<cfb8::Encryptor<aes::Aes128>>, reader: R) -> Self {
        Self {
            compression: None,
            readbuf: Vec::with_capacity(INITIAL_BUF_SIZE),
            reader: Reader {
                reader,
                decryptor,
                #[cfg(feature = "workpool")]
                unblock_threshold: DEFAULT_UNBLOCK_THRESHOLD,
            },
        }
    }

    pub(super) fn enable_encryption(&mut self, key: &[u8]) -> Result<(), InvalidLength> {
        self.reader.decryptor = Some(cfb8::Encryptor::new_from_slices(key, key)?);
        Ok(())
    }

    #[cfg(feature = "workpool")]
    /// sets the threshold which determines if to offload
    /// packet decryption using cfb8/aes128 to the workpool
    pub fn set_blocking_threshold(&mut self, threshold: u32) {
        self.reader.unblock_threshold = threshold;
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

impl<R> ReadHalf<R>
where
    R: AsyncRead + Unpin,
{
    pub async fn read_encoded(&mut self) -> io::Result<EncodedData> {
        self.readbuf.clear();
        if self.compression.is_none() {
            // push a zero-byte so we adhere to the encoding buffer structure
            self.readbuf.push(0);
        }
        let len = read_varint_async(&mut self.reader).await?;
        self.reader.read(&mut self.readbuf, len).await?;
        match &mut self.compression {
            None => Ok(EncodedData(&mut self.readbuf)),
            Some(_) if self.readbuf[0] == 0 => {
                // compression enabled, prefixed with zero-byte
                Ok(EncodedData(&mut self.readbuf))
            }
            Some(compression_buf) => {
                compression_buf.clear();
                compression_buf.push(0);
                let mut reader = std::io::Cursor::new(&self.readbuf[..]);

                let uncompressed_len = read_varint(&mut reader)?;

                verify_len(uncompressed_len)?;

                compression_buf.reserve_exact(uncompressed_len as usize);

                // todo! reuse decompress
                let mut zlib = flate2::Decompress::new(true);

                // error check?
                zlib.decompress_vec(
                    &reader.get_ref()[reader.get_ref().len()..],
                    compression_buf,
                    flate2::FlushDecompress::Finish,
                ).ok();

                // todo! enable this when reusing decompress
                // zlib.reset(true);

                Ok(EncodedData(compression_buf))
            }
        }
    }
}

fn read_varint<R>(reader: &mut R) -> io::Result<u32>
where
    R: io::Read,
{
    let mut val = 0;
    let mut cur_val = [0];
    for i in 0..5 {
        reader.read_exact(&mut cur_val)?;
        val += ((cur_val[0] & 0x7f) as u32) << (i * 7);
        if (cur_val[0] & 0x80) == 0x00 {
            break;
        }
    }
    Ok(val)
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
