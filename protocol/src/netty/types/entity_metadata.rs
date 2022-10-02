use crate::*;
use attrs::*;

use miners_nbt::Compound;
use std::borrow::Cow;
use std::collections::BTreeMap;
use uuid::Uuid;

use super::particle::*;
use super::position::*;
use super::slot::*;

const DUPLICATE_METADATA_INDEX: &str = "duplicate index in EntityMetadata";

/// The first EntityMetadata
#[derive(ToStatic)]
pub struct PackedEntityMetadata0<'a> {
    inner: BTreeMap<u8, Value0<'a>>,
}
/// Chat shifted the ids and packed key and type values are no longer used
/// both now have their own respective bytes
pub type EntityMetadata57<'a> = EntityMetadata<Value57<'a, Slot0<'a>>>;
/// Slot changed
pub type EntityMetadata346<'a> = EntityMetadata<Value57<'a, Slot346<'a>>>;
/// OptChat shifted the ids
pub type EntityMetadata353<'a> =
    EntityMetadata<Value353<'a, Slot346<'a>, Position6, Particle353<'a, Slot346<'a>>>>;
/// Slot type changed
pub type EntityMetadata402<'a> =
    EntityMetadata<Value353<'a, Slot402<'a>, Position6, Particle353<'a, Slot402<'a>>>>;
/// Position type changed
pub type EntityMetadata441<'a> =
    EntityMetadata<Value353<'a, Slot402<'a>, Position441, Particle353<'a, Slot402<'a>>>>;
/// Particles shifted
pub type EntityMetadata463<'a> =
    EntityMetadata<Value353<'a, Slot402<'a>, Position441, Particle463<'a, Slot402<'a>>>>;
/// Particles shifted
pub type EntityMetadata701<'a> =
    EntityMetadata<Value353<'a, Slot402<'a>, Position441, Particle701<'a, Slot402<'a>>>>;
/// Particles shifted
pub type EntityMetadata706<'a> =
    EntityMetadata<Value353<'a, Slot402<'a>, Position441, Particle706<'a, Slot402<'a>>>>;
/// Added Particle information (DustColorTransition, Vibration)
pub type EntityMetadata755<'a> =
    EntityMetadata<Value353<'a, Slot402<'a>, Position441, Particle755<'a, Slot402<'a>>>>;
/// Particles shifted
pub type EntityMetadataS20<'a> =
    EntityMetadata<Value353<'a, Slot402<'a>, Position441, ParticleS20<'a, Slot402<'a>>>>;
/// Particles shifted
pub type EntityMetadata757<'a> =
    EntityMetadata<Value353<'a, Slot402<'a>, Position441, Particle757<'a, Slot402<'a>>>>;
/// Particles shifted
pub type EntityMetadataS74<'a> =
    EntityMetadata<Value353<'a, Slot402<'a>, Position441, ParticleS74<'a, Slot402<'a>>>>;
/// Particles shifted
pub type EntityMetadata759<'a> =
    EntityMetadata<Value353<'a, Slot402<'a>, Position441, Particle759<'a, Slot402<'a>>>>;

#[derive(ToStatic)]
pub enum Value0<'a> {
    Byte(i8),
    Short(i16),
    Int(i32),
    Float(f32),
    String(Cow<'a, str>),
    Slot(Slot0<'a>),
    Position(IntPosition),
    Rotation(Rotation),
}

impl<'dec: 'a, 'a> Decode<'dec> for PackedEntityMetadata0<'a> {
    fn decode(cursor: &mut std::io::Cursor<&'dec [u8]>) -> decode::Result<Self> {
        let mut map = BTreeMap::new();
        loop {
            let packed = u8::decode(cursor)?;
            if packed == 0xff {
                break;
            }
            use std::collections::btree_map::Entry;
            match map.entry(packed & 0b11111) {
                Entry::Vacant(vacant) => {
                    vacant.insert(match packed >> 5 {
                        0 => Value0::Byte(Decode::decode(cursor)?),
                        1 => Value0::Short(Decode::decode(cursor)?),
                        2 => Value0::Int(Decode::decode(cursor)?),
                        3 => Value0::Float(Decode::decode(cursor)?),
                        4 => Value0::String(Decode::decode(cursor)?),
                        5 => Value0::Slot(Decode::decode(cursor)?),
                        6 => Value0::Position(Decode::decode(cursor)?),
                        7 => Value0::Rotation(Decode::decode(cursor)?),
                        _ => return Err(decode::Error::InvalidId),
                    });
                }
                Entry::Occupied(_) => {
                    return Err(decode::Error::Custom(DUPLICATE_METADATA_INDEX));
                }
            }
        }
        Ok(PackedEntityMetadata0 { inner: map })
    }
}
impl<'a> Encode for PackedEntityMetadata0<'a> {
    fn encode(&self, writer: &mut impl std::io::Write) -> encode::Result<()> {
        macro_rules! match_encode {
            ($($case:ident => $num:literal),* $(,)?) => {
                for (index, value) in &self.inner {
                    match value {
                        $(Value0::$case(val) => {
                            ((($num as u8) << 5) | (index & 0b11111)).encode(writer)?;
                            val.encode(writer)?;
                        })*
                    }
                }
            };
        }
        match_encode! {
            Byte => 0,
            Short => 1,
            Int => 2,
            Float => 3,
            String => 4,
            Slot => 5,
            Position => 6,
            Rotation => 7
        }
        Ok(())
    }
}

#[derive(ToStatic)]
pub struct EntityMetadata<Value> {
    inner: BTreeMap<u8, Value>,
}
impl<'dec, Value> Decode<'dec> for EntityMetadata<Value>
where
    Value: Decode<'dec>,
{
    fn decode(cursor: &mut std::io::Cursor<&'dec [u8]>) -> decode::Result<Self> {
        let mut map = BTreeMap::new();
        loop {
            let index = u8::decode(cursor)?;
            if index == 0xff {
                break;
            }
            use std::collections::btree_map::Entry;
            match map.entry(index) {
                Entry::Vacant(vacant) => {
                    vacant.insert(Value::decode(cursor)?);
                }
                Entry::Occupied(_) => return Err(decode::Error::Custom(DUPLICATE_METADATA_INDEX)),
            }
        }
        Ok(EntityMetadata { inner: map })
    }
}
impl<Value> Encode for EntityMetadata<Value>
where
    Value: Encode,
{
    fn encode(&self, writer: &mut impl std::io::Write) -> encode::Result<()> {
        for (index, value) in &self.inner {
            index.encode(writer)?;
            value.encode(writer)?;
        }
        Ok(())
    }
}

#[derive(Encoding, ToStatic)]
pub struct Chat<'a>(Cow<'a, str>);

#[derive(Encoding, ToStatic)]
#[from(u8)]
pub enum Value57<'a, Slot> {
    #[case(0)]
    Byte(i8),
    VarInt(#[varint] i32),
    Float(f32),
    String(Cow<'a, str>),
    Chat(Chat<'a>),
    Slot(Slot),
    Boolean(bool),
    Rotation(Rotation),
    Position(Position6),
    OptPosition(Option<Position6>),
    Direction(Direction),
    OptUuid(Option<Uuid>),
    BlockId(BlockId),
    Nbt(Compound<'a>),
}

#[derive(Encoding, ToStatic)]
#[from(u8)]
pub enum Value353<'a, Slot, Position, Particle> {
    #[case(0)]
    Byte(i8),
    VarInt(#[varint] i32),
    Float(f32),
    String(Cow<'a, str>),
    Chat(Chat<'a>),
    OptChat(Option<Chat<'a>>),
    Slot(Slot),
    Boolean(bool),
    Rotation(Rotation),
    Position(Position),
    OptPosition(Option<Position>),
    Direction(Direction),
    OptUuid(Option<Uuid>),
    BlockId(BlockId),
    Nbt(Compound<'a>),
    Particle(Particle),
    /// introduced in pv451 (18w50a)
    VillagerData(VillagerData),
    /// 0 for absent; 1 + actual value otherwise. Used for entity IDs.
    /// introduced in pv459 (19w06a)
    OptVarInt(#[varint] i32),
    /// introduced in pv461 (19w08a)
    Pose(Pose),
    /// A VarInt that points towards the CAT_VARIANT registry.
    CatVariant(#[varint] i32),
    /// A VarInt that points towards the FROG_VARIANT registry.
    FrogVariant(#[varint] i32),
    GlobalPos(GlobalPos<'a, Position>),
    /// A VarInt that points towards the PAINTING_VARIANT registry.
    PaintingVariant(#[varint] i32),
}

#[derive(Encoding, ToStatic)]
pub struct IntPosition {
    x: i32,
    y: i32,
    z: i32,
}

#[derive(Encoding, ToStatic)]
pub struct Rotation {
    pitch: f32,
    yaw: f32,
    roll: f32,
}

#[derive(Encoding, ToStatic)]
pub struct VillagerData {
    #[varint]
    kind: i32,
    #[varint]
    profession: i32,
    #[varint]
    level: i32,
}

#[derive(Encoding, ToStatic)]
// #[varint]
// technically varint but no values greater than u7
#[from(u8)]
pub enum Pose {
    Standing = 0,
    FallFlying,
    Sleeping,
    Swimming,
    SpinAttack,
    Sneaking,
    Dying,
}

#[derive(Encoding, ToStatic)]
// #[varint]
// technically a varint but can only assume byte values
#[from(u8)]
pub enum Direction {
    Down = 0,
    Up,
    North,
    South,
    West,
    East,
}

#[derive(ToStatic)]
// `id << 4 | data`, varint
// global palette, 0 means absent
pub struct BlockId {
    id: i32,
    data: u8,
}
impl<'dec> Decode<'dec> for BlockId {
    fn decode(cursor: &mut std::io::Cursor<&'dec [u8]>) -> decode::Result<Self> {
        let blockid = u32::decode(cursor)?;
        Ok(BlockId {
            id: (blockid >> 4) as i32,
            data: blockid as u8 & 0b1111,
        })
    }
}
impl Encode for BlockId {
    fn encode(&self, writer: &mut impl std::io::Write) -> encode::Result<()> {
        #[cfg(debug_assertions)]
        if (self.id as u32 & 0b1111_0000_0000_0000_0000_0000_0000_0000) != 0 {
            return Err(encode::Error::Custom(
                "block id too large (small if signed)",
            ));
        }
        #[cfg(debug_assertions)]
        if (self.data & 0b11110000) != 0 {
            return Err(encode::Error::Custom("block data invalid"));
        }
        (((self.id as u32) << 4) | (self.data & 0b1111) as u32).encode(writer)?;
        Ok(())
    }
}

#[derive(Encoding, ToStatic)]
pub struct GlobalPos<'a, Position> {
    dimension: Cow<'a, str>,
    position: Position,
}
