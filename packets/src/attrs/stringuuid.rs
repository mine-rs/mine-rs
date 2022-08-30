use crate::ToStatic;

use super::*;
use std::str::FromStr;

#[repr(transparent)]
pub struct StringUuid(pub uuid::Uuid);

impl ProtocolRead<'_> for StringUuid {
    fn read(cursor: &mut std::io::Cursor<&'_ [u8]>) -> Result<Self, ReadError> {
        let s = <&str>::read(cursor)?;
        Ok(StringUuid(uuid::Uuid::from_str(s)?))
    }
}

impl ProtocolWrite for StringUuid {
    fn write(self, writer: &mut impl std::io::Write) -> Result<(), WriteError> {
        let mut buffer = [0u8; uuid::fmt::Hyphenated::LENGTH];
        self.0.hyphenated().encode_lower(&mut buffer);
        writer.write_all(&[uuid::fmt::Hyphenated::LENGTH as u8])?;
        writer.write_all(&buffer)?;
        Ok(())
    }

    fn size_hint() -> usize {
        uuid::fmt::Hyphenated::LENGTH
    }
}

// todo! move this when we get to implementing protocolwrite and protocolread
// for uuid::Uuid in crate::impls
impl ToStatic for uuid::Uuid {
    type Static = uuid::Uuid;
    fn to_static(&self) -> Self::Static {
        *self
    }
    fn into_static(self) -> Self::Static {
        self
    }
}
