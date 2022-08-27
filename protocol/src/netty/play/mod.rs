use crate::errors::InvalidEnumId;
use crate::*;

use protocol_derive::Protocol;
use std::borrow::Cow;

pub mod clientbound;
pub mod serverbound;

#[derive(Protocol)]
#[from(u8)]
pub enum AnimationId0 {
    None = 0,
    SwingArm,
    Damage,
    LeaveBed,
    EatFood,
    Crit,
    MagicCrit,
    Unknown = 102,
    Crouch,
    Uncrouch,
}

pub struct PlayerAbilities0 {
    pub invulnerable: bool,
    pub flying: bool,
    pub allow_flying: bool,
    pub creative_mode: bool,
    pub flying_speed: f32,
    /// Modifies the field of view, like a speed potion. A Notchian server will
    /// use the same value as the movement speed (send in the Entity Properties
    /// packet).
    pub fov: f32,
}
impl ProtocolRead<'_> for PlayerAbilities0 {
    fn read(cursor: &'_ mut std::io::Cursor<&[u8]>) -> Result<Self, ReadError> {
        let flags = u8::read(cursor)?;
        Ok(PlayerAbilities0 {
            invulnerable: flags & 0b0001 != 0,
            flying: flags & 0b0010 != 0,
            allow_flying: flags & 0b0100 != 0,
            creative_mode: flags & 0b1000 != 0,
            flying_speed: f32::read(cursor)?,
            fov: f32::read(cursor)?,
        })
    }
}
impl ProtocolWrite for PlayerAbilities0 {
    fn write(self, writer: &mut impl std::io::Write) -> Result<(), WriteError> {
        ((self.invulnerable as u8)
            + ((self.flying as u8) << 1)
            + ((self.allow_flying as u8) << 2)
            + ((self.creative_mode as u8) << 3))
            .write(writer)?;
        self.flying_speed.write(writer)?;
        self.fov.write(writer)?;
        Ok(())
    }

    fn size_hint() -> usize {
        9
    }
}

#[derive(Protocol)]
#[from(u8)]
pub enum Difficulty0 {
    Peaceful = 0,
    Easy,
    Normal,
    Hard,
}

#[derive(Protocol)]
// https://dinnerbone.com/blog/2012/01/13/minecraft-plugin-channels-messaging/
pub struct PluginMessage0<'a> {
    pub channel: Cow<'a, str>,
    pub data: Cow<'a, [u8]>,
}
