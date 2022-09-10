use crate::{encoding::EncodedData, helpers::varint_vec};

const ZLIB_BUF_MIN: u32 = 1024;

pub struct Compression {
    threshold: u32,
    zlib: flate2::Compress,
}
impl Compression {
    fn do_compress<'compressed, 'encoded, 'packed>(
        &mut self,
        encoded: EncodedData<'encoded>,
        buf: &'compressed mut Vec<u8>,
    ) -> PackedData<'packed>
    where
        'encoded: 'packed,
        'compressed: 'packed,
    {
        buf.clear();

        let uncompressed_len = encoded.uncompressed_len();

        // either do the max check here or allocate once and make sure
        // to never shrink the buffer, it might fail with small packets
        // as zlib might take up more space than the original data
        //
        // note: 5 bytes is the maximum size of the prefixing varint
        buf.reserve(uncompressed_len.max(ZLIB_BUF_MIN + 5) as usize);

        varint_vec(uncompressed_len, buf);

        // todo! error handling?
        // this really shouldn't throw errors as we reserve
        self.zlib
            .compress_vec(&encoded.0[1..], buf, flate2::FlushCompress::Finish)
            .ok();

        self.zlib.reset();

        PackedData(buf, false)
    }
    pub(crate) fn maybe_compress<'compressed, 'encoded, 'mutslice>(
        &mut self,
        encoded: EncodedData<'encoded>,
        buf: &'compressed mut Vec<u8>,
    ) -> PackedData<'mutslice>
    where
        'encoded: 'mutslice,
        'compressed: 'mutslice,
    {
        if encoded.uncompressed_len() >= self.threshold {
            self.do_compress(encoded, buf)
        } else {
            encoded.zero_prefixed()
        }
    }
}

pub struct Compressor {
    compression: Compression,
    buf: Vec<u8>,
}
impl Compressor {
    pub(crate) fn maybe_compress<'compressed, 'encoded, 'mutslice>(
        &'compressed mut self,
        encoded: EncodedData<'encoded>,
    ) -> PackedData<'mutslice>
    where
        'compressed: 'mutslice,
        'encoded: 'mutslice,
    {
        self.compression.maybe_compress(encoded, &mut self.buf)
    }
}

pub struct PackedData<'a>(pub(crate) &'a mut Vec<u8>, pub(crate) bool);
impl<'a> PackedData<'a> {
    pub(crate) fn get(&self) -> &[u8] {
        &self.0[self.1 as usize..]
    }
    pub(crate) fn get_mut(&mut self) -> &mut [u8] {
        &mut self.0[self.1 as usize..]
    }
    pub(crate) fn len(&self) -> u32 {
        self.get().len() as u32
    }
    pub fn fork<'fork>(&self, fork_location: &'fork mut Vec<u8>) -> PackedData<'fork> {
        fork_location.clear();
        fork_location.extend_from_slice(self.0);
        PackedData(fork_location, self.1)
    }
}
