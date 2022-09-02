use crate::netty::Position;
use crate::*;
use attrs::*;

use std::borrow::Cow;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Encoding, ToStatic)]
pub struct KeepAlive0 {
    pub id: i32,
}

#[derive(ToStatic)]
pub struct JoinGame0 {
    pub entity_id: i32,
    pub hardcore: bool,
    pub gamemode: GameMode0,
    pub dimension: Dimension0,
    pub difficulty: super::Difficulty0,
    pub max_players: u8,
}

impl<'dec> Decode<'dec> for JoinGame0 {
    fn decode(cursor: &mut std::io::Cursor<&'dec [u8]>) -> decode::Result<Self> {
        let entity_id = i32::decode(cursor)?;
        let bitfield = u8::decode(cursor)?;
        let hardcore = bitfield & 0x08 != 0;
        let gamemode = match bitfield & 0b11 {
            0 => GameMode0::Survival,
            1 => GameMode0::Creative,
            2 => GameMode0::Adventure,
            _ => return Err(decode::Error::InvalidId),
        };
        Ok(Self {
            entity_id,
            hardcore,
            gamemode,
            dimension: Dimension0::decode(cursor)?,
            difficulty: Difficulty0::decode(cursor)?,
            max_players: u8::decode(cursor)?,
        })
    }
}

impl Encode for JoinGame0 {
    fn encode(&self, writer: &mut impl ::std::io::Write) -> Result<(), encode::Error> {
        self.entity_id.encode(writer)?;
        (match self.gamemode {
            GameMode0::Survival => 0,
            GameMode0::Creative => 1,
            GameMode0::Adventure => 2,
        } & ((self.hardcore as u8) << 3))
            .encode(writer)?;
        self.dimension.encode(writer)?;
        self.difficulty.encode(writer)?;
        self.max_players.encode(writer)?;
        Ok(())
    }
}

#[derive(ToStatic)]
pub struct JoinGame1<'a> {
    pub entity_id: i32,
    pub hardcore: bool,
    pub gamemode: GameMode0,
    pub dimension: Dimension0,
    pub difficulty: super::Difficulty0,
    pub max_players: u8,
    /// "default", "flat", "largeBiomes", "amplified", "default_1_1"
    pub level_type: Cow<'a, str>,
}

impl<'dec, 'a> Decode<'dec> for JoinGame1<'a>
where
    'dec: 'a,
{
    fn decode(cursor: &mut std::io::Cursor<&'dec [u8]>) -> decode::Result<Self> {
        let entity_id = i32::decode(cursor)?;
        let bitfield = u8::decode(cursor)?;
        let hardcore = bitfield & 0x08 != 0;
        let gamemode = match bitfield & 0b11 {
            0 => GameMode0::Survival,
            1 => GameMode0::Adventure,
            2 => GameMode0::Creative,
            _ => return Err(decode::Error::InvalidId),
        };
        Ok(Self {
            entity_id,
            hardcore,
            gamemode,
            dimension: Dimension0::decode(cursor)?,
            difficulty: Difficulty0::decode(cursor)?,
            max_players: u8::decode(cursor)?,
            level_type: Cow::decode(cursor)?,
        })
    }
}

impl Encode for JoinGame1<'_> {
    fn encode(&self, writer: &mut impl ::std::io::Write) -> Result<(), encode::Error> {
        self.entity_id.encode(writer)?;
        (match self.gamemode {
            GameMode0::Survival => 0,
            GameMode0::Creative => 1,
            GameMode0::Adventure => 2,
        } & ((self.hardcore as u8) << 3))
            .encode(writer)?;
        self.dimension.encode(writer)?;
        self.difficulty.encode(writer)?;
        self.max_players.encode(writer)?;
        self.level_type.encode(writer)?;
        Ok(())
    }
}

#[derive(Encoding, ToStatic, Clone, Copy)]
#[from(u8)]
pub enum GameMode0 {
    Survival = 0,
    Creative,
    Adventure,
}

pub use super::Difficulty0;

#[derive(Encoding, ToStatic)]
#[from(i8)]
pub enum Dimension0 {
    Nether = -1,
    Overworld = 0,
    End,
}

#[derive(Encoding, ToStatic)]
pub struct ChatMessage0<'a> {
    // todo! add ChatMessage json thing
    pub message: Cow<'a, str>,
}

#[derive(Encoding, ToStatic)]
pub struct ChatMessage6<'a> {
    // todo! add ChatMessage json thing
    pub message: Cow<'a, str>,
    pub position: ChatMessagePosition6,
}

#[derive(Encoding, ToStatic)]
#[from(u8)]
pub enum ChatMessagePosition6 {
    Chat = 0,
    System,
    Hotbar,
}

#[derive(Encoding, ToStatic)]
pub struct TimeUpdate0 {
    pub ticks: i64,
    pub time_of_day: i64,
}

#[derive(Encoding, ToStatic)]
pub struct EntityEquipment0 {
    pub entity_id: i32,
    pub slot: EquipmentSlot0,
    // todo! slot data
    // item: Slot,
}

#[derive(Encoding, ToStatic)]
pub struct EntityEquipment7 {
    #[varint]
    pub entity_id: i32,
    pub slot: EquipmentSlot0,
    // todo! slot data
    // item: Slot,
}

#[derive(Encoding, ToStatic)]
#[from(u16)]
pub enum EquipmentSlot0 {
    Held = 0,
    Boots,
    Leggings,
    Chestplate,
    Helmet,
}

#[derive(Encoding, ToStatic)]
pub struct SpawnPosition0 {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Encoding, ToStatic, Clone, Copy)]
#[bitfield]
pub struct SpawnPosition6 {
    #[bits(26)]
    pub x: i32,
    #[bits(26)]
    pub z: i32,
    #[bits(12)]
    pub y: i16,
}

#[derive(Encoding, ToStatic)]
pub struct UpdateHealth0 {
    /// 0.0 means dead, 20.0 = full HP
    pub health: f32,
    /// 0-20
    pub food: i16,
    /// 0.0 to 5.0 in integer increments?
    pub saturation: f32,
}

#[derive(Encoding, ToStatic)]
pub struct UpdateHealth7 {
    /// 0.0 means dead, 20.0 = full HP
    pub health: f32,
    /// 0-20
    #[varint]
    pub food: i32,
    /// 0.0 to 5.0 in integer increments?
    pub saturation: f32,
}

#[derive(Encoding, ToStatic)]
pub struct Respawn0 {
    pub dimension: i32,
    pub difficulty: Difficulty0,
    // no hardcore flag here
    pub gamemode: GameMode0,
}

#[derive(Encoding, ToStatic)]
pub struct Respawn1<'a> {
    pub dimension: i32,
    pub difficulty: Difficulty0,
    // no hardcore flag here
    pub gamemode: GameMode0,
    /// "default", "flat", "largeBiomes", "amplified", "default_1_1"
    pub level_type: Cow<'a, str>,
}

#[derive(Encoding, ToStatic)]
pub struct PositionAndLook0 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    /// Absolute rotation on the X Axis, in degrees
    pub yaw: f32,
    /// Absolute rotation on the Y Axis, in degrees
    pub pitch: f32,
    pub on_ground: bool,
}

#[derive(Encoding, ToStatic)]
pub struct PositionAndLook6 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    /// Absolute rotation on the X Axis, in degrees
    pub yaw: f32,
    /// Absolute rotation on the Y Axis, in degrees
    pub pitch: f32,
    pub relativity: PositionAndLookBitfield6,
}

#[derive(Encoding, ToStatic)]
#[bitfield(u8, reverse)]
pub struct PositionAndLookBitfield6 {
    #[bool]
    pub x: bool,
    #[bool]
    pub y: bool,
    #[bool]
    pub z: bool,
    #[bool]
    pub pitch: bool,
    #[bool]
    pub yaw: bool,
}

#[test]
fn position_and_look_bitfield6() {
    let val = &[0b00011111u8];
    let mut cursor = std::io::Cursor::new(&val[..]);
    #[allow(clippy::unwrap_used)]
    let res = PositionAndLookBitfield6::decode(&mut cursor).unwrap();
    assert!(res.x);
    assert!(res.y);
    assert!(res.z);
    assert!(res.pitch);
    assert!(res.yaw);
    let mut cursor = vec![];
    #[allow(clippy::unwrap_used)]
    res.encode(&mut cursor).unwrap();
    assert_eq!(&cursor[..], &val[..])
}

#[derive(Encoding, ToStatic)]
pub struct HeldItemChange0 {
    /// The slot which the player has selected (0-8)
    pub slot: u8,
}

#[derive(Encoding, ToStatic)]
pub struct UseBed0 {
    pub entity_id: i32,
    pub x: i32,
    pub y: i8,
    pub z: i32,
}

#[derive(Encoding, ToStatic)]
pub struct UseBed6 {
    pub entity_id: i32,
    pub location: Position,
}

#[derive(Encoding, ToStatic)]
pub struct UseBed7 {
    #[varint]
    pub entity_id: i32,
    pub location: Position,
}

#[derive(Encoding, ToStatic)]
pub struct Animation0 {
    #[varint]
    pub entity_id: i32,
    animation: super::AnimationId0,
}

#[derive(Encoding, ToStatic)]
pub struct SpawnPlayer0<'a> {
    #[varint]
    pub entity_id: i32,
    #[stringuuid]
    pub player_uuid: Uuid,
    pub name: Cow<'a, str>,
    #[fixed(5, i32)]
    /// Player X as a Fixed-Point number
    pub x: f64,
    #[fixed(5, i32)]
    /// Player Y as a Fixed-Point number
    pub y: f64,
    #[fixed(5, i32)]
    /// Player Z as a Fixed-Point number
    pub z: f64,
    pub yaw: Angle,
    pub pitch: Angle,
    /// The item the player is currently holding. Note that this should be 0
    /// for "no item", unlike -1 used in other packets. A negative value
    /// crashes clients.
    pub current_item: u16,
    pub metadata: EntityMetadata,
}

#[derive(ToStatic)]
pub struct SpawnPlayer5<'a> {
    // varint
    pub entity_id: i32,
    pub player_uuid: Option<Uuid>,
    pub name: Cow<'a, str>,
    pub properties: Vec<PlayerProperty<'a>>,
    // fixed(5, i32)
    pub x: f64,
    // fixed(5, i32)
    pub y: f64,
    // fixed(5, i32)
    pub z: f64,
    pub yaw: Angle,
    pub pitch: Angle,
    /// The item the player is currently holding. Note that this should be 0
    /// for "no item", unlike -1 used in other packets. A negative value
    /// crashes clients.
    pub current_item: u16,
    pub metadata: EntityMetadata,
}
impl<'dec, 'a> Decode<'dec> for SpawnPlayer5<'a>
where
    'dec: 'a,
{
    fn decode(cursor: &mut std::io::Cursor<&'dec [u8]>) -> decode::Result<Self> {
        let entity_id = Var::decode(cursor)?.into_inner();
        let uuid = <&str>::decode(cursor)?;

        Ok(Self {
            entity_id,
            player_uuid: if !uuid.is_empty() {
                Some(Uuid::from_str(uuid)?)
            } else {
                None
            },
            properties: Vec::decode(cursor)?,
            name: Decode::decode(cursor)?,
            x: Fixed::<5, i32, _>::decode(cursor)?.into_inner(),
            y: Fixed::<5, i32, _>::decode(cursor)?.into_inner(),
            z: Fixed::<5, i32, _>::decode(cursor)?.into_inner(),
            yaw: Decode::decode(cursor)?,
            pitch: Decode::decode(cursor)?,
            current_item: Decode::decode(cursor)?,
            metadata: Decode::decode(cursor)?,
        })
    }
}
impl<'a> Encode for SpawnPlayer5<'a> {
    fn encode(&self, cursor: &mut impl ::std::io::Write) -> encode::Result<()> {
        let Self {
            entity_id,
            player_uuid,
            name,
            properties,
            x,
            y,
            z,
            yaw,
            pitch,
            current_item,
            metadata,
        } = self;
        Encode::encode(&Var::from(*entity_id), cursor)?;
        if let Some(player_uuid) = player_uuid {
            Encode::encode(&StringUuid::from(*player_uuid), cursor)?;
        } else {
            "".encode(cursor)?;
        }
        name.encode(cursor)?;
        properties.encode(cursor)?;
        Fixed::<5, i32, _>::from(x).encode(cursor)?;
        Fixed::<5, i32, _>::from(y).encode(cursor)?;
        Fixed::<5, i32, _>::from(z).encode(cursor)?;
        yaw.encode(cursor)?;
        pitch.encode(cursor)?;
        current_item.encode(cursor)?;
        metadata.encode(cursor)?;
        Ok(())
    }
}

#[derive(ToStatic)]
pub struct SpawnPlayer7<'a> {
    // varint
    pub entity_id: i32,
    pub player_uuid: Uuid,
    pub name: Cow<'a, str>,
    pub properties: Vec<PlayerProperty<'a>>,
    // fixed(5, i32)
    pub x: f64,
    // fixed(5, i32)
    pub y: f64,
    // fixed(5, i32)
    pub z: f64,
    pub yaw: Angle,
    pub pitch: Angle,
    /// The item the player is currently holding. Note that this should be 0
    /// for "no item", unlike -1 used in other packets. A negative value
    /// crashes clients.
    pub current_item: u16,
    pub metadata: EntityMetadata,
}
impl<'dec, 'a> Decode<'dec> for SpawnPlayer7<'a>
where
    'dec: 'a,
{
    fn decode(cursor: &mut std::io::Cursor<&'dec [u8]>) -> decode::Result<Self> {
        Ok(Self {
            entity_id: Var::decode(cursor)?.into_inner(),
            player_uuid: StringUuid::decode(cursor)?.into_inner(),
            properties: Vec::decode(cursor)?,
            name: Decode::decode(cursor)?,
            x: Fixed::<5, i32, _>::decode(cursor)?.into_inner(),
            y: Fixed::<5, i32, _>::decode(cursor)?.into_inner(),
            z: Fixed::<5, i32, _>::decode(cursor)?.into_inner(),
            yaw: Decode::decode(cursor)?,
            pitch: Decode::decode(cursor)?,
            current_item: Decode::decode(cursor)?,
            metadata: Decode::decode(cursor)?,
        })
    }
}
impl<'a> Encode for SpawnPlayer7<'a> {
    fn encode(&self, cursor: &mut impl ::std::io::Write) -> encode::Result<()> {
        let Self {
            entity_id,
            player_uuid,
            name,
            properties,
            x,
            y,
            z,
            yaw,
            pitch,
            current_item,
            metadata,
        } = self;
        Encode::encode(&Var::from(*entity_id), cursor)?;
        Encode::encode(&StringUuid::from(*player_uuid), cursor)?;
        name.encode(cursor)?;
        properties.encode(cursor)?;
        Fixed::<5, i32, _>::from(x).encode(cursor)?;
        Fixed::<5, i32, _>::from(y).encode(cursor)?;
        Fixed::<5, i32, _>::from(z).encode(cursor)?;
        yaw.encode(cursor)?;
        pitch.encode(cursor)?;
        current_item.encode(cursor)?;
        metadata.encode(cursor)?;
        Ok(())
    }
}

#[derive(Encoding, ToStatic)]
pub struct SpawnPlayer19<'a> {
    #[varint]
    pub entity_id: i32,
    pub player_uuid: Uuid,
    pub name: Cow<'a, str>,
    #[fixed(5, i32)]
    /// Player X as a Fixed-Point number
    pub x: f64,
    #[fixed(5, i32)]
    /// Player Y as a Fixed-Point number
    pub y: f64,
    #[fixed(5, i32)]
    /// Player Z as a Fixed-Point number
    pub z: f64,
    pub yaw: Angle,
    pub pitch: Angle,
    /// The item the player is currently holding. Note that this should be 0
    /// for "no item", unlike -1 used in other packets. A negative value
    /// crashes clients.
    pub current_item: u16,
    pub metadata: EntityMetadata,
}

#[derive(Encoding, ToStatic)]
pub struct PlayerProperty<'a> {
    pub name: Cow<'a, str>,
    pub value: Cow<'a, str>,
    pub signature: Cow<'a, str>,
}

#[derive(Encoding, ToStatic)]
// todo! metadata
pub struct EntityMetadata {}

#[derive(Encoding, ToStatic)]
pub struct CollectItem0 {
    pub collected_id: i32,
    pub collector_id: i32,
}

#[derive(Encoding, ToStatic)]
pub struct CollectItem7 {
    #[varint]
    pub collected_id: i32,
    #[varint]
    pub collector_id: i32,
}

#[derive(Encoding, ToStatic)]
pub struct SpawnObject0 {
    #[varint]
    pub entity_id: i32,
    // todo! (see [`Object0`])
    pub kind: Object0,
    #[fixed(5, i32)]
    /// X position as a Fixed-Point number
    pub x: f64,
    #[fixed(5, i32)]
    /// Y position as a Fixed-Point number
    pub y: f64,
    #[fixed(5, i32)]
    /// Z position as a Fixed-Point number
    pub z: f64,
    pub pitch: Angle,
    pub yaw: Angle,
    // todo! should be covered by kind, look above
    pub data: Object0,
}

#[derive(Encoding, ToStatic)]
#[from(u8)]
// todo! add #[separated] to have a custom option for
// separated type and cursor/writer impl
// would produce a custom read like `read(kind: $from, cursor: Cursor<&[u8]>) -> Result`
// and write like `write_kind(&self, writer: impl Write) -> Result`
// and `write_self(self, writer: impl Write) -> Result
pub enum Object0 {
    #[case(1)]
    Boat,
    ItemStack,
    AreaEffectCloud,
    #[case(10)]
    Minecart,
    /// unused since 1.6.x
    StorageMinecart,
    /// unused since 1.6.x
    PoweredMinecart,
    #[case(50)]
    ActivatedTNT,
    EnderCrystal,
    #[case(60)]
    Arrow,
    Snowball,
    Egg,
    Fireball,
    FireCharge,
    ThrownEnderpearl,
    WitherSkull,
    ShulkerBullet,
    #[case(70)]
    FallingObject,
    ItemFrame(Orientation),
    EyeOfEnder,
    ThrownPotion,
    FallingDragonEgg,
    ThrownExpBottle,
    FireworkRocket,
    LeashKnot,
    ArmorStand,
    #[case(90)]
    FishingFloat,
    SpectralArrow,
    TippedArrow,
    DragonFireball,
}

#[derive(Encoding, ToStatic)]
// todo
pub enum Orientation {
    #[case(0)]
    _NonExhaustive,
}

#[derive(Encoding, ToStatic)]
pub enum EntityKind0 {
    Mob = 48,
    Monster,
    Creeper,
    Skeleton,
    Spider,
    GiantZombie,
    Zombie,
    Slime,
    Ghast,
    ZombiePigman,
    Enderman,
    CaveSpider,
    SilverFish,
    Blaze,
    MagmaCube,
    EnderDragon,
    Wither,
    Bat,
    Witch,
    Endermite,
    Guardian,
    Shulker,
    Pig = 90,
    Sheep,
    Cow,
    Chicken,
    Squid,
    Wolf,
    Mooshroom,
    Snowman,
    Ocelot,
    IronGolem,
    Horse,
    Rabbit,
    Villager = 120,
}

#[derive(Encoding, ToStatic)]
pub struct SpawnMob0 {
    #[varint]
    pub entity_id: i32,
    // todo! see #[separated] on Object
    pub kind: EntityKind0,
    #[fixed(5, i32)]
    pub x: f64,
    #[fixed(5, i32)]
    pub y: f64,
    #[fixed(5, i32)]
    pub z: f64,
    pub pitch: Angle,
    pub head_pitch: Angle,
    pub yaw: Angle,
    pub velocity_x: i16,
    pub velocity_y: i16,
    pub velocity_z: i16,
    // todo! see type
    pub metadata: EntityMetadata,
}

#[derive(Encoding, ToStatic)]
pub struct SpawnPainting0<'a> {
    #[varint]
    pub entity_id: i32,
    // todo! #[max_len(13)]
    /// Name of the painting. Max length 13
    pub title: Cow<'a, str>,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub direction: Direction0,
}
#[derive(Encoding, ToStatic)]
pub struct SpawnPainting8<'a> {
    #[varint]
    pub entity_id: i32,
    // todo! #[max_len(13)]
    /// Name of the painting. Max length 13
    pub title: Cow<'a, str>,
    pub location: Position,
    pub direction: Direction0,
}

#[derive(Encoding, ToStatic)]
#[from(u32)]
pub enum Direction0 {
    NegZ = 0,
    NegX,
    PosZ,
    PosX,
}

#[derive(Encoding, ToStatic)]
pub struct SpawnExpOrb0 {
    #[varint]
    pub entity_id: i32,
    #[fixed(5, i32)]
    pub x: f64,
    #[fixed(5, i32)]
    pub y: f64,
    #[fixed(5, i32)]
    pub z: f64,
    /// The amount of experience this orb will reward once collected
    pub count: i16,
}

#[derive(ToStatic)]
pub struct EntityVelocity0 {
    pub entity_id: i32,
    /// watch out, this value is clamped to +3.9 and -3.9 in the notchian client
    pub x: f32,
    /// watch out, this value is clamped to +3.9 and -3.9 in the notchian client
    pub y: f32,
    /// watch out, this value is clamped to +3.9 and -3.9 in the notchian client
    pub z: f32,
}
impl<'dec> Decode<'dec> for EntityVelocity0 {
    fn decode(cursor: &mut std::io::Cursor<&'dec [u8]>) -> decode::Result<Self> {
        Ok(Self {
            entity_id: Decode::decode(cursor)?,
            x: Fixed::<0, i16, f32>::decode(cursor)?.into_inner() / 8000.0,
            y: Fixed::<0, i16, f32>::decode(cursor)?.into_inner() / 8000.0,
            z: Fixed::<0, i16, f32>::decode(cursor)?.into_inner() / 8000.0,
        })
    }
}
impl Encode for EntityVelocity0 {
    fn encode(&self, cursor: &mut impl ::std::io::Write) -> Result<(), encode::Error> {
        let Self { entity_id, x, y, z } = self;
        entity_id.encode(cursor)?;
        Fixed::<0, i16, f32>::from(x * 8000.0).encode(cursor)?;
        Fixed::<0, i16, f32>::from(y * 8000.0).encode(cursor)?;
        Fixed::<0, i16, f32>::from(z * 8000.0).encode(cursor)?;
        Ok(())
    }
}
#[derive(ToStatic)]
pub struct EntityVelocity7 {
    // varint
    pub entity_id: i32,
    /// watch out, this value is clamped to +3.9 and -3.9 in the notchian client
    pub x: f32,
    /// watch out, this value is clamped to +3.9 and -3.9 in the notchian client
    pub y: f32,
    /// watch out, this value is clamped to +3.9 and -3.9 in the notchian client
    pub z: f32,
}
impl<'dec> Decode<'dec> for EntityVelocity7 {
    fn decode(cursor: &mut std::io::Cursor<&'dec [u8]>) -> decode::Result<Self> {
        Ok(Self {
            entity_id: Var::decode(cursor)?.into_inner(),
            x: Fixed::<0, i16, f32>::decode(cursor)?.into_inner() / 8000.0,
            y: Fixed::<0, i16, f32>::decode(cursor)?.into_inner() / 8000.0,
            z: Fixed::<0, i16, f32>::decode(cursor)?.into_inner() / 8000.0,
        })
    }
}
impl Encode for EntityVelocity7 {
    fn encode(&self, cursor: &mut impl ::std::io::Write) -> Result<(), encode::Error> {
        let Self { entity_id, x, y, z } = self;
        Var::from(*entity_id).encode(cursor)?;
        Fixed::<0, i16, f32>::from(x * 8000.0).encode(cursor)?;
        Fixed::<0, i16, f32>::from(y * 8000.0).encode(cursor)?;
        Fixed::<0, i16, f32>::from(z * 8000.0).encode(cursor)?;
        Ok(())
    }
}

#[derive(Encoding, ToStatic)]
pub struct DestroyEntities0 {
    #[counted(u8)]
    pub entities: Vec<i32>,
}

#[derive(ToStatic)]
pub struct DestroyEntities7 {
    pub entities: Vec<i32>,
}
impl<'dec> Decode<'dec> for DestroyEntities7 {
    fn decode(cursor: &mut std::io::Cursor<&'dec [u8]>) -> decode::Result<Self> {
        let len = Var::<u32>::decode(cursor)?.into_inner();
        let entities = (0..len)
            .map(|_| Var::<i32>::decode(cursor).map(|var| var.into_inner()))
            .collect::<Result<_, _>>()?;
        Ok(Self { entities })
    }
}
impl Encode for DestroyEntities7 {
    fn encode(&self, writer: &mut impl std::io::Write) -> encode::Result<()> {
        Var::from(self.entities.len() as u32).encode(writer)?;
        for entity in &self.entities {
            Var::from(*entity).encode(writer)?;
        }
        Ok(())
    }
}

#[derive(Encoding, ToStatic)]
pub struct Entity0 {
    pub entity_id: i32,
}

#[derive(Encoding, ToStatic)]
pub struct Entity7 {
    #[varint]
    pub entity_id: i32,
}

#[derive(Encoding, ToStatic)]
pub struct EntityRelativeMove0 {
    pub entity_id: i32,
    // todo! round x and z but floor y
    /// watch out, this must satisfy -4.0 <= x < 4.0
    /// if it is, use EntityTeleport instead
    #[fixed(5, i8)]
    pub dx: f32,
    /// watch out, this must satisfy -4.0 <= x < 4.0
    /// if it is, use EntityTeleport instead
    #[fixed(5, i8)]
    pub dy: f32,
    /// watch out, this must satisfy -4.0 <= x < 4.0
    /// if it is, use EntityTeleport instead
    #[fixed(5, i8)]
    pub dz: f32,
}

#[derive(Encoding, ToStatic)]
pub struct EntityRelativeMove7 {
    #[varint]
    pub entity_id: i32,
    // todo! round x and z but floor y
    /// watch out, this must satisfy -4.0 <= x < 4.0
    /// if it is, use EntityTeleport instead
    #[fixed(5, i8)]
    pub dx: f32,
    /// watch out, this must satisfy -4.0 <= x < 4.0
    /// if it is, use EntityTeleport instead
    #[fixed(5, i8)]
    pub dy: f32,
    /// watch out, this must satisfy -4.0 <= x < 4.0
    /// if it is, use EntityTeleport instead
    #[fixed(5, i8)]
    pub dz: f32,
}

#[derive(Encoding, ToStatic)]
pub struct EntityLook0 {
    pub entity_id: i32,
    pub yaw: Angle,
    pub pitch: Angle,
}

#[derive(Encoding, ToStatic)]
pub struct EntityLook7 {
    #[varint]
    pub entity_id: i32,
    pub yaw: Angle,
    pub pitch: Angle,
}

#[derive(Encoding, ToStatic)]
pub struct EntityLookAndRelativeMove0 {
    pub entity_id: i32,
    // todo! round x and z but floor y
    /// watch out, this must satisfy -4.0 <= x < 4.0
    /// if it is, use EntityTeleport instead
    #[fixed(5, i8)]
    pub dx: f32,
    /// watch out, this must satisfy -4.0 <= x < 4.0
    /// if it is, use EntityTeleport instead
    #[fixed(5, i8)]
    pub dy: f32,
    /// watch out, this must satisfy -4.0 <= x < 4.0
    /// if it is, use EntityTeleport instead
    #[fixed(5, i8)]
    pub dz: f32,
    pub yaw: Angle,
    pub pitch: Angle,
}

#[derive(Encoding, ToStatic)]
pub struct EntityLookAndRelativeMove7 {
    #[varint]
    pub entity_id: i32,
    // todo! round x and z but floor y
    /// watch out, this must satisfy -4.0 <= x < 4.0
    /// if it is, use EntityTeleport instead
    #[fixed(5, i8)]
    pub dx: f32,
    /// watch out, this must satisfy -4.0 <= x < 4.0
    /// if it is, use EntityTeleport instead
    #[fixed(5, i8)]
    pub dy: f32,
    /// watch out, this must satisfy -4.0 <= x < 4.0
    /// if it is, use EntityTeleport instead
    #[fixed(5, i8)]
    pub dz: f32,
    pub yaw: Angle,
    pub pitch: Angle,
}

#[derive(Encoding, ToStatic)]
pub struct EntityTeleport0 {
    pub entity_id: i32,
    // todo! round x and z but floor y
    #[fixed(5, i32)]
    pub x: f64,
    #[fixed(5, i32)]
    pub y: f64,
    #[fixed(5, i32)]
    pub z: f64,
    pub yaw: Angle,
    pub pitch: Angle,
}

#[derive(Encoding, ToStatic)]
pub struct EntityTeleport7 {
    #[varint]
    pub entity_id: i32,
    // todo! round x and z but floor y
    #[fixed(5, i32)]
    pub x: f64,
    #[fixed(5, i32)]
    pub y: f64,
    #[fixed(5, i32)]
    pub z: f64,
    pub yaw: Angle,
    pub pitch: Angle,
}

#[derive(Encoding, ToStatic)]
pub struct EntityHeadLook0 {
    pub entity_id: i32,
    pub head_yaw: Angle,
}

#[derive(Encoding, ToStatic)]
pub struct EntityHeadLook7 {
    #[varint]
    pub entity_id: i32,
    pub head_yaw: Angle,
}

#[derive(Encoding, ToStatic)]
pub struct EntityStatus0 {
    pub entity_id: i32,
    pub entity_status: Status0,
}

#[derive(Encoding, ToStatic)]
#[from(u8)]
pub enum Status0 {
    EntityHurt = 2,
    EntityDead,
    WolfTaming = 6,
    WolfTamed,
    WolfShakingWater,
    SelfEatingAccepted,
    SheepEatingGrass,
    IronGolemRose,
    VillagerHeart,
    VillagerAngry,
    VillagerHappy,
    WitchMagic,
    /// zombie converting into villager
    ZombieShakingViolently,
    FireworkExplosion,
}

#[derive(Encoding, ToStatic)]
pub struct AttachEntity0 {
    pub entity_id: i32,
    pub vehicle_id: i32,
    pub leash: bool,
}

#[derive(Encoding, ToStatic)]
pub struct EntityMetadata0 {
    pub entity_id: i32,
    pub metadata: EntityMetadata,
}

#[derive(Encoding, ToStatic)]
pub struct EntityMetadata7 {
    #[varint]
    pub entity_id: i32,
    pub metadata: EntityMetadata,
}

#[derive(Encoding, ToStatic)]
pub struct EntityEffect0 {
    pub entity_id: i32,
    // todo! effect ids
    pub effect_id: i8,
    pub amplifier: i8,
    pub duration: i16,
}

#[derive(Encoding, ToStatic)]
pub struct EntityEffect7 {
    #[varint]
    pub entity_id: i32,
    // todo! effect ids
    pub effect_id: i8,
    pub amplifier: i8,
    #[varint]
    pub duration: i32,
}

#[derive(Encoding, ToStatic)]
pub struct EntityEffect10 {
    #[varint]
    pub entity_id: i32,
    // todo! effect ids
    pub effect_id: i8,
    pub amplifier: i8,
    #[varint]
    pub duration: i32,
    pub hide_particles: bool,
}

#[derive(Encoding, ToStatic)]
pub struct RemoveEntityEffect0 {
    pub entity_id: i32,
    pub effect_id: i8,
}
#[derive(Encoding, ToStatic)]
pub struct RemoveEntityEffect7 {
    #[varint]
    pub entity_id: i32,
    pub effect_id: i8,
}

#[derive(Encoding, ToStatic)]
pub struct SetExperience0 {
    pub experience_bar: f32,
    pub level: i16,
    pub total_exp: i16,
}
#[derive(Encoding, ToStatic)]
pub struct SetExperience7 {
    pub experience_bar: f32,
    #[varint]
    pub level: i32,
    #[varint]
    pub total_exp: i32,
}

#[derive(Encoding, ToStatic)]
pub struct EntityProperties0<'a> {
    pub entity_id: i32,
    #[counted(u32)]
    pub properties: Vec<EntityProperty0<'a>>,
}
#[derive(Encoding, ToStatic)]
pub struct EntityProperty0<'a> {
    pub key: Cow<'a, str>,
    pub value: f64,
    #[counted(u16)]
    pub modifiers: Vec<Modifier0>,
}
#[derive(Encoding, ToStatic)]
pub struct Modifier0 {
    pub uuid: Uuid,
    pub amount: f64,
    pub operation: ModifierOperation0,
}

#[derive(Encoding, ToStatic)]
pub struct EntityProperties7<'a> {
    pub entity_id: i32,
    #[counted(u32)]
    pub properties: Vec<EntityProperty7<'a>>,
}
#[derive(Encoding, ToStatic)]
pub struct EntityProperty7<'a> {
    pub key: Cow<'a, str>,
    pub value: f64,
    pub modifiers: Vec<Modifier0>,
}

#[derive(Encoding, ToStatic)]
/// The mathematical behavior is as follows:
///
///   - add: Increment X by Amount
///   - multiply_base: Increment Y by X * Amount
///   - multiply: Y = Y * (1 + Amount) (equivalent to Increment Y by Y * Amount)
///
/// The game first sets X = Base, then executes all Operation add, then sets
/// Y = X, then executes all multiply_base modifiers, and finally executes all
/// multiply modifiers.
///
/// https://minecraft.fandom.com/wiki/Attribute#Vanilla_modifiers
pub enum ModifierOperation0 {
    /// Adds all of the modifiers' amounts to the current value of the
    /// attribute. For example, modifying an attribute with
    /// `{Amount:2,Operation:0}` and `{Amount:4,Operation:0}` with a Base of
    /// 3 results in 9 (3 + 2 + 4 = 9).
    Add = 0,
    /// Multiplies the current value of the attribute by (1 + x), where x is
    /// the sum of the modifiers' amounts. For example, modifying an attribute
    /// with `{Amount:2,Operation:1}` and `{Amount:4,Operation:1}` with a Base
    /// of 3 results in 21 (3 * (1 + 2 + 4) = 21).
    MultiplyBase,
    /// For every modifier, multiplies the current value of the attribute by
    /// (1 + x), where x is the amount of the particular modifier. Functions
    /// the same as Operation 1 if there is only a single modifier with
    /// operation 1 or 2. However, for multiple modifiers it multiplies the
    /// modifiers rather than adding them. For example, modifying an attribute
    /// with `{Amount:2,Operation:2}` and `{Amount:4,Operation:2}` with a Base
    /// of 3 results in 45 (3 * (1 + 2) * (1 + 4) = 45).
    Multiply,
}

#[derive(Encoding, ToStatic)]
// todo! make this nice to interact with
pub struct ChunkData0<'a> {
    pub chunk_x: i32,
    pub chunk_y: i32,
    /// This is True if the packet represents all sections in this vertical
    /// column, where the primary bit map specifies exactly which sections are
    /// included, and which are air
    pub continuous: bool,
    /// Bitmask with 1 for every 16x16x16 section which data follows in the compressed data.
    pub primary_bitmap: u16,
    // todo! waht is this for?
    /// Same as above, but this is used exclusively for the 'add' portion of the payload
    pub add_bitmap: u16,
    #[counted(u32)]
    pub compressed_data: Cow<'a, [u8]>,
}

#[derive(ToStatic)]
pub struct MultiBlockChange0 {
    // varint
    pub chunk_x: i32,
    // varint
    pub chunk_y: i32,
    // count(u16)
    pub records: Vec<Record>,
}

impl<'dec> Decode<'dec> for MultiBlockChange0 {
    fn decode(cursor: &mut std::io::Cursor<&'dec [u8]>) -> decode::Result<Self> {
        let chunk_x: i32 = Var::decode(cursor)?.into_inner();
        let chunk_y: i32 = Var::decode(cursor)?.into_inner();
        let record_count = u16::decode(cursor)?;
        let data_size: i32 = i32::decode(cursor)?;
        if data_size != record_count as i32 * 4 {
            // todo! different error
            return Err(decode::Error::InvalidId);
        }
        let records: Vec<_> = (0..record_count)
            .map(|_| Record::decode(cursor))
            .collect::<Result<_, _>>()?;
        Ok(Self {
            chunk_x,
            chunk_y,
            records,
        })
    }
}

impl Encode for MultiBlockChange0 {
    fn encode(&self, writer: &mut impl std::io::Write) -> Result<(), encode::Error> {
        Var::from(self.chunk_x).encode(writer)?;
        Var::from(self.chunk_y).encode(writer)?;
        (self.records.len() as u16).encode(writer)?;
        (self.records.len() as i32 * 4).encode(writer)?;
        for record in &self.records {
            record.encode(writer)?;
        }
        Ok(())
    }
}

#[derive(ToStatic)]
pub struct MultiBlockChange4 {
    pub chunk_x: i32,
    pub chunk_y: i32,
    // count(u16)
    pub records: Vec<Record>,
}

impl<'dec> Decode<'dec> for MultiBlockChange4 {
    fn decode(cursor: &mut std::io::Cursor<&'dec [u8]>) -> decode::Result<Self> {
        let chunk_x = i32::decode(cursor)?;
        let chunk_y = i32::decode(cursor)?;
        let record_count = u16::decode(cursor)?;
        let data_size: i32 = i32::decode(cursor)?;
        if data_size != record_count as i32 * 4 {
            // todo! different error
            return Err(decode::Error::InvalidId);
        }
        let records: Vec<_> = (0..record_count)
            .map(|_| Record::decode(cursor))
            .collect::<Result<_, _>>()?;
        Ok(Self {
            chunk_x,
            chunk_y,
            records,
        })
    }
}

impl Encode for MultiBlockChange4 {
    fn encode(&self, writer: &mut impl std::io::Write) -> Result<(), encode::Error> {
        self.chunk_x.encode(writer)?;
        self.chunk_y.encode(writer)?;
        (self.records.len() as u16).encode(writer)?;
        (self.records.len() as i32 * 4).encode(writer)?;
        for record in &self.records {
            record.encode(writer)?;
        }
        Ok(())
    }
}

#[derive(ToStatic)]
pub struct Record {
    pub block_state: u16,
    y: u8,
    pub rel_x: u8,
    pub rel_z: u8,
}

impl<'dec> Decode<'dec> for Record {
    fn decode(cursor: &'_ mut std::io::Cursor<&[u8]>) -> decode::Result<Self> {
        let xz = u8::decode(cursor)?;
        Ok(Record {
            rel_z: xz >> 4,
            rel_x: xz & 0b1111,
            y: u8::decode(cursor)?,
            block_state: u16::decode(cursor)?,
        })
    }
}

impl Encode for Record {
    fn encode(&self, writer: &mut impl std::io::Write) -> Result<(), encode::Error> {
        ((self.rel_x & 0b1111) + (self.rel_z << 4)).encode(writer)?;
        self.y.encode(writer)?;
        self.block_state.encode(writer)?;
        Ok(())
    }
}

#[derive(Encoding, ToStatic)]
pub struct BlockChange0 {
    pub x: i32,
    y: u8,
    pub z: i32,
    #[varint]
    // todo! extract the next two variables into concrete types for the version
    pub block_type: i32,
    pub block_data: u8,
}

#[derive(Encoding, ToStatic)]
pub struct BlockChange6 {
    pub location: Position,
    // todo! global palette block id, maybe separate into id and type?
    // https://wiki.vg/index.php?title=Protocol&oldid=7368#Block_Change
    #[varint]
    pub block_id: i32,
}

#[derive(Encoding, ToStatic)]
pub struct BlockAction0 {
    pub x: i32,
    pub y: i16,
    pub z: i32,
    pub action_id: u8,
    pub action_param: u8,
    /// The block type ID for the block, not including metadata/damage value
    #[varint]
    pub block_type: i32,
}

#[derive(Encoding, ToStatic)]
pub struct BlockAction6 {
    pub location: Position,
    pub action_id: u8,
    pub action_param: u8,
    /// The block type ID for the block, not including metadata/damage value
    #[varint]
    pub block_type: i32,
}

#[derive(Encoding, ToStatic)]
pub struct BlockBreakAnimation0 {
    #[varint]
    pub entity_id: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    /// 0-9
    pub destroy_stage: u8,
}

#[derive(Encoding, ToStatic)]
pub struct BlockBreakAnimation6 {
    #[varint]
    pub entity_id: i32,
    pub location: Position,
    /// 0-9
    pub destroy_stage: u8,
}

#[derive(ToStatic)]
pub struct MapChunkBulk0<'a> {
    /// Whether or not the chunk data contains a light nibble array. This is
    /// true in the main world, false in the end + nether
    pub skylight_sent: bool,
    pub data: Cow<'a, [u8]>,
    pub column_metas: Vec<ChunkMeta0>,
}
impl<'dec, 'a> Decode<'dec> for MapChunkBulk0<'a>
where
    'dec: 'a,
{
    fn decode(cursor: &'_ mut std::io::Cursor<&'dec [u8]>) -> decode::Result<Self> {
        let column_count = u16::decode(cursor)?;
        let data_len = u32::decode(cursor)?;
        let skylight_sent = bool::decode(cursor)?;
        let data = cursor
            .get_ref()
            .get(0..data_len as usize + cursor.position() as usize)
            // todo! different error
            .ok_or(decode::Error::InvalidId)?;
        let column_metas = (0..column_count)
            .map(|_| ChunkMeta0::decode(cursor))
            .collect::<Result<_, _>>()?;
        Ok(Self {
            skylight_sent,
            data: Cow::Borrowed(data),
            column_metas,
        })
    }
}
impl<'a> Encode for MapChunkBulk0<'a> {
    fn encode(&self, writer: &mut impl std::io::Write) -> Result<(), encode::Error> {
        (self.column_metas.len() as u16).encode(writer)?;
        (self.data.len() as u32).encode(writer)?;
        self.skylight_sent.encode(writer)?;
        writer.write_all(&self.data)?;
        for meta in &self.column_metas {
            meta.encode(writer)?;
        }
        Ok(())
    }
}

#[derive(Encoding, ToStatic)]
pub struct ChunkMeta0 {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub primary_bitmap: u16,
    pub add_bitmap: u16,
}

#[derive(Encoding, ToStatic)]
pub struct Explosion0 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub radius: f32,
    #[counted(u32)]
    pub records: Vec<ExplosionRecord>,
    pub motion_x: f32,
    pub motion_y: f32,
    pub motion_z: f32,
}

#[derive(Encoding, ToStatic)]
pub struct ExplosionRecord {
    pub dx: i8,
    pub dy: i8,
    pub dz: i8,
}

#[derive(Encoding, ToStatic)]
// todo! more detailed data using #[separated]
pub struct Effect0 {
    pub effect_id: i32,
    // todo! relative? fixed point?
    /// The X location of the effect multiplied by 8
    pub x: i32,
    /// The Y location of the effect multiplied by 8
    pub y: i8,
    /// The Z location of the effect multiplied by 8
    pub z: i32,
    pub effect_data: i32,
    pub disable_rel_volume: bool,
}

#[derive(Encoding, ToStatic)]
// todo! see above
pub struct Effect6 {
    pub effect_id: i32,
    pub location: Position,
    pub effect_data: i32,
    pub disable_rel_volume: bool,
}

#[derive(Encoding, ToStatic)]
pub struct SoundEffect0<'a> {
    pub effect_id: Cow<'a, str>,
    // todo! relative? fixed point?
    /// The X location of the effect multiplied by 8
    pub x: i32,
    /// The Y location of the effect multiplied by 8
    pub y: i32,
    /// The Z location of the effect multiplied by 8
    pub z: i32,
    /// 1 is 100%, can be more
    pub volume: f32,
    /// 63 is 100%, can be more
    pub pitch: u8,
    pub category: SoundCategory0,
}

#[derive(Encoding, ToStatic)]
#[from(u8)]
pub enum SoundCategory0 {
    Master = 0,
    Music,
    Records,
    Weather,
    Blocks,
    Mobs,
    Animals,
    Players,
}

#[derive(Encoding, ToStatic)]
pub struct SoundEffect1<'a> {
    pub effect_id: Cow<'a, str>,
    // todo! relative? fixed point?
    /// The X location of the effect multiplied by 8
    pub x: i32,
    /// The Y location of the effect multiplied by 8
    pub y: i32,
    /// The Z location of the effect multiplied by 8
    pub z: i32,
    /// 1 is 100%, can be more
    pub volume: f32,
    /// 63 is 100%, can be more
    pub pitch: u8,
}

#[derive(Encoding, ToStatic)]
pub struct Particle0<'a> {
    // todo! specific strings into enum
    pub name: Cow<'a, str>,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    /// This is added to the X position after being multiplied by random.nextGaussian()
    pub offset_x: f32,
    /// This is added to the Y position after being multiplied by random.nextGaussian()
    pub offset_y: f32,
    /// This is added to the Z position after being multiplied by random.nextGaussian()
    pub offset_z: f32,
    pub speed: f32,
    pub number: i32,
}

#[derive(Encoding, ToStatic)]
pub struct Particle17<'a> {
    // todo! specific strings into enum
    pub name: Cow<'a, str>,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    /// This is added to the X position after being multiplied by random.nextGaussian()
    pub offset_x: f32,
    /// This is added to the Y position after being multiplied by random.nextGaussian()
    pub offset_y: f32,
    /// This is added to the Z position after being multiplied by random.nextGaussian()
    pub offset_z: f32,
    pub speed: f32,
    pub number: i32,
    // todo! read exact number of varints using enum of possible names
    // https://wiki.vg/index.php?title=Protocol&oldid=7368#Particle_2
    #[rest]
    pub data: Cow<'a, [u8]>,
}

// #[derive(Encoding, ToStatic)]
// struct ChangeGameState0 {
//     reason: GameStateChangeReason,
// }

// #[derive(Encoding, ToStatic)]
// #[from(u8)]
#[derive(ToStatic)]
pub enum ChangeGameState0 {
    // #[case(0)]
    InvalidBed,
    BeginRaining,
    EndRaining,
    ChangeGameMode(GameMode0),
    EnterCredits,
    DemoMessage(DemoMessage0),
    BowHitSound,
    /// The current darkness value. 1 = Dark, 0 = Bright, Setting the value higher causes the game to change color and freeze
    FadeValue(f32),
    /// Time in ticks for the sky to fade
    FadeTime(f32),
}

#[derive(Encoding, ToStatic, Clone, Copy)]
#[from(u8)]
pub enum DemoMessage0 {
    WelcomeToDemo = 0,
    MovementControl = 101,
    JumpControl,
    InventoryControl,
}

impl<'dec> Decode<'dec> for ChangeGameState0 {
    fn decode(cursor: &'_ mut std::io::Cursor<&[u8]>) -> decode::Result<Self> {
        let reason = u8::decode(cursor)?;
        let value = f32::decode(cursor)?;
        use self::DemoMessage0::*;
        use ChangeGameState0::*;
        Ok(match reason {
            0 => InvalidBed,
            1 => BeginRaining,
            2 => EndRaining,
            3 => ChangeGameMode(match value as u8 {
                0 => GameMode0::Survival,
                1 => GameMode0::Creative,
                2 => GameMode0::Adventure,
                _ => return Err(decode::Error::InvalidId),
            }),
            4 => EnterCredits,
            5 => DemoMessage(match value as u8 {
                0 => WelcomeToDemo,
                101 => MovementControl,
                102 => JumpControl,
                103 => InventoryControl,
                _ => return Err(decode::Error::InvalidId),
            }),
            6 => BowHitSound,
            7 => FadeValue(value),
            8 => FadeTime(value),
            _ => return Err(decode::Error::InvalidId),
        })
    }
}

impl Encode for ChangeGameState0 {
    fn encode(&self, writer: &mut impl std::io::Write) -> Result<(), encode::Error> {
        let (reason, value) = match self {
            ChangeGameState0::InvalidBed => (0u8, 0.0),
            ChangeGameState0::BeginRaining => (1, 0.0),
            ChangeGameState0::EndRaining => (2, 0.0),
            ChangeGameState0::ChangeGameMode(gamemode) => (3, *gamemode as u8 as f32),
            ChangeGameState0::EnterCredits => (4, 0.0),
            ChangeGameState0::DemoMessage(demomessage) => (5, *demomessage as u8 as f32),
            ChangeGameState0::BowHitSound => (6, 0.0),
            ChangeGameState0::FadeValue(value) => (7, *value),
            ChangeGameState0::FadeTime(value) => (8, *value),
        };
        reason.encode(writer)?;
        value.encode(writer)?;
        Ok(())
    }
}

#[derive(Encoding, ToStatic)]
pub struct SpawnGlobalEntity0 {
    #[varint]
    pub entity_id: i32,
    /// The global entity type, currently always 1 for thunderbolt.
    pub kind: u8,
    #[fixed(5, i32)]
    pub x: f64,
    #[fixed(5, i32)]
    pub y: f64,
    #[fixed(5, i32)]
    pub z: f64,
}

#[derive(ToStatic)]
pub struct OpenWindow0<'a> {
    pub window_id: u8,
    pub kind: InventoryKind0,
    pub title: Cow<'a, str>,
    pub slot_count: u8,
    pub use_title: bool,
}

impl<'dec, 'a> Decode<'dec> for OpenWindow0<'a>
where
    'dec: 'a,
{
    fn decode(cursor: &mut std::io::Cursor<&'dec [u8]>) -> decode::Result<Self> {
        let window_id = u8::decode(cursor)?;
        let kind = u8::decode(cursor)?;
        let title = Cow::decode(cursor)?;
        let slot_count = u8::decode(cursor)?;
        let use_title = bool::decode(cursor)?;
        use InventoryKind0::*;
        let kind = match kind {
            0 => Chest,
            1 => CraftingTable,
            2 => Furnace,
            3 => Dispenser,
            4 => EnchantmentTable,
            5 => BrewingStand,
            6 => Villager,
            7 => Beacon,
            8 => Anvil,
            9 => Hopper,
            10 => Dropper,
            11 => Horse {
                entity_id: i32::decode(cursor)?,
            },
            _ => return Err(decode::Error::InvalidId),
        };
        Ok(Self {
            window_id,
            kind,
            title,
            slot_count,
            use_title,
        })
    }
}
impl<'a> Encode for OpenWindow0<'a> {
    fn encode(&self, cursor: &mut impl ::std::io::Write) -> Result<(), encode::Error> {
        let Self {
            window_id,
            kind,
            title,
            slot_count,
            use_title,
        } = self;
        use InventoryKind0::*;
        let (kind, entity_id) = match kind {
            Chest => (0, None),
            CraftingTable => (1, None),
            Furnace => (2, None),
            Dispenser => (3, None),
            EnchantmentTable => (4, None),
            BrewingStand => (5, None),
            Villager => (6, None),
            Beacon => (7, None),
            Anvil => (8, None),
            Hopper => (9, None),
            Dropper => (10, None),
            Horse { entity_id } => (11, Some(entity_id)),
        };
        window_id.encode(cursor)?;
        kind.encode(cursor)?;
        title.encode(cursor)?;
        slot_count.encode(cursor)?;
        use_title.encode(cursor)?;
        if let Some(entity_id) = entity_id {
            Encode::encode(entity_id, cursor)?;
        }
        Ok(())
    }
}

#[derive(ToStatic)]
pub struct OpenWindow6<'a> {
    pub window_id: u8,
    pub kind: InventoryKind6,
    // chat component? at least starting pv13
    pub title: Cow<'a, str>,
    pub slot_count: u8,
}

impl<'dec, 'a> Decode<'dec> for OpenWindow6<'a>
where
    'dec: 'a,
{
    fn decode(cursor: &mut std::io::Cursor<&'dec [u8]>) -> decode::Result<Self> {
        let window_id = u8::decode(cursor)?;
        let kind = <&str>::decode(cursor)?;
        let title = Cow::decode(cursor)?;
        let slot_count = u8::decode(cursor)?;
        use InventoryKind6::*;
        let kind = match kind {
            "minecraft:chest" => Chest,
            "minecraft:crafting_table" => CraftingTable,
            "minecraft:furnace" => Furnace,
            "minecraft:dispenser" => Dispenser,
            "minecraft:enchanting_table" => EnchantmentTable,
            "minecraft:brewing_stand" => BrewingStand,
            "minecraft:villager" => Villager,
            "minecraft:beacon" => Beacon,
            "minecraft:anvil" => Anvil,
            "minecraft:hopper" => Hopper,
            "minecraft:dropper" => Dropper,
            "EntityHorse" => Horse {
                entity_id: i32::decode(cursor)?,
            },
            _ => return Err(decode::Error::InvalidId),
        };
        Ok(Self {
            window_id,
            kind,
            title,
            slot_count,
        })
    }
}
impl<'a> Encode for OpenWindow6<'a> {
    fn encode(&self, writer: &mut impl std::io::Write) -> encode::Result<()> {
        self.window_id.encode(writer)?;
        use InventoryKind6::*;
        let (kind, entity_id) = match self.kind {
            Chest => ("minecraft:chest", None),
            CraftingTable => ("minecraft:crafting_table", None),
            Furnace => ("minecraft:furnace", None),
            Dispenser => ("minecraft:dispenser", None),
            EnchantmentTable => ("minecraft:enchanting_table", None),
            BrewingStand => ("minecraft:brewing_stand", None),
            Villager => ("minecraft:villager", None),
            Beacon => ("minecraft:beacon", None),
            Anvil => ("minecraft:anvil", None),
            Hopper => ("minecraft:hopper", None),
            Dropper => ("minecraft:dropper", None),
            Horse { entity_id } => ("EntityHorse", Some(entity_id)),
        };
        kind.encode(writer)?;
        self.title.encode(writer)?;
        self.slot_count.encode(writer)?;
        if let Some(entity_id) = entity_id {
            entity_id.encode(writer)?;
        }
        Ok(())
    }
}

// #[derive(Encoding, ToStatic)]
#[derive(ToStatic)]
// todo! very good place for #[separate]
pub enum InventoryKind0 {
    /// Chest, large chest, or minecart with chest
    // #[from(0)]
    Chest,
    CraftingTable,
    Furnace,
    Dispenser,
    EnchantmentTable,
    BrewingStand,
    Villager,
    Beacon,
    Anvil,
    /// Hopper or minecart with hopper
    Hopper,
    Dropper,
    /// Horse, donkey, or mule
    Horse {
        entity_id: i32,
    },
}

// #[derive(Encoding, ToStatic)]
#[derive(ToStatic)]
// todo! very good place for #[separate]
pub enum InventoryKind6 {
    /// Chest, large chest, or minecart with chest
    // #[from(0)]
    Chest,
    CraftingTable,
    Furnace,
    Dispenser,
    EnchantmentTable,
    BrewingStand,
    Villager,
    Beacon,
    Anvil,
    /// Hopper or minecart with hopper
    Hopper,
    Dropper,
    /// Horse, donkey, or mule
    Horse {
        entity_id: i32,
    },
}

#[derive(Encoding, ToStatic)]
pub struct CloseWindow0 {
    /// This is the id of the window that was closed. 0 for inventory.
    pub window_id: u8,
}

#[derive(Encoding, ToStatic)]
pub struct SetSlot0 {
    /// The window which is being updated. 0 for player inventory. Note that
    /// all known window types include the player inventory. This packet will
    /// only be sent for the currently opened window while the player is
    /// performing actions, even if it affects the player inventory. After the
    /// window is closed, a number of these packets are sent to update the
    /// player's inventory window (0).
    pub window_id: u8,
    /// The slot that should be updated
    pub slot: u16,
    // todo! slot data
    // data: Slot
}

#[derive(Encoding, ToStatic)]
pub struct WindowItems0 {
    /// The id of window which items are being sent for. 0 for player inventory.
    pub window_id: u8,
    // #[counted(u16)]
    // todo! slot data
    // slots: Vec<Slot>
}

#[derive(Encoding, ToStatic)]
/// see https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5007#Window_Property
pub struct WindowProperty0 {
    pub window_id: u8,
    pub property: u16,
    pub value: u16,
}

#[derive(Encoding, ToStatic)]
pub struct ConfirmTransaction0 {
    pub window_id: u8,
    pub action_number: i16,
    pub accepted: bool,
}

#[derive(Encoding, ToStatic)]
pub struct UpdateSign0<'a> {
    pub x: i32,
    pub y: i16,
    pub z: i32,
    pub line1: Cow<'a, str>,
    pub line2: Cow<'a, str>,
    pub line3: Cow<'a, str>,
    pub line4: Cow<'a, str>,
}

#[derive(Encoding, ToStatic)]
pub struct UpdateSign6<'a> {
    pub location: Position,
    pub line1: Cow<'a, str>,
    pub line2: Cow<'a, str>,
    pub line3: Cow<'a, str>,
    pub line4: Cow<'a, str>,
}

#[derive(Encoding, ToStatic)]
pub struct Maps0 {
    #[varint]
    pub item_damage: i32,
    // todo! #[rest]
    // todo! impl MapData
    // map_data: MapData<'a>,
}

// todo! WTF
// https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5007#Maps
// enum MapData<'a> {
//     ColorColumn{
//         start_x: u8,
//         start_y: u8,
//         color: &'a [u8]
//     },
//     MapScale(u8)
// }

#[derive(Encoding, ToStatic)]
pub struct UpdateBlockEntity0 {
    pub x: i32,
    pub y: i16,
    pub z: i32,
    /// The type of update to perform
    pub action: u8,
    /// varies
    pub data_length: u16,
    // Present if data length > 0. Compressed with gzip. Varies
    // todo! nbt
    // data: Nbt
}

#[derive(Encoding, ToStatic)]
pub struct UpdateBlockEntity6 {
    pub location: Position,
    /// The type of update to perform
    pub action: u8,
    /// varies
    pub data_length: u16,
    // Present if data length > 0. Compressed with gzip. Varies
    // todo! nbt
    // data: Nbt
}

#[derive(Encoding, ToStatic)]
pub struct SignEditorOpen0 {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Encoding, ToStatic)]
#[bitfield]
pub struct SignEditorOpen6 {
    #[bits(26)]
    pub x: i32,
    #[bits(26)]
    pub z: i32,
    #[bits(12)]
    pub y: i16,
}

#[derive(Encoding, ToStatic)]
pub struct Statistics0<'a> {
    pub entries: Vec<Statistic0<'a>>,
}

#[derive(Encoding, ToStatic)]
pub struct Statistic0<'a> {
    pub name: Cow<'a, str>,
    #[varint]
    /// The amount to increase by
    pub amount: i32,
}

#[derive(Encoding, ToStatic)]
pub struct PlayerListItem0<'a> {
    /// Supports chat colouring, limited to 16 characters.
    pub name: Cow<'a, str>,
    /// The client will remove the user from the list if false.
    pub online: bool,
    /// Ping, presumably in ms.
    pub ping: i16,
}

#[derive(Encoding, ToStatic)]
pub struct PlayerListItem7<'a> {
    /// Supports chat colouring, limited to 16 characters.
    pub name: Cow<'a, str>,
    /// The client will remove the user from the list if false.
    pub online: bool,
    /// Ping, presumably in ms.
    #[varint]
    pub ping: i32,
}

#[derive(Encoding, ToStatic)]
pub enum PlayerListItem17<'a> {
    #[case(0)]
    AddPlayers(Vec<PlayerListAddPlayer17<'a>>),
    UpdateGamemode(Vec<PlayerListUpdateGamemode17>),
    UpdateLatency(Vec<PlayerListUpdateLatency17>),
}

#[derive(Encoding, ToStatic)]
pub struct PlayerListAddPlayer17<'a> {
    pub uuid: Uuid,
    pub name: Cow<'a, str>,
    pub gamemode: GameMode17,
    #[varint]
    pub ping: i32,
}

#[derive(Encoding, ToStatic)]
pub struct PlayerListUpdateGamemode17 {
    pub uuid: Uuid,
    pub gamemode: GameMode17,
}

#[derive(Encoding, ToStatic)]
pub struct PlayerListUpdateLatency17 {
    pub uuid: Uuid,
    #[varint]
    pub ping: i32,
}

#[derive(Encoding, ToStatic)]
pub enum PlayerListItem19<'a> {
    #[case(0)]
    AddPlayers(Vec<PlayerListAddPlayer19<'a>>),
    UpdateGamemode(Vec<PlayerListUpdateGamemode17>),
    UpdateLatency(Vec<PlayerListUpdateLatency17>),
    RemovePlayers(Vec<Uuid>),
}

#[derive(Encoding, ToStatic)]
pub struct PlayerListAddPlayer19<'a> {
    pub uuid: Uuid,
    pub name: Cow<'a, str>,
    pub properties: Vec<PlayerProperty19<'a>>,
    pub gamemode: GameMode17,
    #[varint]
    pub ping: i32,
}

#[derive(ToStatic)]
pub struct PlayerProperty19<'a> {
    pub name: Cow<'a, str>,
    pub value: Cow<'a, str>,
    // todo! default Option impl?
    pub signature: Option<Cow<'a, str>>,
}
impl<'dec, 'a> Decode<'dec> for PlayerProperty19<'a>
where
    'dec: 'a,
{
    fn decode(cursor: &mut std::io::Cursor<&'dec [u8]>) -> decode::Result<Self> {
        Ok(Self {
            name: Cow::decode(cursor)?,
            value: Cow::decode(cursor)?,
            signature: if bool::decode(cursor)? {
                Some(Cow::decode(cursor)?)
            } else {
                None
            },
        })
    }
}
impl<'a> Encode for PlayerProperty19<'a> {
    fn encode(&self, writer: &mut impl std::io::Write) -> encode::Result<()> {
        self.name.encode(writer)?;
        self.value.encode(writer)?;
        if let Some(signature) = &self.signature {
            true.encode(writer)?;
            signature.encode(writer)?;
        } else {
            false.encode(writer)?;
        };
        Ok(())
    }
}

#[derive(Encoding, ToStatic, Clone, Copy)]
pub enum GameMode17 {
    Survival = 0,
    Creative,
    Adventure,
}

#[derive(ToStatic)]
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
impl<'dec> Decode<'dec> for PlayerAbilities0 {
    fn decode(cursor: &mut std::io::Cursor<&'dec [u8]>) -> Result<Self, decode::Error> {
        let flags = u8::decode(cursor)?;
        Ok(PlayerAbilities0 {
            invulnerable: flags & 0b0001 != 0,
            flying: flags & 0b0010 != 0,
            allow_flying: flags & 0b0100 != 0,
            creative_mode: flags & 0b1000 != 0,
            flying_speed: f32::decode(cursor)?,
            fov: f32::decode(cursor)?,
        })
    }
}
impl Encode for PlayerAbilities0 {
    fn encode(&self, writer: &mut impl std::io::Write) -> Result<(), encode::Error> {
        ((self.invulnerable as u8)
            + ((self.flying as u8) << 1)
            + ((self.allow_flying as u8) << 2)
            + ((self.creative_mode as u8) << 3))
            .encode(writer)?;
        self.flying_speed.encode(writer)?;
        self.fov.encode(writer)?;
        Ok(())
    }
}

#[derive(Encoding, ToStatic)]
pub struct TabComplete0<'a> {
    /// One eligible command
    pub matches: Vec<Cow<'a, str>>,
}

#[derive(Encoding, ToStatic)]
pub struct ScoreboardObjective0<'a> {
    pub name: Cow<'a, str>,
    pub value: Cow<'a, str>,
    pub action: ScoreboardAction0,
}
#[derive(Encoding, ToStatic)]
#[from(u8)]
pub enum ScoreboardAction0 {
    #[case(0)]
    Create,
    Remove,
    Update,
}

#[derive(Encoding, ToStatic)]
pub struct ScoreboardObjective12<'a> {
    pub name: Cow<'a, str>,
    pub value: Cow<'a, str>,
    pub action: ScoreboardAction0,
}

#[derive(Encoding, ToStatic)]
#[from(u8)]
pub enum ScoreboardObjectiveAction12<'a> {
    #[case(0)]
    Create {
        value: Cow<'a, str>,
        kind: ScoreboardObjectiveKind12,
    },
    Remove,
    Update {
        value: Cow<'a, str>,
        kind: ScoreboardObjectiveKind12,
    },
}

// todo! check that there aren't any other cases in later supported protocol versions
// support up to pv66
#[derive(Encoding, ToStatic)]
#[from(&str)]
pub enum ScoreboardObjectiveKind12 {
    #[case("integer")]
    Integer,
    #[case("hearts")]
    Hearts,
}

#[derive(Encoding, ToStatic)]
pub struct UpdateScore0<'a> {
    /// The name of the score to be updated or removed
    pub name: Cow<'a, str>,
    pub action: UpdateScoreAction0<'a>,
}

#[derive(Encoding, ToStatic)]
#[from(u8)]
pub enum UpdateScoreAction0<'a> {
    #[case(0)]
    Update {
        /// The name of the objective the score belongs to
        text: Cow<'a, str>,
        /// The score to be displayed next to the entry
        kind: i32,
    },
    Remove,
}

#[derive(Encoding, ToStatic)]
pub struct UpdateScore7<'a> {
    /// The name of the score to be updated or removed
    pub name: Cow<'a, str>,
    pub action: UpdateScoreAction7<'a>,
}

#[derive(Encoding, ToStatic)]
#[from(u8)]
pub enum UpdateScoreAction7<'a> {
    #[case(0)]
    Update {
        /// The name of the objective the score belongs to
        text: Cow<'a, str>,
        /// The score to be displayed next to the entry
        #[varint]
        kind: i32,
    },
    Remove,
}

#[derive(Encoding, ToStatic)]
pub struct DisplayScoreboard0<'a> {
    pub position: ScoreboardPosition,
    pub name: Cow<'a, str>,
}

#[derive(Encoding, ToStatic)]
#[from(u8)]
pub enum ScoreboardPosition {
    List = 0,
    Sidebar,
    BelowName,
}

#[derive(Encoding, ToStatic)]
pub struct Teams0<'a> {
    pub name: Cow<'a, str>,
    pub action: TeamAction0<'a>,
}

#[derive(Encoding, ToStatic)]
pub enum TeamAction0<'a> {
    #[case(0)]
    Create {
        display_name: Cow<'a, str>,
        prefix: Cow<'a, str>,
        suffix: Cow<'a, str>,
        friendly_fire: TeamFriendlyFire,
        #[counted(u16)]
        players: Vec<Cow<'a, str>>,
    },
    Remove,
    Update {
        display_name: Cow<'a, str>,
        prefix: Cow<'a, str>,
        suffix: Cow<'a, str>,
        friendly_fire: TeamFriendlyFire,
    },
    AddPlayers {
        #[counted(u16)]
        players: Vec<Cow<'a, str>>,
    },
    RemovePlayers {
        #[counted(u16)]
        players: Vec<Cow<'a, str>>,
    },
}

#[derive(Encoding, ToStatic)]
pub struct Teams7<'a> {
    pub name: Cow<'a, str>,
    pub action: TeamAction7<'a>,
}

#[derive(Encoding, ToStatic)]
pub enum TeamAction7<'a> {
    #[case(0)]
    Create {
        display_name: Cow<'a, str>,
        prefix: Cow<'a, str>,
        suffix: Cow<'a, str>,
        friendly_fire: TeamFriendlyFire,
        players: Vec<Cow<'a, str>>,
    },
    Remove,
    Update {
        display_name: Cow<'a, str>,
        prefix: Cow<'a, str>,
        suffix: Cow<'a, str>,
        friendly_fire: TeamFriendlyFire,
    },
    AddPlayers {
        players: Vec<Cow<'a, str>>,
    },
    RemovePlayers {
        players: Vec<Cow<'a, str>>,
    },
}

#[derive(Encoding, ToStatic)]
pub struct Teams11<'a> {
    pub name: Cow<'a, str>,
    pub action: TeamAction11<'a>,
}

#[derive(Encoding, ToStatic)]
pub enum TeamAction11<'a> {
    #[case(0)]
    Create {
        display_name: Cow<'a, str>,
        prefix: Cow<'a, str>,
        suffix: Cow<'a, str>,
        friendly_fire: TeamFriendlyFire,
        name_tag_vis: NameTagVisibility11,
        /// Same as Chat colors
        color: u8,
        players: Vec<Cow<'a, str>>,
    },
    Remove,
    Update {
        display_name: Cow<'a, str>,
        prefix: Cow<'a, str>,
        suffix: Cow<'a, str>,
        friendly_fire: TeamFriendlyFire,
        name_tag_vis: NameTagVisibility11,
        /// Same as Chat colors
        color: u8,
    },
    AddPlayers {
        players: Vec<Cow<'a, str>>,
    },
    RemovePlayers {
        players: Vec<Cow<'a, str>>,
    },
}

#[derive(Encoding, ToStatic)]
#[from(u8)]
pub enum TeamFriendlyFire {
    Off = 0,
    On,
    FriendliesVisible = 3,
}

#[derive(Encoding, ToStatic)]
#[from(&str)]
pub enum NameTagVisibility11 {
    #[case("always")]
    Always,
    #[case("hideForOtherTeams")]
    HideForOtherTeams,
    #[case("hideForOwnTeam")]
    HideForOwnTeam,
    #[case("never")]
    Never,
}

#[derive(Encoding, ToStatic)]
// https://dinnerbone.com/blog/2012/01/13/minecraft-plugin-channels-messaging/
pub struct PluginMessage0<'a> {
    pub channel: Cow<'a, str>,
    pub data: Cow<'a, [u8]>,
}

#[derive(Encoding, ToStatic)]
pub struct Disconnect0<'a> {
    // chatcomponent, at least starting pv13
    pub reason: Cow<'a, str>,
}

#[derive(Encoding, ToStatic)]
pub struct ServerDifficulty6 {
    pub difficulty: Difficulty0,
}

#[derive(Encoding, ToStatic)]
#[from(i32)]
pub enum CombatEvent7<'a> {
    #[case(0)]
    EnterCombat,
    EndCombat {
        #[varint]
        duration: i32,
        entity_id: i32,
    },
    EntityDead {
        #[varint]
        player_id: i32,
        entity_id: i32,
        message: Cow<'a, str>,
    },
}

#[derive(Encoding, ToStatic)]
#[from(u8)]
pub enum CombatEvent8<'a> {
    #[case(0)]
    EnterCombat,
    EndCombat {
        #[varint]
        duration: i32,
        entity_id: i32,
    },
    EntityDead {
        #[varint]
        player_id: i32,
        entity_id: i32,
        message: Cow<'a, str>,
    },
}

#[derive(Encoding, ToStatic)]
pub struct Camera9 {
    #[varint]
    pub entity_id: i32,
}

#[derive(Encoding, ToStatic)]
pub enum WorldBorder15 {
    #[case(0)]
    SetSize {
        radius: f64,
    },
    LerpSize {
        old_radius: f64,
        new_radius: f64,
        /// number of real-time ticks/seconds (?) until New Radius is reached.
        /// From experiments, it appears that Notchian server does not sync
        /// world border speed to game ticks, so it gets out of sync with
        /// server lag
        #[varint]
        speed: i32,
    },
    SetCenter {
        x: f64,
        z: f64,
    },
}

#[derive(Encoding, ToStatic)]
pub enum WorldBorder16 {
    #[case(0)]
    SetSize {
        radius: f64,
    },
    LerpSize {
        old_radius: f64,
        new_radius: f64,
        /// number of real-time ticks/seconds (?) until New Radius is reached.
        /// From experiments, it appears that Notchian server does not sync
        /// world border speed to game ticks, so it gets out of sync with
        /// server lag
        #[varint]
        speed: i32,
    },
    SetCenter {
        x: f64,
        z: f64,
    },
    Initialize {
        x: f64,
        z: f64,
        old_radius: f64,
        new_radius: f64,
        #[varint]
        speed: i32,
        /// Resulting coordinates from a portal teleport are limited to +-value. Usually 29999984.
        #[varint]
        portal_tp_boundary: i32,
    },
}

#[derive(Encoding, ToStatic)]
pub enum WorldBorder17 {
    #[case(0)]
    SetSize {
        radius: f64,
    },
    LerpSize {
        old_radius: f64,
        new_radius: f64,
        /// number of real-time ticks/seconds (?) until New Radius is reached.
        /// From experiments, it appears that Notchian server does not sync
        /// world border speed to game ticks, so it gets out of sync with
        /// server lag
        #[varint]
        speed: i32,
    },
    SetCenter {
        x: f64,
        z: f64,
    },
    SetWarningTime {
        #[varint]
        warning_time: i32,
    },
    SetWarningBlocks {
        #[varint]
        warning_blocks: i32,
    },
    Initialize {
        x: f64,
        z: f64,
        old_radius: f64,
        new_radius: f64,
        #[varint]
        speed: i32,
        /// Resulting coordinates from a portal teleport are limited to +-value. Usually 29999984.
        #[varint]
        portal_tp_boundary: i32,
        #[varint]
        warning_time: i32,
        #[varint]
        warning_blocks: i32,
    },
}

#[derive(Encoding, ToStatic)]
#[from(u8)]
pub enum Title18<'a> {
    #[case(0)]
    SetTitle {
        text: Cow<'a, str>,
    },
    SetSubTitle {
        text: Cow<'a, str>,
    },
    SetTimesAndDisplay {
        /// ticks
        fade_in: i32,
        /// ticks
        stay: i32,
        /// ticks
        fade_out: i32,
    },
}
