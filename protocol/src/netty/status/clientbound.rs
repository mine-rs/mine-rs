use std::borrow::Cow;

use crate::netty::{ProtocolRead, ProtocolWrite, ReadError, WriteError};
use protocol_derive::Protocol;

#[derive(Protocol)]
struct Response<'a> {
    // todo! json thing
    data: Cow<'a, str>
}

pub use super::Ping0;