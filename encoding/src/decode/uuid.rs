use crate::*;
use uuid::Uuid;

impl<'dec> Decode<'dec> for Uuid {
    fn decode(cursor: &mut Cursor<&'dec [u8]>) -> decode::Result<Self> {
        u128::decode(cursor).map(Uuid::from_u128)
    }
}
