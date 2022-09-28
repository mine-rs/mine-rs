use crate::*;

impl<T> Encode for Option<T>
where
    T: Encode,
{
    fn encode(&self, writer: &mut impl Write) -> encode::Result<()> {
        match self {
            Some(t) => {
                true.encode(writer)?;
                t.encode(writer)
            }
            None => false.encode(writer),
        }
    }
}
