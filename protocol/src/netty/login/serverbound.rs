use crate::*;
use attrs::*;

use std::borrow::Cow;

#[derive(Encoding, ToStatic)]
pub struct LoginStart0<'a> {
    pub username: Cow<'a, str>,
}

#[derive(Encoding, ToStatic)]
pub struct EncryptionRequest0<'a> {
    #[counted(u16)]
    pub public_key: Cow<'a, [u8]>,
    #[counted(u16)]
    pub verify_token: Cow<'a, [u8]>,
}
