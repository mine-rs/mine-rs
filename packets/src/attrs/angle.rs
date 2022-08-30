use crate::*;

pub struct Angle(u8);

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

impl ProtocolRead<'_> for Angle {
    fn read(cursor: &'_ mut std::io::Cursor<&[u8]>) -> Result<Self, ReadError> {
        u8::read(cursor).map(Self)
    }
}
impl ProtocolWrite for Angle {
    fn write(self, writer: &mut impl std::io::Write) -> Result<(), WriteError> {
        self.0.write(writer)
    }
    fn size_hint() -> usize {
        1
    }
}
impl ToStatic for Angle {
    type Static = Angle;
    fn to_static(&self) -> Self::Static {
        Angle(self.0)
    }
    fn into_static(self) -> Self::Static {
        self
    }
}