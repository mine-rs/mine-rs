use ::miners_encoding::{attrs::StringUuid, decode, encode, Decode, Encode};

use std::{borrow::Cow, str::FromStr};
use uuid::Uuid;

#[derive(Encoding, ToStatic)]
pub struct Disconnect0<'a> {
    // chat most likely, for sure starting pv13
    pub reason: Cow<'a, str>,
}

#[derive(Encoding, ToStatic)]
pub struct EncryptionRequest0<'a> {
    pub server_id: Cow<'a, str>,
    #[encoding(counted = "u16")]
    pub public_key: Cow<'a, [u8]>,
    #[encoding(counted = "u16")]
    pub verify_token: Cow<'a, [u8]>,
}

#[derive(Encoding, ToStatic)]
pub struct EncryptionRequest19<'a> {
    pub server_id: Cow<'a, str>,
    pub public_key: Cow<'a, [u8]>,
    pub verify_token: Cow<'a, [u8]>,
}

#[derive(Encoding, ToStatic)]
pub struct Success0<'a> {
    pub uuid: StringUuid,
    pub username: Cow<'a, str>,
}

#[derive(ToStatic)]
pub struct Success5<'a> {
    // stringuuid
    pub uuid: Option<Uuid>,
    pub username: Cow<'a, str>,
}

impl<'dec: 'a, 'a> Decode<'dec> for Success5<'a> {
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

#[derive(Encoding, ToStatic)]
pub struct SetCompression27 {
    #[encoding(varint)]
    pub threshold: i32,
}
