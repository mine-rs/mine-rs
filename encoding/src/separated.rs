use crate::*;

pub trait SeparatedDecode<'dec>: Sized {
    type Part;
    fn decode_with_part(part: Self::Part, cursor: &mut Cursor<&'dec [u8]>) -> crate::decode::Result<Self>;
}

pub trait SeparatedEncode {
    type Part;
    fn encode_part(&self, writer: &mut impl Write) -> crate::encode::Result<()>;
    fn encode_rest(&self, writer: &mut impl Write) -> crate::encode::Result<()>;
}
