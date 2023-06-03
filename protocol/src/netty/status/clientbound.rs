use std::borrow::Cow;

#[derive(Encoding, ToStatic, Debug)]
pub struct Response0<'a> {
    // TODO: json thing
    pub data: Cow<'a, str>,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct Ping0 {
    pub time: i64,
}
