#[derive(Encoding, ToStatic, Debug)]
pub struct Request0 {}

#[derive(Encoding, ToStatic, Debug)]
pub struct Ping0 {
    pub time: i64,
}
