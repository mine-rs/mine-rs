use crate::*;

impl Encode for bool {
    fn encode(&self, writer: &mut impl Write) -> encode::Result<()> {
        (*self as u8).encode(writer)
    }
}
