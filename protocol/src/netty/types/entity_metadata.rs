use crate::*;
use attrs::*;

use miners_nbt::Compound;
use std::borrow::Cow;
use std::collections::BTreeMap;
use uuid::Uuid;

use super::position::Position;
use super::slot::*;

const DUPLICATE_METADATA_INDEX: &str = "duplicate index in Metadata";

/// The first EntityMetadata
pub type EntityMetadata0<'a> = PackedEntityMetadata<Value0<'a>>;
/// Chat shifted the ids
pub type EntityMetadata57<'a> = PackedEntityMetadata<Value57<'a, Slot0<'a>>>;
// todo! figure out if this was changed in version 67, 68 or 69
/// change to key and type in their own respective bytes
pub type EntityMetadata69<'a> = EntityMetadata<Value57<'a, Slot0<'a>>>;
/// Slot changed
pub type EntityMetadata350<'a> = EntityMetadata<Value57<'a, Slot350<'a>>>;
// todo! possible changes because of particle from here on
// pv353 OptChat shifted the ids
// pub type EntityMetadata353<'a> = EntityMetadata<Value353<'a, Slot350<'a>, Position, Particle>>;
// pv402 Slot type changed
// pub type EntityMetadata402<'a> = EntityMetadata<Value353<'a, Slot402<'a>, Position, Particle>>;
// pv442 Position type changed
// pub type EntityMetadata442<'a> = EntityMetadata<Value353<'a, Slot402<'a>, Position, Particle>>;

pub struct PackedEntityMetadata<Value> {
    inner: BTreeMap<u8, Value>,
}
impl<'dec, Value> Decode<'dec> for PackedEntityMetadata<Value>
where
    Value: PackedMetadataDecode<'dec>,
{
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
                    vacant.insert(Value::metadata_decode(packed >> 5, cursor)?);
                }
                Entry::Occupied(_) => {
                    return Err(decode::Error::Custom(DUPLICATE_METADATA_INDEX));
                }
            }
        }
        Ok(PackedEntityMetadata { inner: map })
    }
}
impl<Value> Encode for PackedEntityMetadata<Value>
where
    Value: PackedMetadataEncode,
{
    fn encode(&self, writer: &mut impl std::io::Write) -> encode::Result<()> {
        for (index, value) in &self.inner {
            value.encode_packed(*index, writer)?;
        }
        Ok(())
    }
}

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

trait PackedMetadataDecode<'dec>: Sized {
    fn metadata_decode(id: u8, cursor: &mut std::io::Cursor<&[u8]>) -> decode::Result<Self>;
}
trait PackedMetadataEncode {
    fn encode_packed(&self, index: u8, writer: &mut impl std::io::Write) -> encode::Result<()>;
}

macro_rules! packed_metadata {
    (
        $(#[$($enum_attr:tt)*])*
        pub enum $name:ident $(<[$($generics:tt)*]>)? {
            $(
                $(#[$($case_attr:tt)*])?
                $case:ident
                $( ($( $(#[$($unnamed_attr:tt)*])* $unnamed_typ:ty),* $(,)?) )?
                $( {$( $field:ident : $(#[$($named_attr:tt)*])* $named_typ:ty ),* $(,)?} )?
                $(= $id:literal)?
            ),* $(,)?
        }
    ) => {
        $(#[$($enum_attr)*])*
        pub enum $name $(<$($generics)*>)? {
            $(
                $(#[$($case_attr)*])?
                $case
                $( ($( $(#[$($unnamed_attr)*])* $unnamed_typ),*) )?
                $( {$( $field : $(#[$($named_attr)*])* $named_typ ),*} )?
            ),*
        }
    };
}
packed_metadata! {
    pub enum Value0 <['a]> {
        Byte(i8) = 0,
        Short(i16) = 1,
        Int(i32) = 2,
        Float(f32) = 3,
        String(Cow<'a, str>) = 4,
        Slot(Slot0<'a>) = 5,
        Position(IntPosition) = 6,
        Rotation(Rotation) = 7,
    }
}

// todo!()
#[derive(Encoding, ToStatic)]
pub struct Chat<'a>(Cow<'a, str>);

packed_metadata! {
    #[derive(Encoding, ToStatic)]
    #[from(u8)]
    pub enum Value57 <['a, Slot]> {
        #[case(0)]
        Byte(i8) = 0,
        VarInt(#[varint] i32) = 1,
        Float(f32) = 2,
        String(Cow<'a, str>) = 3,
        Chat(Chat<'a>) = 4,
        Slot(Slot) = 5,
        Boolean(bool) = 6,
        Rotation(Rotation) = 7,
        Position(Position) = 8,
        OptPosition(Option<Position>) = 9,
        Direction(Direction) = 10,
        OptUuid(Option<Uuid>) = 11,
        BlockId(BlockId) = 12,
        Nbt(Compound<'a>) = 13,
    }
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
