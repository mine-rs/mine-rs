use std::borrow::Cow;

use crate::*;
use attrs::*;

use super::position::Position442;

macro_rules! particle_versions {
    ($(
        $(#[$($attr:tt)*])*
        $particle:ident =>  {
            $(Block => $block_id:literal $(,)?)?
            $(BlockMarker => $block_marker_id:literal $(,)?)?
            $(Dust => $dust_id:literal $(,)?)?
            $(DustColorTransition => $dust_color_transition_id:literal $(,)?)?
            $(FallingDust => $falling_dust_id:literal $(,)?)?
            $(Item => $item_id:literal $(,)?)?
            $(Vibration => $vibration_id:literal $(,)?)?
        }
    ),* $(,)?) => {$(
        #[derive(ToStatic)]
        $(#[$($attr)*])*
        pub struct $particle<'a, Slot> {
            id: i32,
            data: Option<ParticleData<'a, Slot>>,
        }
        impl<'dec: 'a, 'a, Slot> Decode<'dec> for $particle<'a, Slot>
        where
            Slot: Decode<'dec>,
        {
            fn decode(cursor: &mut std::io::Cursor<&'dec [u8]>) -> decode::Result<Self> {
                let id = Var::decode(cursor)?.into_inner();
                let data = match id {
                    $($block_id => Some(ParticleData::Block(Var::decode(cursor)?.into_inner())),)?
                    $($block_marker_id => Some(ParticleData::BlockMarker(Var::decode(cursor)?.into_inner())),)?
                    $($dust_id => Some(ParticleData::Dust {
                        color: Color::decode(cursor)?,
                        scale: f32::decode(cursor)?,
                    }),)?
                    $($dust_color_transition_id => Some(ParticleData::DustColorTransition {
                        from: Color::decode(cursor)?,
                        scale: f32::decode(cursor)?,
                        to: Color::decode(cursor)?,
                    }),)?
                    $($falling_dust_id => Some(ParticleData::FallingDust(Var::decode(cursor)?.into_inner())),)?
                    $($item_id => Some(ParticleData::Item(Slot::decode(cursor)?)),)?
                    $($vibration_id => Some(ParticleData::Vibration(Vibration::decode(cursor)?)),)?
                    _ => None,
                };
                Ok($particle { id, data })
            }
        }

        impl<'a, Slot> Encode for $particle<'a, Slot>
        where
            Slot: Encode,
        {
            fn encode(&self, writer: &mut impl std::io::Write) -> encode::Result<()> {
                Var::from(self.id).encode(writer)?;
                #[cfg(debug_assertions)]
                match self.id {
                    $($block_id if !matches!(self.data, Some(ParticleData::Block(_))) => {
                        return Err(encode::Error::Custom(
                            "id for Block without Block ParticleData",
                        ));
                    })?
                    $($block_marker_id if !matches!(self.data, Some(ParticleData::BlockMarker(_))) => {
                        return Err(encode::Error::Custom(
                            "id for BlockMarker without BlockMarker ParticleData",
                        ))
                    })?
                    $($dust_id if !matches!(self.data, Some(ParticleData::Dust { .. })) => {
                        return Err(encode::Error::Custom(
                            "id for Dust without Dust ParticleData",
                        ));
                    })?
                    $($dust_color_transition_id if !matches!(self.data, Some(ParticleData::DustColorTransition { .. })) => {
                        return Err(encode::Error::Custom(
                            "id for DustColorTransition without DustColorTransition ParticleData"
                        ));
                    })?
                    $($falling_dust_id if !matches!(self.data, Some(ParticleData::FallingDust(_))) => {
                        return Err(encode::Error::Custom(
                            "id for FallingDust without FallingDust ParticleData",
                        ));
                    })?
                    $($item_id if !matches!(self.data, Some(ParticleData::Item(_))) => {
                        return Err(encode::Error::Custom(
                            "id for Item without Item ParticleData",
                        ));
                    })?
                    $($vibration_id if !matches!(self.data, Some(ParticleData::Vibration(_))) => {
                        return Err(encode::Error::Custom(
                            "id for Vibration without Vibration ParticleData"
                        ))
                    })?
                    _ if self.data.is_some() => {
                        return Err(encode::Error::Custom(
                            "specified additional data for dataless Particle id",
                        ));
                    }
                    _ => {}
                }
                if let Some(data) = &self.data {
                    match data {
                        ParticleData::Block(block) => Var::from(*block).encode(writer)?,
                        ParticleData::BlockMarker(block) => Var::from(*block).encode(writer)?,
                        ParticleData::Dust { color, scale } => {
                            color.encode(writer)?;
                            scale.encode(writer)?;
                        }
                        ParticleData::DustColorTransition { from, scale, to } => {
                            from.encode(writer)?;
                            scale.encode(writer)?;
                            to.encode(writer)?;
                        }
                        ParticleData::FallingDust(block) => Var::from(*block).encode(writer)?,
                        ParticleData::Item(slot) => slot.encode(writer)?,
                        ParticleData::Vibration(vibration) => vibration.encode(writer)?,
                    }
                }
                Ok(())
            }
        }
    )*};
}

particle_versions! {
    Particle353 => {
        Block => 3,
        Dust => 11,
        FallingDust => 20,
        Item => 27
    },
    Particle463 => {
        Block => 3,
        Dust => 14,
        FallingDust => 23,
        Item => 30
    },
    Particle701 => {
        Block => 3,
        Dust => 14,
        FallingDust => 23,
        Item => 31
    },
    /// Applies to snapshot versions 1..=7, 10
    Particle706 => {
        Block => 3,
        Dust => 14,
        FallingDust => 23,
        Item => 32
    },
    /// Applies to snapshot versions 8, 9 and 11..=19
    Particle755 => {
        Block => 3,
        Dust => 14,
        DustColorTransition => 15,
        FallingDust => 24,
        Item => 35,
        Vibration => 36
    },
    /// Applies to snapshot versions 20..=46
    ParticleS20 => {
        Block => 4,
        Dust => 15,
        DustColorTransition => 16,
        FallingDust => 25,
        Item => 36,
        Vibration => 37
    },
    /// Applies to snapshot versions 47..=73
    Particle757 => {
        Block => 2,
        BlockMarker => 3,
        Dust => 14,
        DustColorTransition => 15,
        FallingDust => 24,
        Item => 35,
        Vibration => 36
    },
    /// Applies to snapshot versions 74..=78
    ParticleS74 => {
        Block => 2,
        BlockMarker => 3,
        Dust => 14,
        DustColorTransition => 15,
        FallingDust => 24,
        Item => 38,
        Vibration => 39
    },
    /// Applies to snapshot versions 79..
    Particle759 => {
        Block => 2,
        BlockMarker => 3,
        Dust => 14,
        DustColorTransition => 15,
        FallingDust => 25,
        Item => 39,
        Vibration => 40
    }
}

#[derive(ToStatic)]
pub enum ParticleData<'a, Slot> {
    /// The ID of the block state.
    Block(i32),
    BlockMarker(i32),
    Dust {
        color: Color,
        /// The scale, will be clamped between 0.01 and 4.
        scale: f32,
    },
    DustColorTransition {
        from: Color,
        /// The scale, will be clamped between 0.01 and 4.
        scale: f32,
        to: Color,
    },
    /// The ID of the block state.
    FallingDust(i32),
    Item(Slot),
    Vibration(Vibration<'a>),
}

#[derive(Encoding, ToStatic)]
pub struct Vibration<'a> {
    source: VibrationSource<'a>,
    /// The amount of ticks it takes for the vibration to travel from its
    /// source to its destination.
    #[varint]
    ticks: i32,
}

#[derive(ToStatic)]
pub enum VibrationSource<'a> {
    /// vibration source "minecraft:block"
    Block { position: Position442 },
    /// vibration source "minecraft:entity"
    Entity {
        /// The ID of the entity the vibration originated from
        id: i32,
        /// The height of the entity's eye relative to the entity
        eye_height: f32,
    },
    /// vibration source neither "minecraft:block" nor "minecraft:entity"
    Other(Cow<'a, str>),
}

impl<'dec: 'a, 'a> Decode<'dec> for VibrationSource<'a> {
    fn decode(cursor: &mut std::io::Cursor<&'dec [u8]>) -> decode::Result<Self> {
        use VibrationSource::*;
        Ok(match <&str>::decode(cursor)? {
            "minecraft:block" => Block {
                position: Position442::decode(cursor)?,
            },
            "minecraft:entity" => Entity {
                id: Var::decode(cursor)?.into_inner(),
                eye_height: f32::decode(cursor)?,
            },
            source => Other(Cow::Borrowed(source)),
        })
    }
}
impl<'a> Encode for VibrationSource<'a> {
    fn encode(&self, writer: &mut impl std::io::Write) -> encode::Result<()> {
        use VibrationSource::*;
        match self {
            Block { position } => {
                "minecraft:block".encode(writer)?;
                position.encode(writer)?;
            }
            Entity { id, eye_height } => {
                "minecraft:entity".encode(writer)?;
                Var::from(*id).encode(writer)?;
                eye_height.encode(writer)?;
            }
            Other(source) => {
                #[cfg(debug_assertions)]
                match source.as_ref() {
                    "minecraft:block" => return Err(encode::Error::Custom(
                        "specified \"minecraft:block\" as vibration source without necessary data",
                    )),
                    "minecraft:entity" => return Err(encode::Error::Custom(
                        "specified \"minecraft:entity\" as vibration source without necessary data",
                    )),
                    _ => {}
                }
                source.encode(writer)?;
            }
        }
        Ok(())
    }
}

#[derive(Encoding, ToStatic)]
pub struct Color {
    /// Red value, 0.0..=1.0
    red: f32,
    /// Green value, 0.0..=1.0
    green: f32,
    /// Blue value, 0.0..=1.0
    blue: f32,
}
