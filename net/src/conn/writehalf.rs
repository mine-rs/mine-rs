use std::{io, num::NonZeroU32};

use super::INITIAL_BUF_SIZE;

use aes::{
    cipher::{inout::InOutBuf, BlockEncryptMut, InvalidLength, KeyIvInit},
    Aes128,
};
use cfb8::Encryptor;
use flate2::Compress;
use futures::{io::BufWriter, AsyncWrite, AsyncWriteExt};

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
                // SAFETY: Since we know theshold is zero or more and we add one to it, we can be certain NonZeroU32::new() isn't supplied a zero so we can use `.unwrap_unchecked()`
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

fn encrypt_in_place(encryptor: &mut Encryptor<Aes128>, data: &mut [u8]) {
    let (chunks, rest) = InOutBuf::from(data).into_chunks();
    debug_assert!(rest.is_empty());
    encryptor.encrypt_blocks_inout_mut(chunks);
}
fn encrypt_into_vec(encryptor: &mut Encryptor<Aes128>, data: &[u8], vec: &mut Vec<u8>) {
    // ensure enough space is available

    vec.reserve(data.len());

    let (chunks, rest) =
        unsafe { InOutBuf::from_raw(data.as_ptr(), vec.as_mut_ptr(), data.len()) }.into_chunks();
    debug_assert!(rest.is_empty());
    encryptor.encrypt_blocks_inout_mut(chunks);

    unsafe { vec.set_len(vec.len() + data.len()) };
}

/// The writing half of a connection.
/// Returned from `Connection::split()`
pub struct WriteHalf<W> {
    encryptor: Option<Encryptor<Aes128>>,
    pub(super) compression: Compression,
    workbuf: Vec<u8>,
    #[cfg(feature = "packets")]
    /// used for serializing packets, only exists when `protocol` is enabled
    workbuf2: Vec<u8>,
    writer: BufWriter<W>,
    #[cfg(feature = "blocking")]
    blocking: Option<u32>,
}

impl<W> WriteHalf<W> {
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.workbuf.clear();
        self.workbuf.shrink_to(min_capacity);
        #[cfg(feature = "packets")]
        {
            self.workbuf2.clear();
            self.workbuf2.shrink_to(min_capacity);
        }
    }

    pub(super) fn new(
        encryptor: Option<Encryptor<Aes128>>,
        compression: Compression,
        writer: BufWriter<W>,
    ) -> Self {
        Self {
            encryptor,
            compression,
            workbuf: Vec::with_capacity(INITIAL_BUF_SIZE),
            #[cfg(feature = "packets")]
            workbuf2: Vec::with_capacity(INITIAL_BUF_SIZE),
            writer,
            #[cfg(feature = "blocking")]
            blocking: None,
        }
    }

    #[cfg(feature = "blocking")]
    /// sets the threshold which determines if to offload
    /// packet encryption using cfb8/aes128 to the threadpool
    pub fn set_blocking_threshold(&mut self, threshold: Option<u32>) {
        self.blocking = threshold;
    }

    pub(super) fn enable_encryption(&mut self, key: &[u8]) -> Result<(), InvalidLength> {
        self.encryptor = Some(Encryptor::new_from_slices(key, key)?);

        Ok(())
    }
}

macro_rules! offload_encrypt {
    ($encryptor:ident, &mut $self:ident.$data:ident, $len:ident, self.$threshold:ident;
        { $($blocking:tt)* }
        $(; $($prewrite:tt)*)?) => {
        #[cfg(feature = "blocking")]
        // blocking feature enabled, if threshold set and compressed_len
        // above threshold, offload the encryption to a threadpool
        match $self.$threshold {
            Some(threshold) if $len > threshold => {
                // get a replacement buffer if this async task fails
                // this should be allocationless as the vec is empty

                let mut xchanged_buf = vec![];

                // exchange workbuf with replacement buf

                std::mem::swap::<Vec<_>>(&mut $self.$data, &mut xchanged_buf);

                // run encryption on threadpool

                let mut cloned_encryptor = $encryptor.clone();

                $(xchanged_buf $($prewrite)*;)?

                let (cloned_encryptor, mut xchanged_buf) = blocking::unblock(move || {
                    encrypt_in_place(&mut cloned_encryptor, &mut xchanged_buf);
                    (cloned_encryptor, xchanged_buf)
                })
                .await;

                // swap the workbuf back into place, delete the temporary replacement

                std::mem::swap::<Vec<_>>(&mut $self.$data, &mut xchanged_buf);
                drop(xchanged_buf);

                // update the encryptor

                *$encryptor = cloned_encryptor
            },
            _ => {
                // encrypt the data in place on this thread
                $($blocking)*
            }
        };

        #[cfg(not(feature = "blocking"))]
        // blocking feature disabled
        // encrypt the data in place on this thread as normal
        $($blocking)*
    };
}

impl<W> WriteHalf<W>
where
    W: AsyncWrite + Unpin,
{
    // todo! refactor write_raw_packet and write_packet so there is less code duplication
    pub async fn write_raw_packet(&mut self, id: i32, data: &[u8]) -> io::Result<()> {
        self.workbuf.clear();

        if let Some((threshold, compression_level)) = self.compression.get_options() {
            // there is compression, packet format as follows:
            //
            // Length              | VarInt          | how many bytes follow
            // Uncompressed Length | VarInt          | 0 if below threshold, otherwise the uncompressed length
            // Data                | (VarInt, &[u8]) | packet id and data, zlib-compressed

            let mut id_buf = [0u8; 6];
            let id_varint = {
                let [_, id_buf @ ..] = &mut id_buf;
                write_varint(id as u32, id_buf)
            };

            let uncompressed_len = id_varint.len() as u32 + data.len() as u32;

            if uncompressed_len > threshold {
                // more than threshold

                // compress to workvec

                let mut zlib = Compress::new(compression_level, true);
                zlib.compress_vec(id_varint, &mut self.workbuf, flate2::FlushCompress::None)?;
                zlib.compress_vec(data, &mut self.workbuf, flate2::FlushCompress::Finish)?;

                // write uncompressed len to buffer

                let mut uncompressed_len_varbuf = [0u8; 5];
                let uncompressed_len_varint =
                    write_varint(uncompressed_len, &mut uncompressed_len_varbuf);

                // write compressed len to buffer

                let compressed_len =
                    uncompressed_len_varint.len() as u32 + self.workbuf.len() as u32;

                let mut compressed_len_varbuf = [0u8; 5];
                let compressed_len_varint =
                    write_varint(compressed_len, &mut compressed_len_varbuf);

                // encrypt data

                if let Some(encryptor) = &mut self.encryptor {
                    // encrypt in order
                    encrypt_in_place(encryptor, compressed_len_varint);
                    encrypt_in_place(encryptor, uncompressed_len_varint);

                    offload_encrypt! {
                        encryptor, &mut self.workbuf, compressed_len, self.blocking;
                        {encrypt_in_place(encryptor, &mut self.workbuf)}
                    }
                }

                // write data to stream

                self.writer.write_all(compressed_len_varint).await?;
                self.writer.write_all(uncompressed_len_varint).await?;
                self.writer.write_all(&self.workbuf).await?;
            } else {
                // less than threshold

                // use trick to "add" a 0 before the id varint
                // this serves as a marker to tell the other party that we aren't using
                // compression as the data size is smaller than the threshold

                let _0_plus_varint = {
                    let varint_len = id_varint.len();
                    &id_buf[0..varint_len + 1]
                };

                let packet_len = _0_plus_varint.len() + data.len();

                if let Some(encryptor) = &mut self.encryptor {
                    // there is encryption

                    // write packet length into workbuf

                    let mut packet_length = [0; 5];
                    let packet_length_varint = write_varint(packet_len as u32, &mut packet_length);
                    encrypt_into_vec(encryptor, packet_length_varint, &mut self.workbuf);

                    // write data with the Encryptor to workbuf

                    encrypt_into_vec(encryptor, _0_plus_varint, &mut self.workbuf);

                    offload_encrypt! {
                        encryptor, &mut self.workbuf, uncompressed_len, self.blocking;
                        {encrypt_into_vec(encryptor, data, &mut self.workbuf)};
                        .write_all(data).await?
                    }

                    // then write the data to the writer

                    self.writer.write_all(&self.workbuf).await?;
                } else {
                    // there is no encryption

                    // write packet length

                    write_varint_async(packet_len as u32, &mut self.writer).await?;

                    // write normal packet but with 0 after varint

                    self.writer.write_all(_0_plus_varint).await?;
                    self.writer.write_all(data).await?;
                }
            }
        } else {
            // there is no compression, packet format as follows:
            //
            // Length | VarInt | how many bytes follow
            // Id     | VarInt | which packet this is
            // Data   | &[u8]  | the data the packet carries

            let packet_len = varint_len(id as u32) + data.len() as u32;

            if let Some(encryptor) = &mut self.encryptor {
                // there is encryption

                // write packet length into workbuf

                let mut packet_length = [0; 5];
                let packet_length_varint = write_varint(packet_len as u32, &mut packet_length);
                encrypt_into_vec(encryptor, packet_length_varint, &mut self.workbuf);

                // write data with the Encryptor to workbuf

                let mut id_buf = [0u8; 5];
                let id = write_varint(id as u32, &mut id_buf);
                encrypt_into_vec(encryptor, id, &mut self.workbuf);

                let uncompressed_len = data.len() as u32;

                offload_encrypt! {
                    encryptor, &mut self.workbuf, uncompressed_len, self.blocking;
                    {encrypt_into_vec(encryptor, data, &mut self.workbuf)};
                    .write_all(data).await?
                }

                // then write it to the writer

                self.writer.write_all(&self.workbuf).await?;
            } else {
                // there is no encryption

                // just write a normal packet

                write_varint_async(packet_len, &mut self.writer).await?;
                write_varint_async(id as u32, &mut self.writer).await?;
                self.writer.write_all(data).await?;
            }
        }

        Ok(())
    }

    #[cfg(feature = "packets")]
    pub async fn write_packet<P>(
        &mut self,
        id: i32,
        packet: P,
    ) -> Result<(), miners_packets::WriteError>
    where
        P: miners_packets::ProtocolWrite,
    {
        self.workbuf.clear();
        self.workbuf2.clear();

        // serialize packet
        // we can optimize a little because we now own the vec
        // this comes in handy for encryption

        packet.write(&mut self.workbuf)?;

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

                // here we can cut **one** corner for the ProtocolWrite type
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
}

fn varint_len(num: u32) -> u32 {
    ((i32::BITS - num.leading_ones()) * 8 + 6) / 7
}
async fn write_varint_async<W>(mut num: u32, writer: &mut W) -> io::Result<()>
where
    W: AsyncWrite + Unpin,
{
    loop {
        let next_val = num >> 7;
        if next_val == 0 {
            writer.write_all(&[num as u8]).await?;
            break;
        }
        writer.write_all(&[num as u8 | 0x80]).await?;
        num = next_val;
    }
    Ok(())
}

fn write_varint(mut num: u32, buf: &mut [u8; 5]) -> &mut [u8] {
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
            let varbuf = write_varint(*num as u32, &mut buf);
            assert_eq!(*res, varbuf)
        }
    }
}
