use crate::*;
use attrs::Var;

impl<T: Encode> Encode for Vec<T> {
    fn encode(&self, writer: &mut impl Write) -> encode::Result<()> {
        Var(self.len() as u32).encode(writer)?;
        self.iter().try_for_each(|t| t.encode(writer))
    }
}
