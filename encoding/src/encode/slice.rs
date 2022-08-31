use crate::*;
use attrs::Var;

impl Encode for [u8] {
    fn encode(&self, writer: &mut impl Write) -> encode::Result<()> {
        Var(self.len() as u32).encode(writer)?;
        writer.write_all(self)?;
        Ok(())
    }
}
