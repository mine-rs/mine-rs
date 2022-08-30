use crate::*;

use packets_derive::Protocol;

#[derive(Protocol)]
pub struct Request0 {}

pub use super::Ping0;
