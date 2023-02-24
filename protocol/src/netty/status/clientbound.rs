use std::borrow::Cow;

#[derive(Encoding, ToStatic)]
pub struct Response0<'a> {
    // TODO: json thing
    pub data: Cow<'a, str>,
}

#[derive(Encoding, ToStatic)]
pub struct Ping0 {
    pub time: i64,
}
