use miners_util::bufpool::{request_buf, BufGuard};

use crate::{encoding::EncodedData, helpers::varint_vec};

const ZLIB_BUF_MIN: u32 = 1024;

pub(crate) struct Compression {
    pub(crate) threshold: u32,
    pub(crate) zlib: flate2::Compress,
}

impl Compression {
    fn do_compress<'compressed, 'encoded, 'packed>(
        &mut self,
        encoded: EncodedData,
    ) -> PackedData
    where
        'encoded: 'packed,
        'compressed: 'packed,
    {
        let mut buf = request_buf(encoded.uncompressed_len() as usize);
        buf.clear();

        let uncompressed_len = encoded.uncompressed_len();

        // either do the max check here or allocate once and make sure
        // to never shrink the buffer, it might fail with small packets
        // as zlib might take up more space than the original data
        //
        // note: 5 bytes is the maximum size of the prefixing varint
        buf.reserve(uncompressed_len.max(ZLIB_BUF_MIN + 5) as usize);

        varint_vec(uncompressed_len, &mut buf);

        self.zlib
            .compress_vec(&encoded.0[1..], &mut buf, flate2::FlushCompress::Finish)
            .ok();

        self.zlib.reset();

        PackedData(buf, false)
    }

    pub(crate) fn maybe_compress<'compressed, 'encoded, 'mutslice>(
        &mut self,
        encoded: EncodedData,
    ) -> PackedData
    where
        'encoded: 'mutslice,
        'compressed: 'mutslice,
    {
        if encoded.uncompressed_len() >= self.threshold {
            self.do_compress(encoded)
        } else {
            encoded.zero_prefixed()
        }
    }
}

pub(crate) struct Compressor {
    compression: Compression,
}

impl Compressor {
    pub(crate) fn maybe_compress<'compressed, 'encoded, 'mutslice>(
        &'compressed mut self,
        encoded: EncodedData,
    ) -> PackedData
    where
        'compressed: 'mutslice,
        'encoded: 'mutslice,
    {
        self.compression.maybe_compress(encoded)
    }
}

pub struct PackedData(pub(crate) BufGuard, pub(crate) bool);

impl PackedData {
    pub(crate) fn get(&self) -> &[u8] {
        &self.0[self.1 as usize..]
    }

    pub(crate) fn get_mut(&mut self) -> &mut [u8] {
        &mut self.0[self.1 as usize..]
    }

    pub(crate) fn len(&self) -> u32 {
        self.get().len() as u32
    }

    pub fn fork(&self, mut fork_location: BufGuard) -> PackedData {
        fork_location.clear();
        fork_location.extend_from_slice(&self.0);
        PackedData(fork_location, self.1)
    }
}
