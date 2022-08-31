use crate::*;
use uuid::Uuid;

pub struct StringUuid(pub(crate) Uuid);

impl From<Uuid> for StringUuid {
    fn from(uuid: Uuid) -> Self {
        StringUuid(uuid)
    }
}

impl StringUuid {
    pub fn into_inner(self) -> Uuid {
        self.0
    }
}

impl<'dec> Decode<'dec> for StringUuid {
    fn decode(cursor: &mut Cursor<&'dec [u8]>) -> decode::Result<Self> {
        let string = <&str>::decode(cursor)?;
        Ok(StringUuid(uuid::Uuid::parse_str(string)?))
    }
}

impl Encode for StringUuid {
    fn encode(&self, writer: &mut impl Write) -> encode::Result<()> {
        let mut buffer = [0u8; uuid::fmt::Hyphenated::LENGTH];
        self.0.hyphenated().encode_lower(&mut buffer);
        writer.write_all(&[uuid::fmt::Hyphenated::LENGTH as u8])?;
        writer.write_all(&buffer)?;
        Ok(())
    }
}

// todo! add tests
