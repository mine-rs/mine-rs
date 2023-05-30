use std::borrow::Cow;

#[derive(Encoding, ToStatic)]
pub struct LoginStart0<'a> {
    pub username: Cow<'a, str>,
}

#[derive(Encoding, ToStatic)]
pub struct EncryptionResponse0<'a> {
    pub server_id: Cow<'a, str>,
    #[encoding(counted = "u16")]
    pub public_key: Cow<'a, [u8]>,
    #[encoding(counted = "u16")]
    pub verify_token: Cow<'a, [u8]>,
}

#[derive(Encoding, ToStatic)]
pub struct EncryptionResponse19<'a> {
    pub server_id: Cow<'a, str>,
    pub public_key: Cow<'a, [u8]>,
    pub verify_token: Cow<'a, [u8]>,
}
