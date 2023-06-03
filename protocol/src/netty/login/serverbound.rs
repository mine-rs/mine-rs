use std::borrow::Cow;

#[derive(Encoding, ToStatic, Debug)]
pub struct LoginStart0<'a> {
    pub username: Cow<'a, str>,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct EncryptionResponse0<'a> {
    #[encoding(counted = "u16")]
    pub secret: Cow<'a, [u8]>,
    #[encoding(counted = "u16")]
    pub verify_token: Cow<'a, [u8]>,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct EncryptionResponse19<'a> {
    pub secret: Cow<'a, [u8]>,
    pub verify_token: Cow<'a, [u8]>,
}
