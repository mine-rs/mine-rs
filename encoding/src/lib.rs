#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_self_imports
)]

#[macro_use]
extern crate thiserror;

use std::io::{Cursor, Read, Write};

pub mod attrs;

pub mod separated;
pub use separated::*;

pub mod decode;
pub mod encode;

pub use decode::Decode;
pub use encode::Encode;
