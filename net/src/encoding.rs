use {
    crate::helpers::varint_vec,
    miners_encoding::{decode, encode, Decode, Encode},
};

use miners_packet::RawPacket;
use miners_util::bufpool::{request_buf, BufGuard};

use crate::packing::{Compression, Compressor, PackedData};

/// Holds a mutable reference to a buffer with the following layout
///
/// | marker | id | encoded data |
///
/// where marker is a single `0` byte, id a varint spanning 1-5 bytes
/// and encoded data the packet data
///
/// It holds a mutable reference because the underlying data is being
/// mutated under certain circumstances when writing, more specifically
/// when encrypting. this saves allocations
pub struct EncodedData(pub(crate) BufGuard);

impl EncodedData {
    pub(crate) fn zero_prefixed(self) -> PackedData {
        PackedData(self.0, false)
    }

    pub(crate) fn uncompressed_len(&self) -> u32 {
        self.0.len() as u32 - 1
    }

    fn stripped_marker(self) -> PackedData {
        PackedData(self.0, true)
    }

    /// Copies the data of an EncodedData reference to a new location,
    /// returning a second one, such "forking" the data
    pub fn fork(&self, mut fork_location: BufGuard) -> EncodedData {
        fork_location.clear();
        fork_location.extend_from_slice(&self.0);
        EncodedData(fork_location)
    }

    /// Constructs a new owned accessor to a vector holding packet data
    ///
    /// # Safety
    ///
    /// the caller must ensure that the referenced vector contains valid
    /// data, else a panic might occur, for example when the vector is empty
    pub unsafe fn from_raw(raw: BufGuard) -> EncodedData {
        EncodedData(raw)
    }

    pub fn to_packet(&self) -> decode::Result<RawPacket> {
        let mut cursor = std::io::Cursor::new(&self.0[1..]);

        let id = miners_encoding::attrs::Var::decode(&mut cursor)?.into_inner();
        let pos = cursor.position() as usize;

        Ok(RawPacket::new(id, &self.0[pos + 1..]))
    }

    /*
        pub fn into_packet<'encoded>(self) -> decode::Result<RawPacket<'encoded>> {
        let mut cursor = std::io::Cursor::new(&self.0[1..]);

        let id = miners_encoding::attrs::Var::decode(&mut cursor)?.into_inner();
        let pos = cursor.position() as usize;

        Ok(RawPacket::new(id, &self.0[pos + 1..]))
    }
    */
}

impl EncodedData {
    pub(crate) fn split_pack<'compressed, 'ret>(
        self,
        compressor: Option<&mut Compression>,
        buf: &'compressed mut Vec<u8>,
    ) -> PackedData
    where
        'compressed: 'ret,
    {
        match compressor {
            Some(compression) => compression.maybe_compress(self),
            None => self.stripped_marker(),
        }
    }

    pub(crate) fn pack<'compression, 'ret>(
        self,
        compression: Option<&'compression mut Compressor>,
    ) -> PackedData
    where
        'compression: 'ret,
    {
        match compression {
            Some(compression) => compression.maybe_compress(self),
            None => self.stripped_marker(),
        }
    }
}

#[derive(Default)]
pub struct Encoder {}

impl From<Vec<u8>> for Encoder {
    fn from(encodebuf: Vec<u8>) -> Self {
        Encoder {}
    }
}

impl Encoder {
    pub fn new() -> Self {
        Encoder {}
    }
}

impl Encoder {
    pub fn encode(&mut self, id: i32, data: impl Encode) -> encode::Result<EncodedData> {
        let mut buf = request_buf(1024); // TODO: Set this to something else, or add a way to request the largest currently held buf to prevent reallocs.
        buf.clear();
        buf.push(0);
        varint_vec(id as u32, &mut buf);
        data.encode(&mut buf as &mut Vec<u8>)?;
        Ok(EncodedData(buf))
    }
    pub fn encode_packet<P>(
        &mut self,
        version: miners_version::ProtocolVersion,
        packet: P,
    ) -> Option<encode::Result<EncodedData>>
    where
        P: miners_packet::Packet,
    {
        let mut buf = request_buf(1024);
        buf.clear();
        buf.push(0);
        match packet.encode_for_version(version, &mut buf as &mut Vec<u8>) {
            Some(Ok(())) => Some(Ok(EncodedData(buf))),
            Some(Err(e)) => Some(Err(e)),
            None => None,
        }
    }
}
