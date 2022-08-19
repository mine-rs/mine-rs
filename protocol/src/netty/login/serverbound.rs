use std::borrow::Cow;

use crate::netty::{ProtocolRead, ProtocolWrite, ReadError, WriteError};
use protocol_derive::Protocol;

#[derive(Protocol)]
struct LoginStart0<'a> {
    username: Cow<'a, str>,
}

#[derive(Protocol)]
struct EncryptionRequest0<'a> {
    // #[count(u16)]
    public_key: Cow<'a, [u8]>,
    // #[count(u16)]
    verify_token: Cow<'a, [u8]>,
}