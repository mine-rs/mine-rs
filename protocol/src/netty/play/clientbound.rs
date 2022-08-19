use std::borrow::Cow;

use crate::netty::{Angle, InvalidEnumId, ProtocolRead, ProtocolWrite, ReadError, Var, WriteError};
use protocol_derive::Protocol;
use uuid::Uuid;

#[derive(Protocol)]
struct KeepAlive0 {
    id: i32,
}

struct JoinGame0 {
    entity_id: i32,
    hardcore: bool,
    gamemode: GameMode0,
    dimension: Dimension0,
    difficulty: super::Difficulty0,
    max_players: u8,
}

impl<'read> ProtocolRead<'read> for JoinGame0 {
    fn read(cursor: &mut std::io::Cursor<&'read [u8]>) -> Result<Self, ReadError> {
        let entity_id = i32::read(cursor)?;
        let bitfield = u8::read(cursor)?;
        let hardcore = bitfield & 0x08 != 0;
        let gamemode = match bitfield & 0b11 {
            0 => GameMode0::Survival,
            1 => GameMode0::Adventure,
            2 => GameMode0::Creative,
            _ => return Err(InvalidEnumId.into()),
        };
        Ok(Self {
            entity_id,
            hardcore,
            gamemode,
            dimension: Dimension0::read(cursor)?,
            difficulty: Difficulty0::read(cursor)?,
            max_players: u8::read(cursor)?,
        })
    }
}

impl ProtocolWrite for JoinGame0 {
    fn write(self, writer: &mut impl ::std::io::Write) -> Result<(), WriteError> {
        self.entity_id.write(writer)?;
        (match self.gamemode {
            GameMode0::Survival => 0,
            GameMode0::Creative => 1,
            GameMode0::Adventure => 2,
        } & ((self.hardcore as u8) << 3))
            .write(writer)?;
        self.dimension.write(writer)?;
        self.difficulty.write(writer)?;
        self.max_players.write(writer)?;
        Ok(())
    }
    #[inline(always)]
    fn size_hint() -> usize {
        <i32 as ProtocolWrite>::size_hint()
            + <GameMode0 as ProtocolWrite>::size_hint()
            + <Difficulty0 as ProtocolWrite>::size_hint()
            + <u8 as ProtocolWrite>::size_hint()
    }
}


struct JoinGame1<'a> {
    entity_id: i32,
    hardcore: bool,
    gamemode: GameMode0,
    dimension: Dimension0,
    difficulty: super::Difficulty0,
    max_players: u8,
    /// "default", "flat", "largeBiomes", "amplified", "default_1_1"
    level_type: Cow<'a, str>,
}

impl<'a> ProtocolRead<'a> for JoinGame1<'a> {
    fn read(cursor: &mut std::io::Cursor<&'a [u8]>) -> Result<Self, ReadError> {
        let entity_id = i32::read(cursor)?;
        let bitfield = u8::read(cursor)?;
        let hardcore = bitfield & 0x08 != 0;
        let gamemode = match bitfield & 0b11 {
            0 => GameMode0::Survival,
            1 => GameMode0::Adventure,
            2 => GameMode0::Creative,
            _ => return Err(InvalidEnumId.into()),
        };
        Ok(Self {
            entity_id,
            hardcore,
            gamemode,
            dimension: Dimension0::read(cursor)?,
            difficulty: Difficulty0::read(cursor)?,
            max_players: u8::read(cursor)?,
            level_type: Cow::read(cursor)?,
        })
    }
}

impl ProtocolWrite for JoinGame1<'_> {
    fn write(self, writer: &mut impl ::std::io::Write) -> Result<(), WriteError> {
        self.entity_id.write(writer)?;
        (match self.gamemode {
            GameMode0::Survival => 0,
            GameMode0::Creative => 1,
            GameMode0::Adventure => 2,
        } & ((self.hardcore as u8) << 3))
            .write(writer)?;
        self.dimension.write(writer)?;
        self.difficulty.write(writer)?;
        self.max_players.write(writer)?;
        self.level_type.write(writer)?;
        Ok(())
    }
    #[inline(always)]
    fn size_hint() -> usize {
        <i32 as ProtocolWrite>::size_hint()
            + <GameMode0 as ProtocolWrite>::size_hint()
            + <Difficulty0 as ProtocolWrite>::size_hint()
            + <u8 as ProtocolWrite>::size_hint()
            + 1
    }
}

#[derive(Protocol)]
#[from(u8)]
enum GameMode0 {
    Survival = 0,
    Creative,
    Adventure,
}

pub use super::Difficulty0;

#[derive(Protocol)]
#[from(i8)]
enum Dimension0 {
    Nether = -1,
    Overworld = 0,
    End,
}

#[derive(Protocol)]
struct ChatMessage0 {
    // todo! add ChatMessage json thing
    message: String,
}

#[derive(Protocol)]
struct TimeUpdate0 {
    ticks: i64,
    time_of_day: i64,
}

#[derive(Protocol)]
struct EntityEquipment0 {
    entity_id: i32,
    slot: EquipmentSlot0,
    // todo! slot data
    // item: Slot,
}

#[derive(Protocol)]
#[from(u16)]
enum EquipmentSlot0 {
    Held = 0,
    Boots,
    Leggings,
    Chestplate,
    Helmet,
}

#[derive(Protocol)]
struct SpawnPosition0 {
    x: i32,
    y: i32,
    z: i32,
}

#[derive(Protocol)]
struct UpdateHealth0 {
    /// 0.0 means dead, 20.0 = full HP
    health: f32,
    /// 0-20
    food: i16,
    /// 0.0 to 5.0 in integer increments?
    saturation: f32,
}

#[derive(Protocol)]
struct Respawn0 {
    dimension: i32,
    difficulty: Difficulty0,
    // no hardcore flag here
    gamemode: GameMode0,
}

#[derive(Protocol)]
struct Respawn1<'a> {
    dimension: i32,
    difficulty: Difficulty0,
    // no hardcore flag here
    gamemode: GameMode0,
    /// "default", "flat", "largeBiomes", "amplified", "default_1_1"
    level_type: Cow<'a, str>,
}

#[derive(Protocol)]
struct PositionAndLook0 {
    x: f64,
    y: f64,
    z: f64,
    /// Absolute rotation on the X Axis, in degrees
    yaw: f32,
    /// Absolute rotation on the Y Axis, in degrees
    pitch: f32,
    on_ground: bool,
}

#[derive(Protocol)]
struct HeldItemChange0 {
    /// The slot which the player has selected (0-8)
    slot: u8,
}

#[derive(Protocol)]
struct UseBed0 {
    entity_id: i32,
    x: i32,
    y: i8,
    z: i32,
}

#[derive(Protocol)]
struct Animation0 {
    #[varint]
    entity_id: i32,
    animation: super::AnimationId0,
}

#[derive(Protocol)]
struct SpawnPlayer0<'a> {
    #[varint]
    entity_id: i32,
    player_uuid: Uuid,
    name: Cow<'a, str>,
    // todo! add #[fixed(5)]
    /// Player X as a Fixed-Point number
    x: i32,
    /// Player Y as a Fixed-Point number
    y: i32,
    /// Player Z as a Fixed-Point number
    z: i32,
    // todo! angle
    yaw: Angle,
    pitch: Angle,
    /// The item the player is currently holding. Note that this should be 0
    /// for "no item", unlike -1 used in other packets. A negative value
    /// crashes clients.
    current_item: u16,
    metadata: EntityMetadata,
}

#[derive(Protocol)]
// todo! metadata
struct EntityMetadata {}

#[derive(Protocol)]
struct CollectItem0 {
    collected_id: i32,
    collector_id: i32,
}

#[derive(Protocol)]
struct SpawnObject0 {
    #[varint]
    entity_id: i32,
    // todo! (see [`Object0`])
    kind: Object0,
    // todo! add #[fixed(5)]
    /// X position as a Fixed-Point number
    x: i32,
    /// Y position as a Fixed-Point number
    y: i32,
    /// Z position as a Fixed-Point number
    z: i32,
    pitch: Angle,
    yaw: Angle,
    // todo! should be covered by kind, look above
    data: Object0,
}

#[derive(Protocol)]
#[from(u8)]
// todo! add #[separated] to have a custom option for
// separated type and cursor/writer impl
// would produce a custom read like `read(kind: $from, cursor: Cursor<&[u8]>) -> Result`
// and write like `write_kind(&self, writer: impl Write) -> Result`
// and `write_self(self, writer: impl Write) -> Result
enum Object0 {
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

#[derive(Protocol)]
// todo
enum Orientation {}

#[derive(Protocol)]
enum EntityKind0 {
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

#[derive(Protocol)]
struct SpawnMob0 {
    #[varint]
    entity_id: i32,
    // todo! see #[separated] on Object
    kind: EntityKind0,
    // todo! see #[fixed]
    /// X position as a Fixed-Point number
    x: i32,
    /// Y position as a Fixed-Point number
    y: i32,
    /// Z position as a Fixed-Point number
    z: i32,
    pitch: Angle,
    head_pitch: Angle,
    yaw: Angle,
    velocity_x: i16,
    velocity_y: i16,
    velocity_z: i16,
    // todo! see type
    metadata: EntityMetadata,
}

#[derive(Protocol)]
struct SpawnPainting<'a> {
    #[varint]
    entity_id: i32,
    // todo! #[max_len(13)]
    /// Name of the painting. Max length 13
    title: Cow<'a, str>,
    x: i32,
    y: i32,
    z: i32,
    direction: Direction0,
}

#[derive(Protocol)]
#[from(u32)]
enum Direction0 {
    NegZ = 0,
    NegX,
    PosZ,
    PosX,
}

#[derive(Protocol)]
struct SpawnExpOrb0 {
    #[varint]
    entity_id: i32,
    // todo! see #[fixed(5)]
    x: i32,
    y: i32,
    z: i32,
    /// The amount of experience this orb will reward once collected
    count: i16,
}

#[derive(Protocol)]
struct EntityVelocity0 {
    entity_id: i32,
    // todo! is this fixed point?
    x: i16,
    y: i16,
    z: i16,
}

#[derive(Protocol)]
struct DestroyEntities0 {
    // todo! #[count(u8)]
    entities: Vec<i32>,
}

#[derive(Protocol)]
struct Entity0 {
    entity_id: i32,
}

#[derive(Protocol)]
struct EntityRelativeMove0 {
    entity_id: i32,
    // todo! see #[fixed(5)]
    dx: i8,
    dy: i8,
    dz: i8,
}

#[derive(Protocol)]
struct EntityLook0 {
    entity_id: i32,
    yaw: Angle,
    pitch: Angle,
}

#[derive(Protocol)]
struct EntityLookAndRelativeMove0 {
    entity_id: i32,
    // todo! see #[fixed(5)]
    dx: i8,
    dy: i8,
    dz: i8,
    yaw: Angle,
    pitch: Angle,
}

#[derive(Protocol)]
struct EntityTeleport0 {
    entity_id: i32,
    // todo! see #[fixed(5)]
    x: i32,
    y: i32,
    z: i32,
    yaw: Angle,
    pitch: Angle,
}

#[derive(Protocol)]
struct EntityHeadLook0 {
    entity_id: i32,
    head_yaw: Angle,
}

#[derive(Protocol)]
struct EntityStatus0 {
    entity_id: i32,
    entity_status: Status0,
}

#[derive(Protocol)]
#[from(u8)]
enum Status0 {
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

#[derive(Protocol)]
struct AttachEntity0 {
    entity_id: i32,
    vehicle_id: i32,
    leash: bool,
}

#[derive(Protocol)]
struct EntityMetadata0 {
    entity_id: i32,
    metadata: EntityMetadata,
}

#[derive(Protocol)]
struct EntityEffect0 {
    entity_id: i32,
    // todo! effect ids
    effect_id: i8,
    amplifier: i8,
    duration: i16,
}

#[derive(Protocol)]
struct RemoveEntityEffect0 {
    entity_id: i32,
    effect_id: i8,
}

#[derive(Protocol)]
struct SetExperience0 {
    experience_bar: f32,
    level: i16,
    total_exp: i16,
}

#[derive(Protocol)]
struct EntityProperties0<'a> {
    entity_id: i32,
    // todo! #[count(i32)]
    properties: Vec<EntityProperty<'a>>,
}

#[derive(Protocol)]
struct EntityProperty<'a> {
    key: Cow<'a, str>,
    value: f64,
    modifiers: Vec<Modifier>,
}

#[derive(Protocol)]
struct Modifier {
    uuid: Uuid,
    amount: f64,
    operation: i8,
}

#[derive(Protocol)]
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
enum ModifierOperation0 {
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

#[derive(Protocol)]
// todo! make this nice to interact with
struct ChunkData0 {
    chunk_x: i32,
    chunk_y: i32,
    /// This is True if the packet represents all sections in this vertical
    /// column, where the primary bit map specifies exactly which sections are
    /// included, and which are air
    continuous: bool,
    /// Bitmask with 1 for every 16x16x16 section which data follows in the compressed data.
    primary_bitmap: u16,
    // todo! waht is this for?
    /// Same as above, but this is used exclusively for the 'add' portion of the payload
    add_bitmap: u16,
    // todo! #[count(i32)]
    compressed_data: Vec<u8>,
}

struct MultiBlockChange0 {
    // varint
    chunk_x: i32,
    // varint
    chunk_y: i32,
    // count(u16)
    records: Vec<Record>,
}

impl ProtocolRead<'_> for MultiBlockChange0 {
    fn read(cursor: &'_ mut std::io::Cursor<&[u8]>) -> Result<Self, ReadError> {
        let chunk_x: i32 = Var::read(cursor)?.0;
        let chunk_y: i32 = Var::read(cursor)?.0;
        let record_count = u16::read(cursor)?;
        let data_size: i32 = i32::read(cursor)?;
        if data_size != record_count as i32 * 4 {
            // todo! different error
            return Err(ReadError::InvalidEnumId);
        }
        let records: Vec<_> = (0..record_count)
            .map(|_| Record::read(cursor))
            .collect::<Result<_, _>>()?;
        Ok(Self {
            chunk_x,
            chunk_y,
            records,
        })
    }
}

impl ProtocolWrite for MultiBlockChange0 {
    fn write(self, writer: &mut impl std::io::Write) -> Result<(), WriteError> {
        Var(self.chunk_x).write(writer)?;
        Var(self.chunk_y).write(writer)?;
        (self.records.len() as u16).write(writer)?;
        (self.records.len() as i32 * 4).write(writer)?;
        for record in self.records {
            record.write(writer)?;
        }
        Ok(())
    }

    fn size_hint() -> usize {
        8
    }
}

struct Record {
    block_state: u16,
    y: u8,
    rel_x: u8,
    rel_z: u8,
}

impl ProtocolRead<'_> for Record {
    fn read(cursor: &'_ mut std::io::Cursor<&[u8]>) -> Result<Self, ReadError> {
        let xz = u8::read(cursor)?;
        Ok(Record {
            rel_z: xz >> 4,
            rel_x: xz & 0b1111,
            y: u8::read(cursor)?,
            block_state: u16::read(cursor)?,
        })
    }
}

impl ProtocolWrite for Record {
    fn write(self, writer: &mut impl std::io::Write) -> Result<(), WriteError> {
        ((self.rel_x & 0b1111) + (self.rel_z << 4)).write(writer)?;
        self.y.write(writer)?;
        self.block_state.write(writer)?;
        Ok(())
    }

    fn size_hint() -> usize {
        4
    }
}

#[derive(Protocol)]
struct BlockChange0 {
    x: i32,
    y: u8,
    z: i32,
    #[varint]
    // todo! extract the next two variables into concrete types for the version
    block_type: i32,
    block_data: u8,
}

#[derive(Protocol)]
struct BlockAction0 {
    x: i32,
    y: i16,
    z: i32,
    action_id: u8,
    action_param: u8,
    #[varint]
    block_type: i32,
}

#[derive(Protocol)]
struct BlockBreakAnimation {
    #[varint]
    entity_id: i32,
    x: i32,
    y: i32,
    z: i32,
    /// 0-9
    destroy_stage: u8,
}

// #[derive(Protocol)]
struct MapChunkBulk0<'a> {
    /// Whether or not the chunk data contains a light nibble array. This is
    /// true in the main world, false in the end + nether
    skylight_sent: bool,
    data: Cow<'a, [u8]>,
    column_metas: Vec<ChunkMeta0>,
}
impl<'a> ProtocolRead<'a> for MapChunkBulk0<'a> {
    fn read(cursor: &'_ mut std::io::Cursor<&'a [u8]>) -> Result<Self, ReadError> {
        let column_count = u16::read(cursor)?;
        let data_len = u32::read(cursor)?;
        let skylight_sent = bool::read(cursor)?;
        let data = cursor
            .get_ref()
            .get(0..data_len as usize + cursor.position() as usize)
            // todo! different error
            .ok_or(ReadError::InvalidEnumId)?;
        let column_metas = (0..column_count)
            .map(|_| ChunkMeta0::read(cursor))
            .collect::<Result<_, _>>()?;
        Ok(Self {
            skylight_sent,
            data: Cow::Borrowed(data),
            column_metas,
        })
    }
}
impl ProtocolWrite for MapChunkBulk0<'_> {
    fn write(self, writer: &mut impl std::io::Write) -> Result<(), WriteError> {
        (self.column_metas.len() as u16).write(writer)?;
        (self.data.len() as u32).write(writer)?;
        self.skylight_sent.write(writer)?;
        writer.write_all(&self.data)?;
        for meta in self.column_metas {
            meta.write(writer)?;
        }
        Ok(())
    }

    fn size_hint() -> usize {
        2 + 4 + 1
    }
}

#[derive(Protocol)]
struct ChunkMeta0 {
    chunk_x: i32,
    chunk_z: i32,
    primary_bitmap: u16,
    add_bitmap: u16,
}

#[derive(Protocol)]
struct Explosion0 {
    x: f32,
    y: f32,
    z: f32,
    radius: f32,
    // todo! #[count(i32)]
    records: Vec<ExplosionRecord>,
    motion_x: f32,
    motion_y: f32,
    motion_z: f32,
}

#[derive(Protocol)]
struct ExplosionRecord {
    dx: i8,
    dy: i8,
    dz: i8,
}

#[derive(Protocol)]
// todo! more detailed data using #[separated]
struct Effect0 {
    effect_id: i32,
    // todo! relative? fixed point?
    /// The X location of the effect multiplied by 8
    x: i32,
    /// The Y location of the effect multiplied by 8
    y: i8,
    /// The Z location of the effect multiplied by 8
    z: i32,
    effect_data: i32,
    disable_rel_volume: bool,
}

#[derive(Protocol)]
struct SoundEffect0<'a> {
    effect_id: Cow<'a, str>,
    // todo! relative? fixed point?
    /// The X location of the effect multiplied by 8
    x: i32,
    /// The Y location of the effect multiplied by 8
    y: i32,
    /// The Z location of the effect multiplied by 8
    z: i32,
    /// 1 is 100%, can be more
    volume: f32,
    /// 63 is 100%, can be more
    pitch: u8,
    category: SoundCategory0,
}

#[derive(Protocol)]
#[from(u8)]
enum SoundCategory0 {
    Master = 0,
    Music,
    Records,
    Weather,
    Blocks,
    Mobs,
    Animals,
    Players,
}

#[derive(Protocol)]
struct SoundEffect1<'a> {
    effect_id: Cow<'a, str>,
    // todo! relative? fixed point?
    /// The X location of the effect multiplied by 8
    x: i32,
    /// The Y location of the effect multiplied by 8
    y: i32,
    /// The Z location of the effect multiplied by 8
    z: i32,
    /// 1 is 100%, can be more
    volume: f32,
    /// 63 is 100%, can be more
    pitch: u8,
}

#[derive(Protocol)]
struct Particle0<'a> {
    // todo! specific strings into enum
    name: Cow<'a, str>,
    x: f32,
    y: f32,
    z: f32,
    /// This is added to the X position after being multiplied by random.nextGaussian()
    offset_x: f32,
    /// This is added to the Y position after being multiplied by random.nextGaussian()
    offset_y: f32,
    /// This is added to the Z position after being multiplied by random.nextGaussian()
    offset_z: f32,
    speed: f32,
    number: i32,
}

// #[derive(Protocol)]
// struct ChangeGameState0 {
//     reason: GameStateChangeReason,
// }

// #[derive(Protocol)]
// #[from(u8)]
enum GameStateChange0 {
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

#[derive(Protocol)]
#[from(u8)]
enum DemoMessage0 {
    WelcomeToDemo = 0,
    MovementControl = 101,
    JumpControl,
    InventoryControl,
}

impl ProtocolRead<'_> for GameStateChange0 {
    fn read(cursor: &'_ mut std::io::Cursor<&[u8]>) -> Result<Self, ReadError> {
        let reason = u8::read(cursor)?;
        let value = f32::read(cursor)?;
        use self::DemoMessage0::*;
        use GameStateChange0::*;
        Ok(match reason {
            0 => InvalidBed,
            1 => BeginRaining,
            2 => EndRaining,
            3 => ChangeGameMode(match value as u8 {
                0 => GameMode0::Survival,
                1 => GameMode0::Creative,
                2 => GameMode0::Adventure,
                _ => return Err(ReadError::InvalidEnumId),
            }),
            4 => EnterCredits,
            5 => DemoMessage(match value as u8 {
                0 => WelcomeToDemo,
                101 => MovementControl,
                102 => JumpControl,
                103 => InventoryControl,
                _ => return Err(ReadError::InvalidEnumId),
            }),
            6 => BowHitSound,
            7 => FadeValue(value),
            8 => FadeTime(value),
            _ => return Err(ReadError::InvalidEnumId),
        })
    }
}

impl ProtocolWrite for GameStateChange0 {
    fn write(self, writer: &mut impl std::io::Write) -> Result<(), WriteError> {
        let (reason, value) = match self {
            GameStateChange0::InvalidBed => (0u8, 0.0),
            GameStateChange0::BeginRaining => (1, 0.0),
            GameStateChange0::EndRaining => (2, 0.0),
            GameStateChange0::ChangeGameMode(gamemode) => (3, gamemode as u8 as f32),
            GameStateChange0::EnterCredits => (4, 0.0),
            GameStateChange0::DemoMessage(demomessage) => (5, demomessage as u8 as f32),
            GameStateChange0::BowHitSound => (6, 0.0),
            GameStateChange0::FadeValue(value) => (7, value),
            GameStateChange0::FadeTime(value) => (8, value),
        };
        reason.write(writer)?;
        value.write(writer)?;
        Ok(())
    }

    fn size_hint() -> usize {
        5
    }
}

#[derive(Protocol)]
struct SpawnGlobalEntity0 {
    #[varint]
    entity_id: i32,
    /// The global entity type, currently always 1 for thunderbolt.
    kind: u8,
    // todo! #[fixed(5)]
    x: i32,
    y: i32,
    z: i32,
}

struct OpenWindow0<'a> {
    window_id: u8,
    kind: InventoryKind0,
    title: Cow<'a, str>,
    slot_count: u8,
    use_title: bool,
}

impl<'read, 'a> ProtocolRead<'read> for OpenWindow0<'a>
where
    'read: 'a,
{
    fn read(cursor: &mut std::io::Cursor<&'read [u8]>) -> Result<Self, ReadError> {
        let window_id = u8::read(cursor)?;
        let kind = u8::read(cursor)?;
        let title = Cow::read(cursor)?;
        let slot_count = u8::read(cursor)?;
        let use_title = bool::read(cursor)?;
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
                entity_id: i32::read(cursor)?,
            },
            _ => return Err(ReadError::InvalidEnumId),
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
impl<'a> ProtocolWrite for OpenWindow0<'a> {
    fn write(self, buf: &mut impl ::std::io::Write) -> Result<(), WriteError> {
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
        ProtocolWrite::write(window_id, buf)?;
        ProtocolWrite::write(kind, buf)?;
        ProtocolWrite::write(title, buf)?;
        ProtocolWrite::write(slot_count, buf)?;
        ProtocolWrite::write(use_title, buf)?;
        if let Some(entity_id) = entity_id {
            ProtocolWrite::write(entity_id, buf)?;
        }
        Ok(())
    }
    #[inline(always)]
    fn size_hint() -> usize {
        <u8 as ProtocolWrite>::size_hint()
            + 1
            + <Cow<'a, str> as ProtocolWrite>::size_hint()
            + <u8 as ProtocolWrite>::size_hint()
            + <bool as ProtocolWrite>::size_hint()
            + <i32 as ProtocolWrite>::size_hint()
    }
}

// #[derive(Protocol)]
// todo! very good place for #[separate]
enum InventoryKind0 {
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

#[derive(Protocol)]
struct CloseWindow0 {
    /// This is the id of the window that was closed. 0 for inventory.
    window_id: u8,
}

#[derive(Protocol)]
struct SetSlot0 {
    /// The window which is being updated. 0 for player inventory. Note that
    /// all known window types include the player inventory. This packet will
    /// only be sent for the currently opened window while the player is
    /// performing actions, even if it affects the player inventory. After the
    /// window is closed, a number of these packets are sent to update the
    /// player's inventory window (0).
    window_id: u8,
    /// The slot that should be updated
    slot: u16,
    // todo! slot data
    // data: Slot
}

#[derive(Protocol)]
struct WindowItems {
    /// The id of window which items are being sent for. 0 for player inventory.
    window_id: u8,
    // todo! #[count(u16)]
    // todo! slot data
    // slots: Vec<Slot>
}

#[derive(Protocol)]
/// see https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5007#Window_Property
struct WindowProperty0 {
    window_id: u8,
    property: u16,
    value: u16,
}

#[derive(Protocol)]
struct ConfirmTransaction0 {
    window_id: u8,
    action_number: i16,
    accepted: bool,
}

#[derive(Protocol)]
struct UpdateSign0<'a> {
    x: i32,
    y: i16,
    z: i32,
    line1: Cow<'a, str>,
    line2: Cow<'a, str>,
    line3: Cow<'a, str>,
    line4: Cow<'a, str>,
}

#[derive(Protocol)]
struct Maps0 {
    #[varint]
    item_damage: i32,
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

#[derive(Protocol)]
struct UpdateBlockEntity0 {
    x: i32,
    y: i16,
    z: i32,
    /// The type of update to perform
    action: u8,
    /// varies
    data_length: u16,
    // Present if data length > 0. Compressed with gzip. Varies
    // todo! nbt
    // data: Nbt
}

#[derive(Protocol)]
struct SignEditorOpen0 {
    x: i32,
    y: i32,
    z: i32,
}

#[derive(Protocol)]
struct Statistics0<'a> {
    entries: Vec<Statistic0<'a>>,
}

#[derive(Protocol)]
struct Statistic0<'a> {
    name: Cow<'a, str>,
    #[varint]
    /// The amount to increase by
    amount: i32,
}

#[derive(Protocol)]
struct PlayerListItem0<'a> {
    /// Supports chat colouring, limited to 16 characters.
    name: Cow<'a, str>,
    /// The client will remove the user from the list if false.
    online: bool,
    /// Ping, presumably in ms.
    ping: i16,
}

pub use super::PlayerAbilities0;

#[derive(Protocol)]
struct TabComplete0<'a> {
    /// One eligible command
    matches: Vec<Cow<'a, str>>,
}

#[derive(Protocol)]
struct ScoreboardObjective0<'a> {
    name: Cow<'a, str>,
    value: Cow<'a, str>,
    action: ScoreboardAction0,
}
#[derive(Protocol)]
#[from(u8)]
enum ScoreboardAction0 {
    #[case(0)]
    Create,
    Remove,
    Update,
}

#[derive(Protocol)]
struct UpdateScore0<'a> {
    /// The name of the score to be updated or removed
    name: Cow<'a, str>,
    action: UpdateScoreAction0<'a>,
}

#[derive(Protocol)]
#[from(u8)]
enum UpdateScoreAction0<'a> {
    #[case(0)]
    Update {
        /// The name of the objective the score belongs to
        text: Cow<'a, str>,
        /// The score to be displayed next to the entry
        kind: i32,
    },
    Remove,
}

#[derive(Protocol)]
struct DisplayScoreboard0<'a> {
    position: ScoreboardPosition,
    name: Cow<'a, str>,
}

#[derive(Protocol)]
#[from(u8)]
enum ScoreboardPosition {
    List = 0,
    Sidebar,
    BelowName,
}

#[derive(Protocol)]
struct Teams0<'a> {
    name: Cow<'a, str>,
    action: TeamAction0<'a>,
}

#[derive(Protocol)]
enum TeamAction0<'a> {
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

#[derive(Protocol)]
#[from(u8)]
enum TeamFriendlyFire {
    Off = 0,
    On,
    FriendliesVisible = 3,
}

// for later protocol versions
#[derive(Protocol)]
#[from(&str)]
enum NameTagVisibility /*version?*/ {
    #[case("always")]
    Always,
    #[case("hideForOtherTeams")]
    HideForOtherTeams,
    #[case("hideForOwnTeam")]
    HideForOwnTeam,
    #[case("never")]
    Never,
}

pub use super::PluginMessage0;

#[derive(Protocol)]
struct Disconnect0<'a> {
    reason: Cow<'a, str>,
}
