use std::{io, num::NonZeroU32};

use super::INITIAL_BUF_SIZE;

use aes::cipher::{inout::InOutBuf, BlockEncryptMut, InvalidLength, KeyIvInit};
use aes::Aes128;
use cfb8::Encryptor;
use flate2::Compress;
use futures_channel::oneshot;
use futures_lite::{io::BufWriter, AsyncWrite, AsyncWriteExt};
use miners_net_encryption::{crypt, Encryption, EncryptionTask, Task};


/// compression threshold + 1 for memory layout optimization
pub(super) struct Compression(Option<(NonZeroU32, flate2::Compression)>);
impl Compression {
    pub fn new() -> Self {
        Self(None)
    }

    pub fn set_compression(&mut self, threshold: i32, compression: flate2::Compression) {
        if threshold < 0 {
            self.0 = None;
        } else {
            self.0 = Some((
                // SAFETY: Since we know theshold is zero or more and we add
                // one to it, we can be certain NonZeroU32::new() isn't
                // supplied a zero so we can use `.unwrap_unchecked()`
                unsafe { NonZeroU32::new((threshold + 1) as u32).unwrap_unchecked() },
                compression,
            ));
        }
    }

    pub fn get_options(&self) -> Option<(u32, flate2::Compression)> {
        self.0
            .map(|(threshold, lvl)| (u32::from(threshold) - 1, lvl))
    }
}

/// The writing half of a connection.
/// Returned from `Connection::split()`
pub struct WriteHalf<W> {
    pub(super) compression: Compression,
    workbuf: Vec<u8>,
    #[cfg(feature = "encoding")]
    /// used for serializing packets, only exists when `protocol` is enabled
    workbuf2: Vec<u8>,
    writer: Writer<W>,
}

struct Writer<W> {
    pub(crate) writer: W,
    pub buf: Vec<u8>,
    pub(crate) encryption: Option<(*mut Encryption, bool)>,
}

impl<W> Drop for Writer<W> {
    fn drop(&mut self) {
        if let Some((encryption, busy)) = self.encryption {
            if !busy {
                // SAFETY: The busy variable is only false when writer is the sole owner of encryption
                unsafe { encryption.drop_in_place() }
            } // else the threadpool will realise the writer has been dropped and drop the pointer
        }
    }
}

impl<W: AsyncWrite + Unpin> Writer<W> {
    // TODO: Remove the encryptor parameter
    pub fn new(writer: W, _encryptor: Option<Encryptor<Aes128>>) -> Self {
        Self {
            writer,
            buf: Vec::with_capacity(INITIAL_BUF_SIZE),
            encryption: None,
        }
    }
}

impl<W: AsyncWrite + Unpin> Writer<W> {
    #[inline]
    pub async fn write_all(&mut self, buf: &mut Vec<u8>) -> io::Result<()> {
        let owned_buf = std::mem::take(buf);

        match self.encryption.as_mut() {
            None => self.buf.write_all(buf).await,
            Some((encryption, busy)) => {
                if !*busy {
                    let encryption = *encryption;
                    
                    // SAFETY: this is fine because we know the threadpool isn't using the raw pointer atm.
                    unsafe {
                        // We swap the buffer
                        std::mem::swap(&mut(*encryption).buf, &mut self.buf);
                    }

                    let (send, recv) = oneshot::channel();
                    let task = EncryptionTask {
                        data: owned_buf,
                        send,
                        encryption,
                    };
                    // SAFETY: This is fine because we only return after calling .recv and we have a boolean keeping track of if we own the encryptor or not
                    // We can also use unwrap_unchecked because we know we call .send before dropping the sender but we're using expect here
                    unsafe {
                        *busy = true;
                        crypt(Task::Encrypt(task));
                        recv.await
                            .expect("encryption task was terminated externally");
                        
                        // We swap the buffer back
                        std::mem::swap(&mut(*encryption).buf, &mut self.buf);
                        *busy = false;

                    }
                } else {
                    return Err(io::Error::new(
                        io::ErrorKind::Interrupted,
                        "you haven't awaited previous write calls properly",
                    ));
                };
                Ok(())
            }
        }
    }

    #[inline]
    pub async fn flush(&mut self) -> io::Result<()> {
        if let Some((_, busy)) = self.encryption {
            if busy {
                return Err(
                    io::Error::new(
                        io::ErrorKind::Other,
                        "you haven't awaited previous write calls properly"
                    )
                )
            }
        }
        self.writer.write_all(&self.buf).await?;
        self.buf.clear();
        Ok(())
    }

    #[inline]
    pub(super) fn enable_encryption(&mut self, key: &[u8]) -> Result<(), InvalidLength> {
        let encryption = Box::new(Encryption {
            encryptor: Encryptor::new_from_slices(key, key)?,
            buf: Vec::new(),
        });

        self.encryption = Some((Box::into_raw(encryption), false));

        Ok(())
    }
}

impl<W> WriteHalf<W>
where
    W: AsyncWrite + Unpin,
{
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.workbuf.clear();
        self.workbuf.shrink_to(min_capacity);
        #[cfg(feature = "encoding")]
        {
            self.workbuf2.clear();
            self.workbuf2.shrink_to(min_capacity);
        }
    }

    pub(super) fn new(
        encryptor: Option<Encryptor<Aes128>>,
        compression: Compression,
        writer: W,
    ) -> Self {
        Self {
            compression,
            workbuf: Vec::with_capacity(INITIAL_BUF_SIZE),
            #[cfg(feature = "encoding")]
            workbuf2: Vec::with_capacity(INITIAL_BUF_SIZE),
            writer: Writer::new(writer, encryptor),
        }
    }

    #[inline]
    pub(super) fn enable_encryption(&mut self, key: &[u8]) -> Result<(), InvalidLength> {
        self.writer.enable_encryption(key)
    }

    pub async fn write_raw_packet(&mut self, id: i32, data: &[u8]) -> io::Result<()> {
        self.workbuf.clear();

        let mut id_buf = [0u8; 6];
        let id_slice = {
            let [_, id_buf @ ..] = &mut id_buf;
            varint_slice(id as u32, id_buf)
        };

        let id_slice = if let Some((threshold, compression_level)) = self.compression.get_options()
        {
            // there is compression, packet format as follows:
            //
            // Length              | VarInt          | how many bytes follow
            // Uncompressed Length | VarInt          | 0 if below threshold, otherwise the uncompressed length
            // Data                | (VarInt, &[u8]) | packet id and data, zlib-compressed

            let uncompressed_length = id_slice.len() as u32 + data.len() as u32;

            if uncompressed_length > threshold {
                // more than threshold

                // compress to workbuf

                let mut zlib = Compress::new(compression_level, true);
                zlib.compress_vec(id_slice, &mut self.workbuf, flate2::FlushCompress::None)?;
                zlib.compress_vec(data, &mut self.workbuf, flate2::FlushCompress::Finish)?;

                // write uncompressed len to buffer

                let mut uncompressed_length_buf = [0u8; 5];
                let uncompressed_length_slice =
                    varint_slice(uncompressed_length, &mut uncompressed_length_buf);

                let length = uncompressed_length_slice.len() as u32 + self.workbuf.len() as u32;

                let mut length_buf = [0u8; 5];
                /*
                self.writer
                    .write_all(varint_slice(length, &mut length_buf))
                    .await?;
                
                self.writer.write_all(uncompressed_length_slice).await?;
                */
                self.writer.write_all(&mut self.workbuf).await?;
                return Ok(());
            } else {
                // less than threshold

                // use trick to "add" a 0 before the id varint
                // this serves as a marker to tell the other party that we aren't using
                // compression as the data size is smaller than the threshold

                let varint_len = id_slice.len();
                &mut id_buf[0..varint_len + 1]
            }
        } else {
            // there is no compression, packet format as follows:
            //
            // Length | VarInt | how many bytes follow
            // Id     | VarInt | which packet this is
            // Data   | &[u8]  | the data the packet carries

            id_slice
        };

        // id slice is now either just the packet id or the id prefixed with 0
        // depending on if compression is enabled or not

        // compute actual packet length

        let length = id_slice.len() as u32 + data.len() as u32;

        let mut length_buf = [0; 5];
        /*/
        self.writer
            .write_all(varint_slice(length, &mut length_buf))
            .await?;

        self.writer.write_all(id_slice).await?;

        */
        // only copy to workbuf if encryption is enabled

        if self.writer.encryption.is_some() {
            std::io::Write::write_all(&mut self.workbuf, data)?;
            self.writer.write_all(&mut self.workbuf).await?;
        } else {
            self.writer.writer.write_all(data).await?;
        }

        Ok(())
    }

    /*
    // This will hopefully replace write_packet
    #[cfg(feature = "encoding")]
    pub async fn write_packet_no_duplication<P>(
        &mut self,
        id: i32,
        packet: P,
    ) -> Result<(), miners_encoding::encode::Error>
    where
        P: miners_encoding::Encode,
    {
        self.workbuf2.clear();
        packet.encode(&mut self.workbuf2)?;
        let buf = std::mem::take(&mut self.workbuf2);
        self.write_raw_packet(id, &buf).await?;
        self.workbuf2 = buf;
        Ok(())
    }

    #[cfg(feature = "encoding")]
    pub async fn write_packet<P>(
        &mut self,
        id: i32,
        packet: P,
    ) -> Result<(), miners_encoding::encode::Error>
    where
        P: miners_encoding::Encode,
    {
        self.workbuf.clear();
        self.workbuf2.clear();

        // serialize packet
        // we can optimize a little because we now own the vec
        // this comes in handy for encryption

        packet.encode(&mut self.workbuf)?;

        if let Some((threshold, compression_level)) = self.compression.get_options() {
            // there is compression, packet format as follows:
            //
            // Length              | VarInt          | how many bytes follow
            // Uncompressed Length | VarInt          | 0 if below threshold, otherwise the uncompressed length
            // Data                | (VarInt, &[u8]) | packet id and data, zlib-compressed

            // write packet id

            let mut id_buf = [0u8; 6];
            let id_varint = {
                let [_, id_buf @ ..] = &mut id_buf;
                write_varint(id as u32, id_buf)
            };

            let uncompressed_len = id_varint.len() as u32 + self.workbuf.len() as u32;

            if uncompressed_len > threshold {
                // more than threshold

                // compress to workvec2

                let mut zlib = Compress::new(compression_level, true);
                zlib.compress_vec(id_varint, &mut self.workbuf2, flate2::FlushCompress::None)
                    .map_err(io::Error::from)?;
                zlib.compress_vec(
                    &self.workbuf,
                    &mut self.workbuf2,
                    flate2::FlushCompress::Finish,
                )
                .map_err(io::Error::from)?;

                // write uncompressed len to buffer

                let mut uncompressed_len_varbuf = [0u8; 5];
                let uncompressed_len_varint =
                    write_varint(uncompressed_len, &mut uncompressed_len_varbuf);

                // write compressed len to buffer

                let compressed_len =
                    uncompressed_len_varint.len() as u32 + self.workbuf2.len() as u32;

                let mut compressed_len_varbuf = [0u8; 5];
                let compressed_len_varint =
                    write_varint(compressed_len, &mut compressed_len_varbuf);

                // encrypt data

                if let Some(encryptor) = &mut self.encryptor {
                    // encrypt in order
                    encrypt_in_place(encryptor, compressed_len_varint);
                    encrypt_in_place(encryptor, uncompressed_len_varint);
                    offload_encrypt! {
                        encryptor, &mut self.workbuf2, uncompressed_len, self.blocking;
                        {encrypt_in_place(encryptor, &mut self.workbuf2)}
                    }
                }

                self.writer.write_all(compressed_len_varint).await?;
                self.writer.write_all(uncompressed_len_varint).await?;
                self.writer.write_all(&self.workbuf2).await?;
            } else {
                // less than threshold

                // use trick to "add" a 0 before the id varint
                // this serves as a marker to tell the other party that we aren't using
                // compression as the data size is smaller than the threshold

                let _0_plus_varint = {
                    let varint_len = id_varint.len();
                    &id_buf[0..varint_len + 1]
                };

                let packet_len = _0_plus_varint.len() + self.workbuf.len();

                if let Some(encryptor) = &mut self.encryptor {
                    // there is encryption

                    // write packet length into workbuf2

                    let mut packet_length = [0; 5];
                    let packet_length_varint = write_varint(packet_len as u32, &mut packet_length);
                    encrypt_into_vec(encryptor, packet_length_varint, &mut self.workbuf2);

                    // write data with the Encryptor to workbuf

                    encrypt_into_vec(encryptor, _0_plus_varint, &mut self.workbuf2);

                    #[cfg(feature = "blocking")]
                    // blocking feature enabled, if threshold set and compressed_len
                    // above threshold, offload the encryption to a threadpool
                    match self.blocking {
                        Some(threshold) if uncompressed_len > threshold => {
                            // get a replacement buffer if this async task fails
                            // this should be allocationless as the vec is empty

                            let mut xchanged_inbuf = vec![];
                            let mut xchanged_outbuf = vec![];

                            // exchange workbuf with replacement buf

                            std::mem::swap::<Vec<_>>(&mut self.workbuf, &mut xchanged_inbuf);
                            std::mem::swap::<Vec<_>>(&mut self.workbuf2, &mut xchanged_outbuf);

                            // run encryption on threadpool

                            let mut cloned_encryptor = encryptor.clone();

                            let (cloned_encryptor, mut xchanged_inbuf, mut xchanged_outbuf) =
                                blocking::unblock(move || {
                                    encrypt_into_vec(
                                        &mut cloned_encryptor,
                                        &xchanged_inbuf,
                                        &mut xchanged_outbuf,
                                    );
                                    (cloned_encryptor, xchanged_inbuf, xchanged_outbuf)
                                })
                                .await;

                            // swap the workbuf back into place, delete the temporary replacement

                            std::mem::swap::<Vec<_>>(&mut self.workbuf, &mut xchanged_inbuf);
                            std::mem::swap::<Vec<_>>(&mut self.workbuf2, &mut xchanged_outbuf);
                            drop(xchanged_inbuf);
                            drop(xchanged_outbuf);

                            // update the encryptor

                            *encryptor = cloned_encryptor
                        }
                        _ => {
                            // encrypt the data in place on this thread
                            encrypt_into_vec(encryptor, &self.workbuf, &mut self.workbuf2);
                        }
                    };

                    #[cfg(not(feature = "blocking"))]
                    // blocking feature disabled
                    // encrypt the data in place on this thread as normal
                    encrypt_into_vec(encryptor, &self.workbuf, &mut self.workbuf2);

                    // then write the data to the writer

                    self.writer.write_all(&self.workbuf2).await?;
                } else {
                    // there is no encryption

                    // write packet length

                    write_varint_async(packet_len as u32, &mut self.writer).await?;

                    // write normal packet but with 0 after varint

                    self.writer.write_all(_0_plus_varint).await?;
                    self.writer.write_all(&self.workbuf).await?;
                }
            }
        } else {
            // there is no compression, packet format as follows:
            //
            // Length | VarInt | how many bytes follow
            // Id     | VarInt | which packet this is
            // Data   | &[u8]  | the data the packet carries

            // write packet id

            let mut id_buf = [0u8; 5];
            let id_varint = write_varint(id as u32, &mut id_buf);

            // write packet length

            let packet_len = id_varint.len() as u32 + self.workbuf.len() as u32;
            let mut packet_length = [0; 5];
            let packet_length_varint = write_varint(packet_len, &mut packet_length);

            if let Some(encryptor) = &mut self.encryptor {
                encrypt_in_place(encryptor, packet_length_varint);
                encrypt_in_place(encryptor, id_varint);

                // here we can cut **one** corner for the Encode type
                // because we own the vec so we can encrypt it in place

                offload_encrypt! {
                    encryptor, &mut self.workbuf, packet_len, self.blocking;
                    {encrypt_in_place(encryptor, &mut self.workbuf)}
                }
            }

            self.writer.write_all(packet_length_varint).await?;
            self.writer.write_all(id_varint).await?;
            self.writer.write_all(&self.workbuf).await?;
        }

        Ok(())
    }
    */
    // */
}

fn varint_slice(mut num: u32, buf: &mut [u8; 5]) -> &mut [u8] {
    #[allow(clippy::needless_range_loop)]
    for i in 0..5 {
        let next_val = num >> 7;
        if next_val == 0 {
            buf[i] = num as u8;
            return &mut buf[..i + 1];
        }
        buf[i] = num as u8 | 0x80;
        num = next_val;
    }
    &mut buf[..]
}

fn varint_vec(mut num: u32, vec: &mut Vec<u8>) {
    for _ in 0..5 {
        let next_val = num >> 7;
        if next_val == 0 {
            vec.push(num as u8);
            break;
        }
        vec.push(num as u8 | 0x80);
        num = next_val;
    }
}

#[cfg(test)]
mod varint {
    use super::*;
    const TESTS: &[(i32, &[u8])] = &[
        (0, &[0x00]),
        (1, &[0x01]),
        (127, &[0x7f]),
        (128, &[0x80, 1]),
        (255, &[0xff, 0x01]),
        (25565, &[0xdd, 0xc7, 0x01]),
        (2097151, &[0xff, 0xff, 0x7f]),
        (2147483647, &[0xff, 0xff, 0xff, 0xff, 0x07]),
        (-1, &[0xff, 0xff, 0xff, 0xff, 0x0f]),
        (-2147483648, &[0x80, 0x80, 0x80, 0x80, 0x08]),
    ];
    #[test]
    fn write() {
        for (num, res) in TESTS {
            let mut buf = [0u8; 5];
            let varbuf = varint_slice(*num as u32, &mut buf);
            assert_eq!(*res, varbuf)
        }
    }
}
