mod to_static;
use miners_encoding::{Decode, Encode};
pub use to_static::ToStatic;

pub trait Packet<'dec>: Encode + Decode<'dec> + ToStatic {
    fn id_for_version(version: i32) -> Option<i32>;
}
