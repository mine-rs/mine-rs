use self::byteorder::{BigEndian, NativeEndian};
use miners::encoding::Encode;

pub(crate) mod byteorder;

#[derive(Default, Clone)]
pub struct PackedBits<const N: usize, B: byteorder::ByteOrderedU64> {
    pub(crate) bits: usize,
    mask: u64,
    vpe: usize, // values per element in the vector
    data: Vec<B>,
}

impl<const N: usize> Encode for PackedBits<N, byteorder::NativeEndian> {
    fn encode(&self, writer: &mut impl std::io::Write) -> miners::encoding::encode::Result<()> {
        for i in &self.data {
            i.encode(writer)?;
        }
        Ok(())
    }
}

impl<const N: usize> AsRef<[u64]> for PackedBits<N, byteorder::NativeEndian> {
    fn as_ref(&self) -> &[u64] {
        // SAFETY: This is fine because the `NativeEndian` struct has the same layout as `u64`
        unsafe { std::mem::transmute(self.data.as_slice()) }
    }
}

impl<const N: usize> Encode for PackedBits<N, BigEndian> {
    fn encode(&self, writer: &mut impl std::io::Write) -> miners::encoding::encode::Result<()> {
        writer.write_all(self.as_ref()).map_err(From::from)
    }
}

impl<const N: usize> AsRef<[u8]> for PackedBits<N, BigEndian> {
    fn as_ref(&self) -> &[u8] {
        // SAFETY: This is fine because the `BigEndian` struct has the same layout as a `u64`
        // and there are 8 bytes in a `u64` so the length is multiplied by 8.
        unsafe {
            std::slice::from_raw_parts(std::mem::transmute(self.data.as_ptr()), self.data.len() * 8)
        }
    }
}

impl<const N: usize> PackedBits<N, BigEndian> {
    pub fn from_reader_unchecked(
        rdr: &mut impl std::io::Read,
        bits: usize,
    ) -> std::io::Result<Self> {
        let vpe = 64 / bits; // values per element
        let len = N / vpe;

        let mut data = Vec::<BigEndian>::with_capacity(len);
        data.resize(len, BigEndian::ZERO);

        // SAFETY: This is fine because a u64 is 8 bytes
        let slice = unsafe {
            std::slice::from_raw_parts_mut(std::mem::transmute(data.as_mut_ptr()), len * 8)
        };
        rdr.read_exact(slice)?;

        Ok(Self {
            bits,
            data,
            vpe,
            mask: Self::calculate_mask(bits),
        })
    }

    pub fn from_reader(rdr: &mut impl std::io::Read, bits: usize) -> std::io::Result<Self> {
        if bits == 0 || bits > 32 {
            panic!("invalid amount of bits")
        }
        Self::from_reader_unchecked(rdr, bits)
    }
}

impl<const N: usize> PackedBits<N, NativeEndian> {
    pub fn from_reader_unchecked(
        rdr: &mut impl std::io::Read,
        bits: usize,
    ) -> std::io::Result<Self> {
        let vpe = 64 / bits; // values per element
        let len = N / vpe;

        let mut data = vec![0u64; len];

        // SAFETY: This is fine because a u64 is 8 bytes
        let slice = unsafe {
            std::slice::from_raw_parts_mut(std::mem::transmute(data.as_mut_ptr()), len * 8)
        };
        rdr.read_exact(slice)?;

        for i in &mut data {
            // this swaps the bytes on little endian, converting it to little endian despite the name of the function
            // on big endian this does nothing, as it should
            *i = i.to_be()
        }

        // SAFETY: This is fine because `NativeEndian` is just a u64.
        let data = unsafe { std::mem::transmute(data) };

        Ok(Self {
            bits,
            data,
            vpe,
            mask: Self::calculate_mask(bits),
        })
    }

    pub fn from_reader(rdr: &mut impl std::io::Read, bits: usize) -> std::io::Result<Self> {
        if bits == 0 || bits > 32 {
            panic!("invalid amount of bits")
        }
        Self::from_reader_unchecked(rdr, bits)
    }
}

pub struct PackedBitsIter<const N: usize, B: byteorder::ByteOrderedU64> {
    inner: PackedBits<N, B>,
    index: usize,
}
impl<const N: usize, B: byteorder::ByteOrderedU64> Iterator for PackedBitsIter<N, B> {
    type Item = u32;
    fn next(&mut self) -> Option<Self::Item> {
        let v = self.inner.get(self.index);
        self.index += 1;
        v
    }
}

impl<const N: usize, B: byteorder::ByteOrderedU64> IntoIterator for PackedBits<N, B> {
    type Item = u32;
    type IntoIter = PackedBitsIter<N, B>;
    fn into_iter(self) -> Self::IntoIter {
        PackedBitsIter {
            index: 0,
            inner: self,
        }
    }
}

//impl<const N: usize, B: byteorder::ByteOrderedU64> AsRef<Vec<u64>> for PackedBits<N, B> {
//    fn as_ref(&self) -> &Vec<u64> {
//        &self.data
//    }
//}

impl<const N: usize, B: byteorder::ByteOrderedU64> PackedBits<N, B> {
    /// Constructs a new `PackedBits`, panics if `bits` is equal to zero or if bits is greater than 32.
    #[inline]
    pub fn new(bits: usize) -> Self {
        if bits == 0 || bits > 32 {
            panic!("invalid amount of bits")
        }
        Self::new_unchecked(bits)
    }

    #[inline]
    pub fn new_unchecked(bits: usize) -> Self {
        let vpe = 64 / bits;
        let rlen = (N + vpe - 1) / vpe; // The real length of the vec
        Self {
            bits,
            mask: Self::calculate_mask(bits),
            data: vec![B::from_ne(0); rlen],
            vpe: 64 / bits,
        }
    }

    /// Constructs a new `PackedBits` with data, the data supplied has to already be packed **and be in the right endianness**.
    #[inline]
    #[allow(dead_code)]
    pub fn with_data(bits: usize, data: &[u64]) -> Self {
        let mut this = Self::new(bits);
        // SAFETY: This is fine because we know `B` has the same size as `u64`
        let data = unsafe { std::mem::transmute(data) };
        this.data.copy_from_slice(data);
        this
    }

    /// Constructs a new `PackedBits` with data, the data supplied has to not have been packed yet **and be in native endianness**.
    #[inline]
    #[allow(dead_code)]
    pub fn with_data_unpacked(bits: usize, data: &[u32]) -> Self {
        let mut this = Self::new(bits);
        for (i, v) in data.iter().enumerate() {
            this.set(i, *v);
        }
        this
    }

    #[inline]
    fn calculate_index(&self, i: usize) -> (usize, u64, usize) {
        let vi = i / self.vpe; // vec index
        let bo = i % self.vpe * self.bits; // bit offset
        let bits = self.mask >> bo;
        (vi, bits, bo)
    }

    #[inline]
    pub fn get(&self, i: usize) -> Option<u32> {
        if i >= N {
            return None;
        }
        // SAFETY: This is fine because we already checked that the index is within bounds.
        unsafe { Some(self.get_unchecked(i)) }
    }

    /// # Safety
    /// This is safe as long as `i` is within bounds.
    #[inline]
    pub unsafe fn get_unchecked(&self, i: usize) -> u32 {
        let (vi, bits, bo) = self.calculate_index(i);
        let num = self.data.get_unchecked(vi).to_ne();
        ((num & bits) << bo).rotate_left(self.bits as u32) as u32
    }

    #[inline]
    pub fn set(&mut self, i: usize, v: u32) {
        if i >= N {
            panic!("out of bounds")
        }
        // SAFETY: This is fine because we already checked that the index is within bounds.
        unsafe { self.set_unchecked(i, v) }
    }

    #[inline]
    /// # Safety
    /// This is safe as long as `i` is within bounds.
    pub unsafe fn set_unchecked(&mut self, i: usize, v: u32) {
        let (vi, bits, bo) = self.calculate_index(i);
        let element = self.data.get_unchecked_mut(vi);
        // convert the endianness for usage
        let mut num = element.to_ne();
        num &= !bits; // set the value to zero
        num |= (v as u64).rotate_right(self.bits as u32) >> bo;
        // convert the endianness for storage
        *element = B::from_ne(num);
    }

    #[inline]
    pub fn swap(&mut self, i: usize, v: u32) -> Option<u32> {
        let val = self.get(i)?;
        //SAFETY: This is fine because the self.get call already checked bounds.
        unsafe { self.set_unchecked(i, v) };
        Some(val)
    }

    #[inline]
    /// # Safety
    /// This is sound as long as `i` is within bounds.
    pub unsafe fn swap_unchecked(&mut self, i: usize, v: u32) -> u32 {
        let val = self.get_unchecked(i);
        self.set_unchecked(i, v);
        val
    }

    pub fn change_bits(&mut self, bits: usize) {
        let mut new = Self::new_unchecked(bits);
        for i in 0..N {
            // SAFETY: We know this is sound because 1. the lenghts are the same, and 2. the for loop makes sure `i` is in bounds
            unsafe { new.set_unchecked(i, self.get_unchecked(i)) }
        }
        *self = new;
    }

    fn calculate_mask(bits: usize) -> u64 {
        (((1_u64) << bits) - 1).rotate_right(bits as u32)
    }

    pub fn rlen(&self) -> usize {
        self.data.len()
    }
}
#[cfg(test)]
mod tests {
    use super::byteorder;
    use super::PackedBits;

    fn bitpack<B: byteorder::ByteOrderedU64>() -> PackedBits<8, B> {
        let data = vec![0, 1, 2, 3, 4, 5, 6, 7];
        let new_data = vec![7, 6, 5, 4, 3, 2, 1, 0];
        let mut packedbits = PackedBits::<8, B>::with_data_unpacked(3, &data);
        for bits in 3..=32 {
            for i in 0..8 {
                assert_eq!(packedbits.get(i).unwrap(), data[i]);
                packedbits.set(i, new_data[i]);
                assert_eq!(packedbits.get(i).unwrap(), new_data[i]);
                packedbits.set(i, data[i])
            }

            if bits == 32 {
                break;
            }

            packedbits.change_bits(bits + 1);
        }
        packedbits
    }

    #[test]
    fn bitpack_ne() {
        let _packedbits = bitpack::<byteorder::NativeEndian>();
    }

    #[test]
    fn bitpack_be() {
        let _packedbits = bitpack::<byteorder::BigEndian>();
    }
}
