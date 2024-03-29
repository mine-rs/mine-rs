use crate::*;
use uuid::Uuid;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StringUuid(pub(crate) Option<Uuid>);

impl From<Uuid> for StringUuid {
    fn from(uuid: Uuid) -> Self {
        StringUuid(Some(uuid))
    }
}

impl StringUuid {
    pub fn into_inner(self) -> Option<Uuid> {
        self.0
    }
}

#[cfg(feature = "to_static")]
impl miners_to_static::ToStatic for StringUuid {
    type Static = StringUuid;
    fn to_static(&self) -> Self::Static {
        self.clone()
    }
    fn into_static(self) -> Self::Static {
        self
    }
}

impl<'dec> Decode<'dec> for StringUuid {
    fn decode(cursor: &mut Cursor<&'dec [u8]>) -> decode::Result<Self> {
        let string = <&str>::decode(cursor)?;
        if !string.is_empty() {
            Ok(StringUuid(Some(uuid::Uuid::parse_str(string)?)))
        } else {
            Ok(StringUuid(None))
        }
    }
}

impl Encode for StringUuid {
    fn encode(&self, writer: &mut impl Write) -> encode::Result<()> {
        match self.0 {
            Some(uuid) => {
                let mut buffer = [0u8; uuid::fmt::Hyphenated::LENGTH];
                uuid.hyphenated().encode_lower(&mut buffer);
                writer.write_all(&[uuid::fmt::Hyphenated::LENGTH as u8])?;
                writer.write_all(&buffer)?;
                Ok(())
            }
            None => "".encode(writer),
        }
    }
}

// TODO: add tests
