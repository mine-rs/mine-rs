#[macro_use]
extern crate miners_protocol_derive;
pub use miners_protocol_derive::replace;
pub use miners_to_static::ToStatic;
pub use miners_to_static_derive::ToStatic;

#[macro_use]
extern crate miners_encoding_derive;

use miners_encoding::*;
pub use miners_packet::*;

pub mod netty;
