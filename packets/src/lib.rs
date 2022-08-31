#![deny(clippy::unwrap_used, clippy::expect_used)]

#[macro_use]
extern crate miners_packets_derive;
use miners_encoding::*;

pub mod netty;
pub use miners_packets_traits::ToStatic;
