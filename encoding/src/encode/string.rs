use crate::*;

impl Encode for String {
    fn encode(&self, writer: &mut impl Write) -> encode::Result<()> {
        self.as_bytes().encode(writer)
    }
}
