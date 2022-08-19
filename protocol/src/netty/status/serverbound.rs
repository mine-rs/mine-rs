use std::borrow::Cow;

use crate::netty::{ProtocolRead, ProtocolWrite, ReadError, WriteError};
use protocol_derive::Protocol;

#[derive(Protocol)]
struct Request0 {}

pub use super::Ping0;