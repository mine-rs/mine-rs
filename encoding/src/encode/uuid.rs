use crate::*;
use uuid::Uuid;

impl Encode for Uuid {
    fn encode(&self, writer: &mut impl Write) -> encode::Result<()> {
        self.as_u128().encode(writer)
    }
}
