use std::borrow::Cow;

use crate::netty::{ProtocolRead, ProtocolWrite, ReadError, WriteError};
use protocol_derive::Protocol;
use uuid::Uuid;

#[derive(Protocol)]
struct Disconnect0<'a> {
    reason: Cow<'a, str>,
}

#[derive(Protocol)]
struct EncryptionResponse0<'a> {
    server_id: Cow<'a, str>,
    // #[count(u16)]
    public_key: Cow<'a, [u8]>,
    // #[count(u16)]
    verify_token: Cow<'a, [u8]>,
}

#[derive(Protocol)]
struct Success0<'a> {
    uuid: Uuid,
    username: Cow<'a, str>,
}
