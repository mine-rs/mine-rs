use crate::*;
use attrs::*;

use std::{borrow::Cow, str::FromStr};
use uuid::Uuid;

#[derive(Encoding, ToStatic)]
pub struct Disconnect0<'a> {
    pub reason: Cow<'a, str>,
}

#[derive(Encoding, ToStatic)]
pub struct EncryptionResponse0<'a> {
    pub server_id: Cow<'a, str>,
    #[counted(u16)]
    pub public_key: Cow<'a, [u8]>,
    #[counted(u16)]
    pub verify_token: Cow<'a, [u8]>,
}

#[derive(Encoding, ToStatic)]
pub struct Success0<'a> {
    #[stringuuid]
    pub uuid: Uuid,
    pub username: Cow<'a, str>,
}

pub struct Success5<'a> {
    pub uuid: Option<Uuid>,
    pub username: Cow<'a, str>,
}

impl<'dec> Decode<'dec> for Success5<'dec> {
    fn decode(buf: &mut std::io::Cursor<&'dec [u8]>) -> decode::Result<Self> {
        let uuid = <&str as Decode>::decode(buf)?;

        Ok(Self {
            uuid: if !uuid.is_empty() {
                Some(Uuid::from_str(uuid)?)
            } else {
                None
            },
            username: Decode::decode(buf)?,
        })
    }
}
impl<'a> Encode for Success5<'a> {
    fn encode(&self, buf: &mut impl ::std::io::Write) -> encode::Result<()> {
        let Self { uuid, username } = self;
        if let Some(uuid) = uuid {
            StringUuid::from(*uuid).encode(buf)?;
        } else {
            "".encode(buf)?;
        }
        username.encode(buf)?;
        Ok(())
    }
}
