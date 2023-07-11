use miners_encoding::{attrs::Var, Decode, Encode};

use super::bitpack::{
    byteorder::{BigEndian, NativeEndian},
    PackedBits,
};
use std::collections::BTreeMap;
/// # Safety
/// This trait is safe to implement as long as you don't override the set/get methods without bounds checking
pub unsafe trait PaletteContainer {
    const N: usize;
    fn new(value: u16) -> Self;
    fn with_bits(bits: usize, value: u16) -> Self;

    fn get(&self, i: usize) -> u16 {
        if i >= Self::N {
            panic!("out of bounds")
        }
        //SAFETY: This is safe because we know i is in bounds.
        unsafe { self.get_unchecked(i) }
    }

    /// # Safety
    /// This method is safe as long as `i` is within bounds.
    unsafe fn get_unchecked(&self, i: usize) -> u16;

    fn set(&mut self, i: usize, v: u16) {
        if i >= Self::N {
            panic!("out of bounds")
        }
        // SAFETY: This is sound because we just checked the bounds
        unsafe { self.set_unchecked(i, v) }
    }

    /// # Safety
    /// This method is safe as long as `i` is within bounds.
    unsafe fn set_unchecked(&mut self, i: usize, v: u16);

    fn swap(&mut self, i: usize, v: u16) -> u16 {
        if i >= Self::N {
            panic!("out of bounds")
        }
        //SAFETY: This is safe because we just checked the bounds.
        unsafe { self.swap_unchecked(i, v) }
    }

    /// # Safety
    /// This method is safe as long as `i` is within bounds
    unsafe fn swap_unchecked(&mut self, i: usize, v: u16) -> u16 {
        let val = self.get_unchecked(i);
        self.set_unchecked(i, v);
        val
    }
}

pub struct BiomePaletteContainer<const N: usize, B: super::bitpack::byteorder::ByteOrderedU64> {
    palette: BiomePalette<N, B>,
}

macro_rules! impl_encoding_for_biome_endian {
    ($endian:ident) => {
        impl<const N: usize> Encode for BiomePaletteContainer<N, $endian> {
            fn encode(
                &self,
                writer: &mut impl std::io::Write,
            ) -> miners_encoding::encode::Result<()> {
                #[inline]
                fn encode_indirect<const N: usize>(
                    writer: &mut impl std::io::Write,
                    palette: &LinearPalette,
                    data: &PackedBits<N, $endian>,
                ) -> miners_encoding::encode::Result<()> {
                    (palette.bits as u8).encode(writer)?; // write the amount of bits
                    Var::<i32>::from(palette.values.len() as i32).encode(writer)?; // write the length of the palette array

                    // write the palette
                    for i in &palette.values {
                        Var::<i32>::from(*i as i32).encode(writer)?;
                    }

                    Var::<i32>::from(data.rlen() as i32).encode(writer)?; // write the length of the data array
                    data.encode(writer) // write the data
                }

                match &self.palette {
                    BiomePalette::SingleValue(v) => {
                        0u8.encode(writer)?; // write the amount of bits (0)
                        Var::<i32>::from(v.0 as i32).encode(writer)?; // write the value
                        Var::<i32>::from(0).encode(writer) // write the length of the data array
                                                           // data array is empty.
                    }
                    BiomePalette::Linear { palette, data } => {
                        encode_indirect(writer, palette, data)
                    }
                }
            }
        }

        impl<'dec, const N: usize> Decode<'dec> for BiomePaletteContainer<N, $endian> {
            fn decode(
                cursor: &mut std::io::Cursor<&'dec [u8]>,
            ) -> miners_encoding::decode::Result<Self> {
                let bits = u8::decode(cursor)?;
                Ok(Self {
                    palette: match bits {
                        0 => BiomePalette::<N, $endian>::SingleValue(SingleValuePalette(
                            Var::<i32>::decode(cursor)?.into_inner() as u16,
                        )),
                        1..=3 => {
                            let len: usize = Var::<i32>::decode(cursor)?.into_inner() as usize;
                            let mut palette = Vec::<u16>::with_capacity(len);
                            for _ in 0..len {
                                palette.push(Var::<i32>::decode(cursor)?.into_inner() as u16)
                            }
                            let _len = Var::<i32>::decode(cursor)?;
                            let data = PackedBits::<N, $endian>::from_reader_unchecked(
                                cursor,
                                bits as usize,
                            )?;

                            let linear = LinearPalette {
                                values: palette,
                                bits: bits as usize,
                            };
                            BiomePalette::Linear {
                                palette: linear,
                                data,
                            }
                        }
                        _ => {
                            return Err(miners_encoding::decode::Error::Custom(
                                "invalid amount of bits for palette container!",
                            ));
                        }
                    },
                })
            }
        }
    };
}

impl_encoding_for_biome_endian!(BigEndian);
impl_encoding_for_biome_endian!(NativeEndian);

enum BiomePalette<const N: usize, B: super::bitpack::byteorder::ByteOrderedU64> {
    SingleValue(SingleValuePalette),
    Linear {
        palette: LinearPalette,
        data: PackedBits<N, B>,
    },
}

// SAFETY: This is fine because we uphold all of the invariants
unsafe impl<const N: usize, B: super::bitpack::byteorder::ByteOrderedU64> PaletteContainer
    for BiomePaletteContainer<N, B>
{
    const N: usize = N;
    fn new(value: u16) -> Self {
        Self {
            palette: BiomePalette::SingleValue(SingleValuePalette(value)),
        }
    }

    fn with_bits(bits: usize, value: u16) -> Self {
        if bits > 3 {
            panic!("bits cannot exceed 3")
        }
        //SAFETY: This is safe because we just checked that bits is not greater than 3.
        unsafe { Self::with_bits_unchecked(bits, value) }
    }

    unsafe fn get_unchecked(&self, i: usize) -> u16 {
        match &self.palette {
            BiomePalette::SingleValue(v) => v.0,
            BiomePalette::Linear { palette, data } => palette.value(data.get_unchecked(i) as usize),
        }
    }

    unsafe fn set_unchecked(&mut self, i: usize, v: u16) {
        loop {
            match &mut self.palette {
                BiomePalette::SingleValue(val) => match val.index(v) {
                    IndexOrBits::Index(_) => return,
                    IndexOrBits::Bits(bits) => {
                        let mut values = Vec::new();
                        values.reserve_exact(2);
                        values.push(val.0);
                        let palette = BiomePalette::Linear {
                            palette: LinearPalette { bits, values },
                            data: PackedBits::new(1),
                        };
                        self.palette = palette
                    }
                },
                BiomePalette::Linear { palette, data } => match palette.index(v) {
                    IndexOrBits::Index(v) => return data.set_unchecked(i, v as u32),
                    IndexOrBits::Bits(bits) => {
                        if bits > 3 {
                            panic!("bits cannot exceed 3")
                        }
                        let mut values = std::mem::take(&mut palette.values);
                        values.reserve_exact(values.capacity());
                        data.change_bits(bits);

                        let data = std::mem::take(data);

                        let palette = BiomePalette::Linear {
                            palette: LinearPalette { bits, values },
                            data,
                        };

                        self.palette = palette
                    }
                },
            }
        }
    }
}

impl<const N: usize, B: super::bitpack::byteorder::ByteOrderedU64> BiomePaletteContainer<N, B> {
    /// # Safety
    /// This method is safe as long as `bits` is not greater than 3.
    pub unsafe fn with_bits_unchecked(bits: usize, value: u16) -> Self {
        match bits {
            0 => Self::new(value),
            // Here we assume bits is 1, 2, or 3
            bits => {
                let mut values = Vec::new();
                values.reserve_exact(2usize.pow(bits as u32));
                let palette = LinearPalette { bits, values };
                Self {
                    palette: BiomePalette::Linear {
                        palette,
                        data: PackedBits::new_unchecked(bits),
                    },
                }
            }
        }
    }
}

pub struct StatePaletteContainer<const N: usize, B: super::bitpack::byteorder::ByteOrderedU64> {
    palette: StatePalette<N, B>,
}

macro_rules! impl_encoding_for_state_endian {
    ($endian:ident) => {
        impl<const N: usize> Encode for StatePaletteContainer<N, $endian> {
            fn encode(
                &self,
                writer: &mut impl std::io::Write,
            ) -> miners_encoding::encode::Result<()> {
                #[inline]
                fn encode_indirect<const N: usize>(
                    writer: &mut impl std::io::Write,
                    palette: &LinearPalette,
                    data: &PackedBits<N, $endian>,
                ) -> miners_encoding::encode::Result<()> {
                    (palette.bits as u8).encode(writer)?; // write the amount of bits
                    Var::<i32>::from(palette.values.len() as i32).encode(writer)?; // write the length of the palette array

                    // write the palette
                    for i in &palette.values {
                        Var::<i32>::from(*i as i32).encode(writer)?;
                    }

                    Var::<i32>::from(data.rlen() as i32).encode(writer)?; // write the length of the data array
                    data.encode(writer) // write the data
                }

                match &self.palette {
                    StatePalette::SingleValue(v) => {
                        0u8.encode(writer)?; // write the amount of bits (0)
                        Var::<i32>::from(v.0 as i32).encode(writer)?; // write the value
                        Var::<i32>::from(0).encode(writer) // write the length of the data array
                                                           // data array is empty.
                    }
                    StatePalette::Linear { palette, data } => {
                        encode_indirect(writer, palette, data)
                    }
                    StatePalette::Mapped { palette, data } => {
                        encode_indirect(writer, &palette.inner, data)
                    }
                    StatePalette::Global { data } => {
                        15u8.encode(writer)?; // write the amount of bits
                        Var::<i32>::from(data.rlen() as i32).encode(writer)?; // write the length of the data
                        data.encode(writer) // write the data
                    }
                }
            }
        }

        impl<'dec, const N: usize> Decode<'dec> for StatePaletteContainer<N, $endian> {
            fn decode(
                cursor: &mut std::io::Cursor<&'dec [u8]>,
            ) -> miners_encoding::decode::Result<Self> {
                let bits = u8::decode(cursor)?;
                Ok(Self {
                    palette: match bits {
                        0 => StatePalette::<N, $endian>::SingleValue(SingleValuePalette(
                            Var::<i32>::decode(cursor)?.into_inner() as u16,
                        )),
                        4..=8 => {
                            let len: usize = Var::<i32>::decode(cursor)?.into_inner() as usize;
                            let mut palette = Vec::<u16>::with_capacity(len);
                            for _ in 0..len {
                                palette.push(Var::<i32>::decode(cursor)?.into_inner() as u16)
                            }
                            let _len = Var::<i32>::decode(cursor)?;
                            let data = PackedBits::<N, $endian>::from_reader_unchecked(
                                cursor,
                                bits as usize,
                            )?;
                            if bits > 4 {
                                let linear = LinearPalette {
                                    values: palette,
                                    bits: bits as usize,
                                };
                                StatePalette::Linear {
                                    palette: linear,
                                    data,
                                }
                            } else {
                                // bits == 4
                                let mut indices = BTreeMap::new();
                                for i in 0..palette.len() {
                                    // SAFETY: This is fine because the len will always be in bounds due to the for loop.
                                    unsafe { indices.insert(*palette.get_unchecked(i), i) };
                                }
                                let linear = LinearPalette {
                                    values: palette,
                                    bits: bits as usize,
                                };
                                StatePalette::Mapped {
                                    palette: MappedPalette {
                                        indices,
                                        inner: linear,
                                    },
                                    data,
                                }
                            }
                        }
                        15 => {
                            let _len = Var::<i32>::decode(cursor)?;
                            StatePalette::Global {
                                data: PackedBits::<N, $endian>::from_reader_unchecked(
                                    cursor,
                                    bits as usize,
                                )?,
                            }
                        }
                        _ => {
                            return Err(miners_encoding::decode::Error::Custom(
                                "invalid amount of bits for palette container!",
                            ));
                        }
                    },
                })
            }
        }
    };
}

impl_encoding_for_state_endian!(BigEndian);
impl_encoding_for_state_endian!(NativeEndian);

// SAFETY: This is fine because we uphold all of the invariants
unsafe impl<const N: usize, B: super::bitpack::byteorder::ByteOrderedU64> PaletteContainer
    for StatePaletteContainer<N, B>
{
    const N: usize = N;
    fn new(value: u16) -> Self {
        Self {
            palette: StatePalette::SingleValue(SingleValuePalette(value)),
        }
    }

    fn with_bits(bits: usize, value: u16) -> Self {
        match bits {
            0 => Self::new(value),
            1..=4 => {
                let mut values = Vec::new();
                values.reserve_exact(2usize.pow(4));
                let palette = LinearPalette { bits: 4, values };
                Self {
                    palette: StatePalette::Linear {
                        palette,
                        data: PackedBits::new_unchecked(4),
                    },
                }
            }
            5..=8 => {
                let mut values = Vec::new();
                values.reserve_exact(2usize.pow(bits as u32));
                let palette = LinearPalette { bits, values };
                let palette = MappedPalette {
                    indices: BTreeMap::new(),
                    inner: palette,
                };
                Self {
                    palette: StatePalette::Mapped {
                        palette,
                        data: PackedBits::new_unchecked(bits),
                    },
                }
            }
            _ => Self {
                palette: StatePalette::Global {
                    data: PackedBits::new(bits),
                },
            },
        }
    }

    unsafe fn get_unchecked(&self, i: usize) -> u16 {
        match &self.palette {
            StatePalette::SingleValue(v) => v.0,
            StatePalette::Linear { palette, data } => palette.value(data.get_unchecked(i) as usize),
            StatePalette::Mapped { palette, data } => palette.value(data.get_unchecked(i) as usize),
            StatePalette::Global { data } => data.get_unchecked(i) as u16,
        }
    }

    unsafe fn set_unchecked(&mut self, i: usize, v: u16) {
        loop {
            match &mut self.palette {
                StatePalette::SingleValue(val) => match val.index(v) {
                    IndexOrBits::Index(_) => return,
                    IndexOrBits::Bits(_) => {
                        let mut values = Vec::new();
                        values.reserve_exact(2usize.pow(4));
                        values.push(val.0);
                        let palette = StatePalette::Linear {
                            palette: LinearPalette { bits: 4, values },
                            data: PackedBits::new(4),
                        };
                        self.palette = palette;
                    }
                },
                StatePalette::Linear { palette, data } => match palette.index(v) {
                    IndexOrBits::Index(v) => return data.set(i, v as u32),
                    IndexOrBits::Bits(bits) => {
                        debug_assert_eq!(bits, 5);
                        // We know bits will always be 5
                        data.change_bits(bits);
                        let data = std::mem::take(data);
                        let mut values = std::mem::take(&mut palette.values);
                        // Here we double the capacity so that it is equal to 2 to the power of 5
                        values.reserve_exact(2usize.pow(4)); // values.capacity() should be equal to 2usize.pow(4)
                        let mut indices = BTreeMap::new();
                        for i in 0..values.len() {
                            indices.insert(*values.get_unchecked(i), i);
                        }
                        let palette = StatePalette::Mapped {
                            palette: MappedPalette {
                                indices,
                                inner: LinearPalette { values, bits: 5 },
                            },
                            data,
                        };

                        self.palette = palette;
                    }
                },
                StatePalette::Mapped { palette, data } => match palette.index(v) {
                    IndexOrBits::Index(v) => return data.set_unchecked(i, v as u32),
                    IndexOrBits::Bits(bits) => {
                        let palette: StatePalette<N, B> = if bits == 9 {
                            let mut new_data = PackedBits::new(15);
                            for i in 0..N {
                                //SAFETY: This is fine because the for loop makes sure `i` stays in bounds
                                new_data.set_unchecked(i, self.get_unchecked(i) as u32);
                            }

                            StatePalette::Global { data: new_data }
                        } else {
                            data.change_bits(bits);
                            let data = std::mem::take(data);

                            let linear = LinearPalette {
                                values: std::mem::take(&mut palette.inner.values),
                                bits,
                            };
                            StatePalette::Mapped {
                                palette: MappedPalette {
                                    indices: std::mem::take(&mut palette.indices),
                                    inner: linear,
                                },
                                data,
                            }
                        };
                        self.palette = palette;
                    }
                },
                StatePalette::Global { data } => return data.set_unchecked(i, v.into()),
            }
        }
    }

    fn get(&self, i: usize) -> u16 {
        if i >= N {
            panic!("out of bounds")
        }
        //SAFETY: This is safe because we know i is in bounds.
        unsafe { self.get_unchecked(i) }
    }

    fn set(&mut self, i: usize, v: u16) {
        if i >= N {
            panic!("out of bounds")
        }
        // SAFETY: This is sound because we just checked the bounds
        unsafe { self.set_unchecked(i, v) }
    }

    fn swap(&mut self, i: usize, v: u16) -> u16 {
        if i >= N {
            panic!("out of bounds")
        }
        //SAFETY: This is safe because we just checked the bounds.
        unsafe { self.swap_unchecked(i, v) }
    }

    unsafe fn swap_unchecked(&mut self, i: usize, v: u16) -> u16 {
        let val = self.get_unchecked(i);
        self.set_unchecked(i, v);
        val
    }
}

enum StatePalette<const N: usize, B: super::bitpack::byteorder::ByteOrderedU64> {
    SingleValue(SingleValuePalette),
    Linear {
        palette: LinearPalette,
        data: PackedBits<N, B>,
    },
    Mapped {
        palette: MappedPalette,
        data: PackedBits<N, B>,
    },
    Global {
        data: PackedBits<N, B>,
    },
}

trait Palette {
    fn index(&mut self, value: u16) -> IndexOrBits;
    fn value(&self, index: usize) -> u16;
}

// TODO: Rename?
enum IndexOrBits {
    Index(u64),
    Bits(usize),
}

#[derive(Copy, Clone)]
struct SingleValuePalette(u16);

impl Palette for SingleValuePalette {
    fn index(&mut self, state: u16) -> IndexOrBits {
        if self.0 == state {
            IndexOrBits::Index(0)
        } else {
            IndexOrBits::Bits(1)
        }
    }

    fn value(&self, index: usize) -> u16 {
        if index == 0 {
            self.0
        } else {
            panic!("index out of bounds")
        }
    }
}

struct LinearPalette {
    pub(crate) values: Vec<u16>,
    pub(crate) bits: usize,
}

impl Palette for LinearPalette {
    fn index(&mut self, state: u16) -> IndexOrBits {
        for i in 0..self.values.len() {
            // SAFETY: This is fine because i can only be in bounds due to the for loop.
            unsafe {
                if *self.values.get_unchecked(i) == state {
                    return IndexOrBits::Index(i as u64);
                }
            }
        }

        let len = self.values.len();
        if self.values.capacity() > len {
            debug_assert_eq!(self.values.capacity(), 2usize.pow(self.bits as u32));
            self.values.push(state);
            IndexOrBits::Index(len as u64)
        } else {
            IndexOrBits::Bits(self.bits + 1)
        }
    }

    #[inline]
    fn value(&self, index: usize) -> u16 {
        self.values[index]
    }
}

/// This makes the `index` method faster at the cost of memory usage.
struct MappedPalette {
    pub(crate) indices: BTreeMap<u16, usize>,
    pub(crate) inner: LinearPalette,
}

impl Palette for MappedPalette {
    fn index(&mut self, state: u16) -> IndexOrBits {
        match self.indices.get(&state) {
            Some(v) => IndexOrBits::Index(*v as u64),
            None => {
                let initial_len = self.inner.values.len();
                if self.inner.values.capacity() > initial_len {
                    debug_assert_eq!(
                        self.inner.values.capacity(),
                        2usize.pow(self.inner.bits as u32)
                    );
                    self.inner.values.push(state);
                    self.indices.insert(state, self.inner.values.len());
                    IndexOrBits::Index(initial_len as u64)
                } else {
                    IndexOrBits::Bits(self.inner.bits + 1)
                }
            }
        }
    }

    fn value(&self, index: usize) -> u16 {
        self.inner.value(index)
    }
}

#[cfg(test)]
mod tests {
    use super::super::bitpack::byteorder;
    use super::{BiomePaletteContainer, PaletteContainer, StatePaletteContainer};

    #[test]
    fn state() {
        let mut data = Vec::new();
        for i in 0..512 {
            data.push(i)
        }
        data.reverse();
        let mut container = StatePaletteContainer::<512, byteorder::NativeEndian>::new(0);
        for i in 0..512 {
            container.set(i, data[i]);
            assert_eq!(container.get(i), data[i]);
            for j in 0..=i {
                assert_eq!(container.get(j), data[j])
            }
        }
    }

    #[test]
    fn biome() {
        let data = vec![7, 6, 5, 4, 3, 2, 1, 0];
        let mut container = BiomePaletteContainer::<8, byteorder::NativeEndian>::new(0);
        for i in 0..8 {
            container.set(i, data[i]);
            assert_eq!(container.get(i), data[i]);
            for j in 0..=i {
                assert_eq!(container.get(j), data[j])
            }
        }
    }
}
