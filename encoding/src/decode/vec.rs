use crate::*;
use attrs::Var;

impl<'dec, T: Decode<'dec>> Decode<'dec> for Vec<T> {
    fn decode(cursor: &mut Cursor<&'dec [u8]>) -> decode::Result<Self> {
        let Var(count) = <Var<u32>>::decode(cursor)?;
        (0..count).map(|_| T::decode(cursor)).collect()
    }
}
