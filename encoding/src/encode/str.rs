use crate::*;

impl Encode for str {
    fn encode(&self, writer: &mut impl Write) -> encode::Result<()> {
        self.as_bytes().encode(writer)
    }
}
