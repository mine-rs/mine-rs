use crate::attrs::*;
use crate::*;

use protocol_derive::Protocol;
use std::{borrow::Cow, str::FromStr};
use uuid::Uuid;

#[derive(Protocol)]
pub struct Disconnect0<'a> {
    pub reason: Cow<'a, str>,
}

#[derive(Protocol)]
pub struct EncryptionResponse0<'a> {
    pub server_id: Cow<'a, str>,
    #[count(u16)]
    pub public_key: Cow<'a, [u8]>,
    #[count(u16)]
    pub verify_token: Cow<'a, [u8]>,
}

#[derive(Protocol)]
pub struct Success0<'a> {
    #[stringuuid]
    pub uuid: Uuid,
    pub username: Cow<'a, str>,
}

pub struct Success5<'a> {
    pub uuid: Option<Uuid>,
    pub username: Cow<'a, str>,
}

impl<'read, 'a> ProtocolRead<'read> for Success5<'a>
where
    'read: 'a,
{
    fn read(buf: &mut std::io::Cursor<&'read [u8]>) -> Result<Self, ReadError> {
        let uuid = <&str as ProtocolRead>::read(buf)?;

        Ok(Self {
            uuid: if !uuid.is_empty() {
                Some(Uuid::from_str(uuid)?)
            } else {
                None
            },
            username: ProtocolRead::read(buf)?,
        })
    }
}
impl<'a> ProtocolWrite for Success5<'a> {
    fn write(self, buf: &mut impl ::std::io::Write) -> Result<(), WriteError> {
        let Self { uuid, username } = self;
        if let Some(uuid) = uuid {
            ProtocolWrite::write(StringUuid(uuid), buf)?;
        } else {
            "".write(buf)?;
        }
        ProtocolWrite::write(username, buf)?;
        Ok(())
    }
    #[inline(always)]
    fn size_hint() -> usize {
        1 + <Cow<'a, str> as ProtocolWrite>::size_hint()
    }
}
