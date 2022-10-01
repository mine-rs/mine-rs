pub mod compound;
pub mod list;
pub mod macros;
pub mod tag;
pub mod value;

pub use compound::Compound;
pub use list::List;
pub use tag::NbtTag;
pub use value::Value;

pub(crate) use miners_encoding::attrs::{Counted, Mutf8};
pub(crate) use miners_encoding::{decode, Decode, Encode};
#[cfg(feature = "to_static")]
pub(crate) use miners_to_static::ToStatic;
pub(crate) use std::{borrow::Cow, collections::HashMap, hint::unreachable_unchecked};
