use crate::attrs::*;
use crate::*;

use packets_derive::Protocol;
use std::borrow::Cow;

#[derive(Protocol)]
pub struct LoginStart0<'a> {
    pub username: Cow<'a, str>,
}

#[derive(Protocol)]
pub struct EncryptionRequest0<'a> {
    #[count(u16)]
    pub public_key: Cow<'a, [u8]>,
    #[count(u16)]
    pub verify_token: Cow<'a, [u8]>,
}
