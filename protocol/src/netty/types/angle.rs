use crate::*;
use std::io::{Cursor, Write};

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Angle(pub(crate) u8);

impl ToStatic for Angle {
    type Static = Angle;
    fn to_static(&self) -> Self::Static {
        *self
    }
    fn into_static(self) -> Self::Static {
        self
    }
}

impl From<u8> for Angle {
    fn from(angle: u8) -> Self {
        Angle(angle)
    }
}

impl Angle {
    pub fn into_inner(self) -> u8 {
        self.0
    }
}

impl AsRef<u8> for Angle {
    fn as_ref(&self) -> &u8 {
        &self.0
    }
}

impl Angle {
    pub fn from_rad(radians: f32) -> Self {
        Self((radians * std::f32::consts::PI / 180.0 * 256.0) as u8)
    }
    pub fn rad(&self) -> f32 {
        self.0 as f32 / 256.0 * 180.0 / std::f32::consts::PI
    }
    pub fn from_deg(degrees: f32) -> Self {
        Self((degrees / 360.0 * 256.0) as u8)
    }
    pub fn deg(&self) -> f32 {
        self.0 as f32 / 256.0 * 360.0
    }
}

impl<'dec> Decode<'dec> for Angle {
    fn decode(cursor: &mut Cursor<&'dec [u8]>) -> decode::Result<Self> {
        u8::decode(cursor).map(Self)
    }
}
impl Encode for Angle {
    fn encode(&self, writer: &mut impl Write) -> encode::Result<()> {
        self.0.encode(writer)
    }
}

// TODO: add tests
