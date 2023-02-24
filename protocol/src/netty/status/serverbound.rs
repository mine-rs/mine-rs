#[derive(Encoding, ToStatic)]
pub struct Request0 {}

#[derive(Encoding, ToStatic)]
pub struct Ping0 {
    pub time: i64,
}
