#[derive(Debug, Clone, Copy)]
pub enum NbtTag {
    End = 0,
    Byte = 1,
    Short = 2,
    Int = 3,
    Long = 4,
    Float = 5,
    Double = 6,
    ByteArray = 7,
    String = 8,
    List = 9,
    Compound = 10,
    IntArray = 11,
    LongArray = 12,
}
pub struct InvalidNbtTagByte;
impl From<InvalidNbtTagByte> for ::miners_encoding::decode::Error {
    fn from(_: InvalidNbtTagByte) -> Self {
        ::miners_encoding::decode::Error::InvalidId
    }
}
impl TryFrom<u8> for NbtTag {
    type Error = InvalidNbtTagByte;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use NbtTag::*;
        Ok(match value {
            0 => End,
            1 => Byte,
            2 => Short,
            3 => Int,
            4 => Long,
            5 => Float,
            6 => Double,
            7 => ByteArray,
            8 => String,
            9 => List,
            10 => Compound,
            11 => IntArray,
            12 => LongArray,
            _ => return Err(InvalidNbtTagByte),
        })
    }
}
impl<'dec> ::miners_encoding::Decode<'dec> for NbtTag {
    fn decode(cursor: &mut std::io::Cursor<&'dec [u8]>) -> ::miners_encoding::decode::Result<Self> {
        u8::decode(cursor)?.try_into().map_err(Into::into)
    }
}
impl ::miners_encoding::Encode for NbtTag {
    fn encode(&self, writer: &mut impl std::io::Write) -> ::miners_encoding::encode::Result<()> {
        (*self as u8).encode(writer)
    }
}
