use miners::encoding::{Decode, Encode};

/// # Safety
/// This trait is safe to implement as long as the struct has the same data layout as `u64`
/// This trait should not be implemented for a type other than `BigEndian` or `NativeEndian`.
pub unsafe trait ByteOrderedU64: Copy + Clone + Default /*+ Encode + for<'a> Decode<'a>*/ {
    // used for using the value internally
    fn to_ne(self) -> u64;
    fn from_ne(v: u64) -> Self;
}

#[repr(transparent)]
#[derive(Clone, Copy, Default)]
pub struct BigEndian(u64);

impl BigEndian {
    pub const ZERO: BigEndian = Self(0);
}

//impl Encode for BigEndian {
//    fn encode(&self, writer: &mut impl std::io::Write) -> miners::encoding::encode::Result<()> {
//        // The data is already in big endian regardless of the endianness of the system so this just preserves the byte order.
//        writer.write_all(&self.0.to_ne_bytes())?;
//        Ok(())
//    }
//}
//
//impl<'dec> Decode<'dec> for BigEndian {
//    fn decode(cursor: &mut std::io::Cursor<&'dec [u8]>) -> miners::encoding::decode::Result<Self> {
//        let mut bytes = [0u8; 8];
//        cursor.read_exact(&mut bytes)?;
//        Ok(Self(u64::from_ne_bytes(bytes)))
//    }
//}

// SAFETY: This is fine because the struct is `repr(transparent)`
unsafe impl ByteOrderedU64 for BigEndian {
    fn to_ne(self) -> u64 {
        // This swaps the bytes on little endian, converting it into little endian even though the function is named `to_be`
        self.0.to_be()
    }

    fn from_ne(v: u64) -> Self {
        Self(v.to_be())
    }
}
#[repr(transparent)]
#[derive(Clone, Copy, Default)]
pub struct NativeEndian(u64);

impl Encode for NativeEndian {
    fn encode(&self, writer: &mut impl std::io::Write) -> miners::encoding::encode::Result<()> {
        u64::encode(&self.0, writer)
    }
}

impl<'dec> Decode<'dec> for NativeEndian {
    fn decode(cursor: &mut std::io::Cursor<&'dec [u8]>) -> miners::encoding::decode::Result<Self> {
        Ok(Self(u64::decode(cursor)?))
    }
}

// SAFETY: This is fine because the struct is `repr(transparent)`
unsafe impl ByteOrderedU64 for NativeEndian {
    fn to_ne(self) -> u64 {
        self.0
    }
    fn from_ne(v: u64) -> Self {
        Self(v)
    }
}
