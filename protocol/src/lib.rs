#[macro_use]
extern crate miners_protocol_derive;
pub use miners_protocol_derive::{replace, ToStatic};

#[macro_use]
extern crate miners_encoding_derive;

use miners_encoding::*;
pub use miners_packet::*;

pub mod netty;
