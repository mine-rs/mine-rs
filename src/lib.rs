//! # mine-rs
//!
//! A project aimed towards providing tools necessary to implement Clients and
//! Servers for Minecraft.
#[cfg(feature = "auth")]
pub use miners_auth as auth;
#[cfg(feature = "chat")]
pub use miners_chat as chat;
#[cfg(feature = "nbt")]
pub use miners_nbt as nbt;
#[cfg(feature = "net")]
pub use miners_net as net;
#[cfg(feature = "packet")]
pub use miners_packet as packet;
#[cfg(feature = "protocol")]
pub use miners_protocol as protocol;
#[cfg(feature = "encoding")]
#[doc(alias = "miners_encoding")]
pub mod encoding {
    pub use miners_encoding::*;
    #[cfg(feature = "encoding_derive")]
    pub use miners_encoding_derive::*;
}
#[cfg(feature = "to_static")]
#[doc(alias = "miners_to_static")]
pub mod to_static {
    pub use miners_to_static::*;
    #[cfg(feature = "to_static_derive")]
    pub use miners_to_static_derive::*;
}
