#[cfg(feature = "encoding")]
use {
    crate::helpers::varint_vec,
    miners_encoding::{decode, encode, Decode, Encode},
};

use miners_packet::DynPacket;

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
pub struct EncodedData<'encoded>(pub(crate) &'encoded mut Vec<u8>);
impl<'encoded> EncodedData<'encoded> {
    pub(crate) fn zero_prefixed(self) -> PackedData<'encoded> {
        PackedData(self.0, false)
    }
    pub(crate) fn uncompressed_len(&self) -> u32 {
        self.0.len() as u32 - 1
    }
    fn stripped_marker(self) -> PackedData<'encoded> {
        PackedData(self.0, true)
    }
    /// Copies the data of an EncodedData reference to a new location,
    /// returning a second one, such "forking" the data
    pub fn fork<'fork>(&self, fork_location: &'fork mut Vec<u8>) -> EncodedData<'fork> {
        fork_location.clear();
        fork_location.extend_from_slice(self.0);
        EncodedData(fork_location)
    }
    /// Constructs a new owned accessor to a vector holding packet data
    ///
    /// # Safety
    ///
    /// the caller must ensure that the referenced vector contains valid
    /// data, else a panic might occur, for example when the vector is empty
    pub unsafe fn from_raw(raw: &mut Vec<u8>) -> EncodedData {
        EncodedData(raw)
    }
    #[cfg(feature = "encoding")]
    pub fn to_packet(&self) -> decode::Result<(i32, &[u8])> {
        let mut cursor = std::io::Cursor::new(&self.0[1..]);

        let id = miners_encoding::attrs::Var::decode(&mut cursor)?.into_inner();
        let pos = cursor.position() as usize;

        Ok((id, &self.0[pos + 1..]))
    }
    #[cfg(feature = "encoding")]
    pub fn into_packet(self) -> decode::Result<(i32, &'encoded [u8])> {
        let mut cursor = std::io::Cursor::new(&self.0[1..]);

        let id = miners_encoding::attrs::Var::decode(&mut cursor)?.into_inner();
        let pos = cursor.position() as usize;

        Ok((id, &self.0[pos + 1..]))
    }
}
impl<'encoded> EncodedData<'encoded> {
    pub fn split_pack<'compressed, 'ret>(
        self,
        compressor: Option<&mut Compression>,
        buf: &'compressed mut Vec<u8>,
    ) -> PackedData<'ret>
    where
        'compressed: 'ret,
        'encoded: 'ret,
    {
        match compressor {
            Some(compression) => compression.maybe_compress(self, buf),
            None => self.stripped_marker(),
        }
    }
    pub fn pack<'compression, 'ret>(
        self,
        compression: Option<&'compression mut Compressor>,
    ) -> PackedData<'ret>
    where
        'compression: 'ret,
        'encoded: 'ret,
    {
        match compression {
            Some(compression) => compression.maybe_compress(self),
            None => self.stripped_marker(),
        }
    }
}

#[derive(Default)]
pub struct Encoder {
    #[cfg_attr(not(feature = "encoding"), allow(unused))]
    encodebuf: Vec<u8>,
}
impl From<Vec<u8>> for Encoder {
    fn from(encodebuf: Vec<u8>) -> Self {
        Encoder { encodebuf }
    }
}
impl Encoder {
    pub fn new() -> Self {
        Encoder { encodebuf: vec![] }
    }
}
impl Encoder {
    #[cfg(feature = "encoding")]
    pub fn encode(&mut self, id: i32, data: impl Encode) -> encode::Result<EncodedData> {
        self.encodebuf.clear();
        self.encodebuf.push(0);
        varint_vec(id as u32, &mut self.encodebuf);
        data.encode(&mut self.encodebuf)?;
        Ok(EncodedData(&mut self.encodebuf))
    }
    #[cfg(feature = "packet")]
    pub fn encode_packet<P>(
        &mut self,
        version: miners_version::ProtocolVersion,
        packet: P,
    ) -> Option<encode::Result<EncodedData>>
    where
        P: miners_packet::Packet,
    {
        self.encodebuf.clear();
        self.encodebuf.push(0);
        match packet.encode_for_version(version, &mut self.encodebuf) {
            Some(Ok(())) => Some(Ok(EncodedData(&mut self.encodebuf))),
            Some(Err(e)) => Some(Err(e)),
            None => None,
        }
    }
    pub fn encode_dyn_packet(
        &mut self,
        version: miners_version::ProtocolVersion,
        packet: &dyn DynPacket<Vec<u8>>,
    ) -> Option<encode::Result<EncodedData>> {
        self.encodebuf.clear();
        self.encodebuf.push(0);
        match packet.dyn_encode_for_version(version, &mut self.encodebuf) {
            Some(Ok(())) => Some(Ok(EncodedData(&mut self.encodebuf))),
            Some(Err(e)) => Some(Err(e)),
            None => None,
        }
    }
}
