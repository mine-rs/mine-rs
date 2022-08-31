#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::undocumented_unsafe_blocks
)]

#[macro_use]
extern crate miners_packets_derive;
#[macro_use]
extern crate miners_encoding_derive;
use miners_encoding::*;

pub mod netty;
pub use miners_packets_traits::ToStatic;
