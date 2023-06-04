use crate::netty::types::angle::Angle;
use crate::netty::types::entity_metadata::PackedEntityMetadata0;
use crate::netty::types::position::Position6;
use crate::netty::types::slot::Slot0;

use ::miners_encoding::{
    attrs::{Fixed, StringUuid, Var},
    decode, encode, Decode, Encode,
};

use std::borrow::Cow;
use uuid::Uuid;

#[derive(Encoding, ToStatic, Debug)]
/// The server will frequently send out a keep-alive, each containing a random
/// ID. The client must respond with the same payload
/// (see [`serverbound::KeepAlive0`][ka0]/[`serverbound::KeepAlive7`][ka7]).
/// If the client does not respond to them for over 30 seconds, the server
/// kicks the client. Vice versa, if the server does not send any keep-alives
/// for 20 seconds, the client will disconnect and yields a "Timed out"
/// exception.
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5007#Keep_Alive)
/// [burger](https://rob9315.github.io/mcpackets/13w41b.html#packets:play_clientbound_00)
///
/// [ka0]: super::serverbound::KeepAlive0
/// [ka7]: super::serverbound::KeepAlive7
pub struct KeepAlive0 {
    pub id: i32,
}

#[derive(Encoding, ToStatic, Debug)]
/// The server will frequently send out a keep-alive, each containing a random
/// ID. The client must respond with the same payload
/// (see [`serverbound::KeepAlive7`][ka7]). If the
/// client does not respond to them for over 30 seconds, the server kicks the
/// client. Vice versa, if the server does not send any keep-alives for 20
/// seconds, the client will disconnect and yields a "Timed out" exception.
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5972#Keep_Alive)
/// [burger diff](https://rob9315.github.io/mcpackets/diff_31_32.html#packets:play_clientbound_00)
///
/// [ka7]: super::serverbound::KeepAlive7
pub struct KeepAlive32 {
    #[encoding(varint)]
    pub id: i32,
}

#[derive(ToStatic, Debug)]
/// Sent after the Login Sequence
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5007#Join_Game)
/// [burger](https://rob9315.github.io/mcpackets/13w41b.html#packets:play_clientbound_01)
pub struct JoinGame0 {
    /// Entity ID of the Player
    pub entity_id: i32,
    pub hardcore: bool,
    pub gamemode: GameMode0,
    pub dimension: Dimension0,
    pub difficulty: Difficulty0,
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

#[derive(ToStatic, Debug)]
/// Sent after the Login Sequence
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5048#Join_Game)
/// [burger diff](https://rob9315.github.io/mcpackets/diff_0_1.html#packets:play_clientbound_01)
pub struct JoinGame1<'a> {
    /// Entity ID of the Player
    pub entity_id: i32,
    pub hardcore: bool,
    pub gamemode: GameMode0,
    pub dimension: Dimension0,
    pub difficulty: Difficulty0,
    pub max_players: u8,
    /// indicates the kind of world gen used for the level, values should be
    /// one of `"default"`, `"flat"`, `"largeBiomes"`, `"amplified"` or
    /// `"default_1_1"`
    pub level_type: Cow<'a, str>,
}

impl<'dec: 'a, 'a> Decode<'dec> for JoinGame1<'a> {
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

#[derive(ToStatic, Debug)]
/// Sent after the Login Sequence
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5947#Join_Game)
/// [burger diff](https://rob9315.github.io/mcpackets/diff_28_29.html#packets:play_clientbound_01)
pub struct JoinGame29<'a> {
    /// Entity ID of the Player
    pub entity_id: i32,
    pub hardcore: bool,
    pub gamemode: GameMode0,
    pub dimension: Dimension0,
    pub difficulty: Difficulty0,
    pub max_players: u8,
    /// indicates the kind of world gen used for the level, values should be
    /// one of `"default"`, `"flat"`, `"largeBiomes"`, `"amplified"` or
    /// `"default_1_1"`
    pub level_type: Cow<'a, str>,
    pub reduced_debug_info: bool,
}

impl<'dec: 'a, 'a> Decode<'dec> for JoinGame29<'a> {
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
            reduced_debug_info: bool::decode(cursor)?,
        })
    }
}

impl Encode for JoinGame29<'_> {
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
        self.reduced_debug_info.encode(writer)?;
        Ok(())
    }
}

#[derive(Encoding, ToStatic, Clone, Copy, Debug)]
#[encoding(from = "u8")]
pub enum GameMode0 {
    Survival = 0,
    Creative,
    Adventure,
}

pub use super::Difficulty0;

#[derive(Encoding, ToStatic, Debug)]
#[encoding(from = "i8")]
pub enum Dimension0 {
    Nether = -1,
    Overworld = 0,
    End,
}

#[derive(Encoding, ToStatic, Debug)]
/// Chat Message
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5007#Chat_Message)
/// [burger](https://rob9315.github.io/mcpackets/13w41b.html#packets:play_clientbound_02)
pub struct ChatMessage0<'a> {
    /// Chat with control codes ($)
    pub message: Cow<'a, str>,
}

#[derive(Encoding, ToStatic, Debug)]
/// Chat/System Message
///
///  Identifying the difference between Chat/System Message is important as it
/// helps respect the user's chat visibility options. While
/// [`ChatMessagePosition6::HotBar`] (position 2) accepts json formatting, it
/// will not display. old style formatting works.
///
/// **warning: wrong information on** [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5368#Chat_Message)
/// the correct protocol version is 6 (14w03a/14w03b), not 5 (14w02a-1.7.10)
/// [burger diff](https://rob9315.github.io/mcpackets/diff_5_6.html#packets:play_clientbound_02)
///
/// [`ChatMessagePosition6::HotBar`]: self::ChatMessagePosition6#variant.HotBar
pub struct ChatMessage6<'a> {
    // TODO: add ChatMessage json thing
    pub message: Cow<'a, str>,
    pub position: ChatMessagePosition6,
}

#[derive(Encoding, ToStatic, Debug)]
#[encoding(from = "u8")]
pub enum ChatMessagePosition6 {
    Chat = 0,
    System,
    Hotbar,
}

#[derive(Encoding, ToStatic, Debug)]
/// Time Update
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5007#Time_Update)
/// [burger](https://rob9315.github.io/mcpackets/13w41b.html#packets:play_clientbound_03)
pub struct TimeUpdate0 {
    /// Age of the world
    ///
    /// In ticks; not changed by server commands
    pub ticks: i64,
    /// Time of day
    ///
    /// The world (or region) time, in ticks. If negative the sun will stop
    /// moving at the Math.abs of the time
    pub time_of_day: i64,
}

#[derive(Encoding, ToStatic, Debug)]
/// Entity Equipment
///
/// Changes the visible Equipment of an Entity, for example the held item or
/// worn armor.
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5007#Entity_Equipment)
/// [burger](https://rob9315.github.io/mcpackets/13w41b.html#packets:play_clientbound_04)
pub struct EntityEquipment0<'a> {
    pub entity_id: i32,
    pub slot: EquipmentSlot0,
    pub item: Slot0<'a>,
}

#[derive(Encoding, ToStatic, Debug)]
/// Entity Equipment
///
/// Changes the visible Equipment of an Entity, for example the held item or
/// worn armor.
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5392#Entity_Equipment)
/// [burger diff](https://rob9315.github.io/mcpackets/diff_6_7.html#packets:play_clientbound_06)
pub struct EntityEquipment7<'a> {
    #[encoding(varint)]
    pub entity_id: i32,
    pub slot: EquipmentSlot0,
    pub item: Slot0<'a>,
}

#[derive(Encoding, ToStatic, Debug)]
#[encoding(from = "u16")]
pub enum EquipmentSlot0 {
    Hand = 0,
    Boots,
    Leggings,
    Chestplate,
    Helmet,
}

#[derive(Encoding, ToStatic, Debug)]
/// Entity Equipment
///
/// Changes the visible Equipment of an Entity, for example the held item or
/// worn armor.
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=6739#Entity_Equipment)
/// [burger diff](https://rob9315.github.io/mcpackets/diff_48_49.html#packets:play_clientbound_04)
pub struct EntityEquipment49<'a> {
    #[encoding(varint)]
    pub entity_id: i32,
    pub slot: EquipmentSlot49,
    pub item: Slot0<'a>,
}

#[derive(Encoding, ToStatic, Debug)]
#[encoding(from = "u8")]
pub enum EquipmentSlot49 {
    Hand = 0,
    Offhand,
    Boots,
    Leggings,
    Chestplate,
    Helmet,
}

#[derive(Encoding, ToStatic, Debug)]
/// Spawn Position
///
/// Sent by the server after login to specify the coordinates of the spawn
/// point (the point at which players spawn at, and which the compass points
/// to). It can be sent at any time to update the point compasses point at.
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5007#Spawn_Position)
/// [burger](https://rob9315.github.io/mcpackets/13w41b.html#packets:play_clientbound_05)
pub struct SpawnPosition0 {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Bitfield, ToStatic, Clone, Copy)]
/// Spawn Position
///
/// Sent by the server after login to specify the coordinates of the spawn
/// point (the point at which players spawn at, and which the compass points
/// to). It can be sent at any time to update the point compasses point at.
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5368#Spawn_Position)
/// [burger diff](https://rob9315.github.io/mcpackets/diff_5_6.html#packets:play_clientbound_05)
pub struct SpawnPosition6 {
    #[encoding(bits = 26)]
    pub x: i32,
    #[encoding(bits = 26)]
    pub z: i32,
    #[encoding(bits = 12)]
    pub y: i16,
}

#[derive(Encoding, ToStatic, Debug)]
/// Update Health
///
/// Sent by the server to update/set the health of the player it is sent to.
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5007#Update_Health)
/// [burger](https://rob9315.github.io/mcpackets/13w41b.html#packets:play_clientbound_06)
pub struct UpdateHealth0 {
    /// Amount of Half Hearts
    ///
    /// 0.0 means dead, 20.0 = full HP
    ///
    /// value range: 0.0..=20.0
    pub health: f32,
    /// Amount of Half Food Bars
    ///
    /// value range: 0..=20?
    pub food: i16,
    /// Food Saturation
    ///
    /// acts as a food "overcharge". Food values will not decrease while the
    /// saturation is over zero. Players logging in automatically get a
    /// saturation of 5.0. Eating food increases the saturation as well as the
    /// food bar.
    ///
    /// value range: 0.0..=5.0 (in integer increments?)
    pub saturation: f32,
}

#[derive(Encoding, ToStatic, Debug)]
/// Update Health
///
/// Sent by the server to update/set the health of the player it is sent to.
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5392#Update_Health)
/// [burger diff](https://rob9315.github.io/mcpackets/diff_6_7.html#packets:play_clientbound_06)
pub struct UpdateHealth7 {
    /// Amount of Half Hearts
    ///
    /// 0.0 means dead, 20.0 = full HP
    ///
    /// value range: 0.0..=20.0
    pub health: f32,
    /// Amount of Half Food Bars
    ///
    /// value range: 0..=20?
    #[encoding(varint)]
    pub food: i32,
    /// Food Saturation
    ///
    /// acts as a food "overcharge". Food values will not decrease while the
    /// saturation is over zero. Players logging in automatically get a
    /// saturation of 5.0. Eating food increases the saturation as well as the
    /// food bar.
    ///
    /// value range: 0.0..=5.0 (in integer increments?)
    pub saturation: f32,
}

#[derive(Encoding, ToStatic, Debug)]
/// Respawn
///
/// To change the player's dimension (overworld/nether/end), send them a
/// respawn packet with the appropriate dimension, followed by prechunks/chunks
/// for the new dimension, and finally a position and look packet. You do not
/// need to unload chunks, the client will do it automatically.
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5007#Respawn)
/// [burger](https://rob9315.github.io/mcpackets/13w41b.html#packets:play_clientbound_07)
pub struct Respawn0 {
    pub dimension: Dimension0,
    pub difficulty: Difficulty0,
    pub gamemode: GameMode0,
}

#[derive(Encoding, ToStatic, Debug)]
/// Respawn
///
/// To change the player's dimension (overworld/nether/end), send them a
/// respawn packet with the appropriate dimension, followed by prechunks/chunks
/// for the new dimension, and finally a position and look packet. You do not
/// need to unload chunks, the client will do it automatically.
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5048#Respawn)
/// [burger diff](https://rob9315.github.io/mcpackets/diff_0_1.html#packets:play_clientbound_07)
pub struct Respawn1<'a> {
    pub dimension: Dimension0,
    pub difficulty: Difficulty0,
    pub gamemode: GameMode0,
    /// indicates the kind of world gen used for the level, values should be
    /// one of `"default"`, `"flat"`, `"largeBiomes"`, `"amplified"` or
    /// `"default_1_1"`
    pub level_type: Cow<'a, str>,
}

#[derive(Encoding, ToStatic, Debug)]
/// Synchronize Player Position
///
/// Updates the player's position on the server. This packet will also close
/// the “Downloading Terrain” screen when joining/respawning.
///
/// If the distance between the last known position of the player on the server
/// and the new position set by this packet is greater than 100 meters, the
/// client will be kicked for “You moved too quickly :( (Hacking?)”.
///
/// Also if the fixed-point number of X or Z is set greater than 3.2E7D the
/// client will be kicked for “Illegal position”.
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5007#Player_Position_And_Look)
/// [burger](https://rob9315.github.io/mcpackets/13w41b.html#packets:play_clientbound_08)
pub struct PositionAndLook0 {
    /// Absolute X Position
    pub x: f64,
    /// Absolute Y Position
    pub y: f64,
    /// Absolute Z Position
    pub z: f64,
    /// Absolute rotation on the X Axis, in degrees
    ///
    /// Does not follow classical trigonometry rules. The unit circle of yaw on
    /// the XZ-plane starts at `(0, 1)` and turns counterclockwise, with 90 at
    /// `(-1, 0)`, 180 at `(0, -1)` and 270 at `(1, 0)`. Additionally, yaw is
    /// not clamped to between 0 and 360 degrees; any number is valid,
    /// including negative numbers and numbers greater than 360.
    pub yaw: f32,
    /// Absolute rotation on the Y Axis, in degrees
    ///
    /// 0 is looking straight ahead, -90 is looking straight up, and 90 is
    /// looking straight down.
    pub pitch: f32,
    pub on_ground: bool,
}

#[derive(Encoding, ToStatic, Debug)]
/// Synchronize Player Position
///
/// Updates the player's position on the server. This packet will also close
/// the “Downloading Terrain” screen when joining/respawning.
///
/// If the distance between the last known position of the player on the server
/// and the new position set by this packet is greater than 100 meters, the
/// client will be kicked for “You moved too quickly :( (Hacking?)”.
///
/// Also if the fixed-point number of X or Z is set greater than 3.2E7D the
/// client will be kicked for “Illegal position”.
///
/// **warning: wrong information on** [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5368#Player_Position_And_Look)
/// the correct protocol version is 6 (14w03a/14w03b), not 5 (14w02a-1.7.10)
/// [burger diff](https://rob9315.github.io/mcpackets/diff_5_6.html#packets:play_clientbound_08)
pub struct PositionAndLook6 {
    /// X Position
    ///
    /// May or may not be relative, see [`relativity`](#structfield.relativity)
    pub x: f64,
    /// Y Position
    ///
    /// May or may not be relative, see [`relativity`](#structfield.relativity)
    pub y: f64,
    /// Z Position
    ///
    /// May or may not be relative, see [`relativity`](#structfield.relativity)
    pub z: f64,
    /// Rotation on the X Axis, in degrees
    ///
    /// May or may not be relative, see [`relativity`](#structfield.relativity)
    ///
    /// Does not follow classical trigonometry rules. The unit circle of yaw on
    /// the XZ-plane starts at `(0, 1)` and turns counterclockwise, with 90 at
    /// `(-1, 0)`, 180 at `(0, -1)` and 270 at `(1, 0)`. Additionally, yaw is
    /// not clamped to between 0 and 360 degrees; any number is valid,
    /// including negative numbers and numbers greater than 360.
    pub yaw: f32,
    /// Rotation on the Y Axis, in degrees
    ///
    /// May or may not be relative, see [`relativity`](#structfield.relativity)
    ///
    /// 0 is looking straight ahead, -90 is looking straight up, and 90 is
    /// looking straight down.
    pub pitch: f32,
    /// Decides if the other fields are relative or not.
    pub relativity: PositionAndLookBitfield6,
}

#[derive(Bitfield, ToStatic, Debug)]
#[encoding(typ = "u8", reverse)]
pub struct PositionAndLookBitfield6 {
    #[encoding(bool)]
    pub x: bool,
    #[encoding(bool)]
    pub y: bool,
    #[encoding(bool)]
    pub z: bool,
    #[encoding(bool)]
    pub pitch: bool,
    #[encoding(bool)]
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

#[derive(Encoding, ToStatic, Debug)]
/// Held Item Change
///
/// Sent to change the player's slot selection.
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5007#Held_Item_Change)
/// [burger](https://rob9315.github.io/mcpackets/13w41b.html#packets:play_clientbound_09)
pub struct HeldItemChange0 {
    /// The slot which the player has selected (0-8)
    pub slot: u8,
}

#[derive(Encoding, ToStatic, Debug)]
/// Use Bed
///
/// This packet tells that a player goes to bed.
///
/// The client with the matching Entity ID will go into bed mode.
///
/// This Packet is sent to all nearby players including the one sent to bed.
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5007#Use_Bed)
/// [burger](https://rob9315.github.io/mcpackets/13w41b.html#packets:play_clientbound_0a)
pub struct UseBed0 {
    pub entity_id: i32,
    /// Bed Head Part X Position
    pub x: i32,
    /// Bed Head Part Y Position
    pub y: i8,
    /// Bed Head Part Z Position
    pub z: i32,
}

#[derive(Encoding, ToStatic, Debug)]
/// Use Bed
///
/// This packet tells that a player goes to bed.
///
/// The client with the matching Entity ID will go into bed mode.
///
/// This Packet is sent to all nearby players including the one sent to bed.
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5368#Use_Bed)
/// [burger diff](https://rob9315.github.io/mcpackets/diff_5_6.html#packets:play_clientbound_0a)
pub struct UseBed6 {
    pub entity_id: i32,
    /// Position of the head part of the targeted bed
    pub location: Position6,
}

#[derive(Encoding, ToStatic, Debug)]
/// Use Bed
///
/// This packet tells that a player goes to bed.
///
/// The client with the matching Entity ID will go into bed mode.
///
/// This Packet is sent to all nearby players including the one sent to bed.
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5392#Use_Bed)
/// [burger diff](https://rob9315.github.io/mcpackets/diff_6_7.html#packets:play_clientbound_0a)
pub struct UseBed7 {
    #[encoding(varint)]
    pub entity_id: i32,
    /// Position of the head part of the targeted bed
    pub location: Position6,
}

#[derive(Encoding, ToStatic, Debug)]
/// Animation
///
/// Sent whenever an entity should change animation.
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5007#Animation)
/// [burger](https://rob9315.github.io/mcpackets/13w41b.html#packets:play_clientbound_0b)
pub struct Animation0 {
    #[encoding(varint)]
    pub entity_id: i32,
    pub animation: super::AnimationId0,
}

#[derive(Encoding, ToStatic, Debug)]
/// Spawn Player
///
/// This packet is sent by the server when a player comes into visible range,
/// **not** when a player joins.
///
/// This packet must be sent after the [`PlayerListItem0`] packet that, adds
/// the player data for the client to use when spawning a player. If the tab
/// list entry for the UUID included in this packet is not present when this
/// packet arrives, the entity will not be spawned. The tab includes skin/cape
/// data.
///
/// Servers can, however, safely spawn player entities for players not in
/// visible range. The client appears to handle it correctly.
///
/// When in online-mode the UUIDs must be valid and have valid skin blobs, in
/// offline-mode UUID v3 is used. For NPCs UUID v2 should be used.
///
/// [pv0 wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5007#Spawn_Player)
/// [pv0 burger](https://rob9315.github.io/mcpackets/13w41b.html#packets:play_clientbound_0c)
/// no pv6 wiki.vg
/// [pv6 burger diff](https://rob9315.github.io/mcpackets/diff_5_6.html#packets:play_clientbound_0c)
pub struct SpawnPlayer0<'a> {
    #[encoding(varint)]
    pub entity_id: i32,
    pub player_uuid: StringUuid,
    pub name: Cow<'a, str>,
    #[encoding(fixed(5, "i32"))]
    pub x: f64,
    #[encoding(fixed(5, "i32"))]
    pub y: f64,
    #[encoding(fixed(5, "i32"))]
    pub z: f64,
    pub yaw: Angle,
    pub pitch: Angle,
    /// The item the player is currently holding. Note that this should be 0
    /// for "no item", unlike -1 used in other packets. A negative value
    /// crashes clients.
    pub current_item: u16,
    /// The client will crash if no metadata is sent
    pub metadata: PackedEntityMetadata0<'a>,
}

#[derive(Encoding, ToStatic, Debug)]
/// Spawn Player
///
/// This packet is sent by the server when a player comes into visible range,
/// **not** when a player joins.
///
/// This packet must be sent after the [`PlayerListItem0`] packet that, adds
/// the player data for the client to use when spawning a player. If the tab
/// list entry for the UUID included in this packet is not present when this
/// packet arrives, the entity will not be spawned. The tab includes skin/cape
/// data.
///
/// Servers can, however, safely spawn player entities for players not in
/// visible range. The client appears to handle it correctly.
///
/// When in online-mode the UUIDs must be valid and have valid skin blobs, in
/// offline-mode UUID v3 is used. For NPCs UUID v2 should be used.
///
/// no pv5 wiki.vg
/// [pv5 burger diff](https://rob9315.github.io/mcpackets/diff_4_5.html#packets:play_clientbound_0c)
/// [pv7 wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5392#Spawn_Player)
/// [pv7 burger diff](https://rob9315.github.io/mcpackets/diff_6_7.html#packets:play_clientbound_0c)
pub struct SpawnPlayer5<'a> {
    #[encoding(varint)]
    pub entity_id: i32,
    pub player_uuid: StringUuid,
    pub name: Cow<'a, str>,
    pub properties: Vec<PlayerProperty<'a>>,
    #[encoding(fixed(5, "i32"))]
    pub x: f64,
    #[encoding(fixed(5, "i32"))]
    pub y: f64,
    #[encoding(fixed(5, "i32"))]
    pub z: f64,
    pub yaw: Angle,
    pub pitch: Angle,
    /// The item the player is currently holding. Note that this should be 0
    /// for "no item", unlike -1 used in other packets. A negative value
    /// crashes clients.
    pub current_item: u16,
    /// The client will crash if no metadata is sent
    pub metadata: PackedEntityMetadata0<'a>,
}

#[derive(Encoding, ToStatic, Debug)]
/// Spawn Player
///
/// This packet is sent by the server when a player comes into visible range,
/// **not** when a player joins.
///
/// This packet must be sent after the [`PlayerListItem0`] packet that, adds
/// the player data for the client to use when spawning a player. If the tab
/// list entry for the UUID included in this packet is not present when this
/// packet arrives, the entity will not be spawned. The tab includes skin/cape
/// data.
///
/// Servers can, however, safely spawn player entities for players not in
/// visible range. The client appears to handle it correctly.
///
/// When in online-mode the UUIDs must be valid and have valid skin blobs, in
/// offline-mode UUID v3 is used. For NPCs UUID v2 should be used.
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5643#Spawn_Player)
/// [burger diff](https://rob9315.github.io/mcpackets/diff_18_19.html#packets:play_clientbound_0c)
pub struct SpawnPlayer19<'a> {
    #[encoding(varint)]
    pub entity_id: i32,
    pub player_uuid: Uuid,
    #[encoding(fixed(5, "i32"))]
    pub x: f64,
    #[encoding(fixed(5, "i32"))]
    pub y: f64,
    #[encoding(fixed(5, "i32"))]
    pub z: f64,
    pub yaw: Angle,
    pub pitch: Angle,
    /// The item the player is currently holding. Note that this should be 0
    /// for "no item", unlike -1 used in other packets. A negative value
    /// crashes clients.
    pub current_item: u16,
    /// The client will crash if no metadata is sent
    pub metadata: PackedEntityMetadata0<'a>,
}

#[derive(Encoding, ToStatic, Debug)]
/// Spawn Player
///
/// This packet is sent by the server when a player comes into visible range,
/// **not** when a player joins.
///
/// This packet must be sent after the [`PlayerListItem0`] packet that, adds
/// the player data for the client to use when spawning a player. If the tab
/// list entry for the UUID included in this packet is not present when this
/// packet arrives, the entity will not be spawned. The tab includes skin/cape
/// data.
///
/// Servers can, however, safely spawn player entities for players not in
/// visible range. The client appears to handle it correctly.
///
/// When in online-mode the UUIDs must be valid and have valid skin blobs, in
/// offline-mode UUID v3 is used. For NPCs UUID v2 should be used.
///
// TODO: check if there really is no wiki.vg for this
/// no wiki.vg
/// [burger diff](https://rob9315.github.io/mcpackets/diff_48_49.html#packets:play_clientbound_0c)
pub struct SpawnPlayer49<EntityMetadata> {
    #[encoding(varint)]
    pub entity_id: i32,
    pub player_uuid: Uuid,
    #[encoding(fixed(5, "i32"))]
    pub x: f64,
    #[encoding(fixed(5, "i32"))]
    pub y: f64,
    #[encoding(fixed(5, "i32"))]
    pub z: f64,
    pub yaw: Angle,
    pub pitch: Angle,
    /// The client will crash if no metadata is sent
    pub metadata: EntityMetadata,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct PlayerProperty<'a> {
    pub name: Cow<'a, str>,
    pub value: Cow<'a, str>,
    pub signature: Cow<'a, str>,
}

#[derive(Encoding, ToStatic, Debug)]
/// Collect Item
///
/// Sent when an entity collects an item.
///
/// The sole purpose seems to be to play the animation of the item flying
/// towards the collector.
///
/// The vanilla server only checks for items to be picked up after each
/// [`PlayerPosition0`] and [`PlayerPositionAndLook0`] packet sent by the
/// client.
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5007#Collect_Item)
/// [burger](https://rob9315.github.io/mcpackets/13w41b.html#packets:play_clientbound_0d)
///
/// [`PlayerPosition0`]: super::serverbound::PlayerPosition0
/// [`PlayerPositionAndLook0`]: super::serverbound::PlayerPositionAndLook0
pub struct CollectItem0 {
    /// The item's entity id
    pub collected_id: i32,
    /// The entity's id that collected the item
    pub collector_id: i32,
}

#[derive(Encoding, ToStatic, Debug)]
/// Collect Item
///
/// Sent when an entity collects an item.
///
/// The sole purpose seems to be to play the animation of the item flying
/// towards the collector.
///
/// The vanilla server only checks for items to be picked up after each
/// [`PlayerPosition0`]/[`PlayerPosition10`] and
/// [`PlayerPositionAndLook0`]/[`PlayerPositionAndLook10`] packet sent by the
/// client.
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5392#Collect_Item)
/// [burger diff](https://rob9315.github.io/mcpackets/diff_6_7.html#packets:play_clientbound_0d)
///
/// [`PlayerPosition0`]: super::serverbound::PlayerPosition0
/// [`PlayerPosition10`]: super::serverbound::PlayerPosition10
/// [`PlayerPositionAndLook0`]: super::serverbound::PlayerPositionAndLook0
/// [`PlayerPositionAndLook10`]: super::serverbound::PlayerPositionAndLook10
pub struct CollectItem7 {
    #[encoding(varint)]
    /// The item's entity id
    pub collected_id: i32,
    #[encoding(varint)]
    /// The entity's id that collected the item
    pub collector_id: i32,
}

#[derive(Encoding, ToStatic, Debug)]
/// Spawn Object
///
/// Sent by the server when a vehicle or other object is created.
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5007#Spawn_Object)
/// [burger](https://rob9315.github.io/mcpackets/13w41b.html#packets:play_clientbound_0e)
pub struct SpawnObject0 {
    #[encoding(varint)]
    pub entity_id: i32,
    pub kind: ObjectKind0,
    #[encoding(fixed(5, "i32"))]
    /// X position as a Fixed-Point number
    pub x: f64,
    #[encoding(fixed(5, "i32"))]
    /// Y position as a Fixed-Point number
    pub y: f64,
    #[encoding(fixed(5, "i32"))]
    /// Z position as a Fixed-Point number
    pub z: f64,
    pub pitch: Angle,
    pub yaw: Angle,
    /// Extra Data
    ///
    /// Meaning depends on [`kind`](#structfield.kind), see [wiki.vg on Object data](https://wiki.vg/Object_Data)
    pub data: ObjectData0,
}

#[derive(Encoding, ToStatic, Debug)]
#[encoding(from = "u8")]
pub enum ObjectKind0 {
    #[encoding(case = "1")]
    Boat,
    ItemStack,
    AreaEffectCloud,
    #[encoding(case = "10")]
    Minecart,
    /// unused since 1.6.x
    StorageMinecart,
    /// unused since 1.6.x
    PoweredMinecart,
    #[encoding(case = "50")]
    ActivatedTNT,
    EnderCrystal,
    #[encoding(case = "60")]
    Arrow,
    Snowball,
    Egg,
    Fireball,
    FireCharge,
    ThrownEnderpearl,
    WitherSkull,
    ShulkerBullet,
    #[encoding(case = "70")]
    FallingObject,
    ItemFrame,
    EyeOfEnder,
    ThrownPotion,
    FallingDragonEgg,
    ThrownExpBottle,
    FireworkRocket,
    LeashKnot,
    ArmorStand,
    #[encoding(case = "90")]
    FishingFloat,
    SpectralArrow,
    TippedArrow,
    DragonFireball,
}

/// Object Data
///
/// Special Data type for Additional Data for Objects in pv0..=pv48.
///
/// If the read integer is ~~nonzero~~ greater than 0, three additional shorts are read.
///
/// Meaning depends on associated [`ObjectKind0`]. Most of the time it is
/// ignored.
///
/// [wiki.vg](https://wiki.vg/Object_Data)
#[derive(ToStatic, Debug)]
pub enum ObjectData0 {
    ZeroOrLess(i32),
    Extra { value: i32, x: i16, y: i16, z: i16 },
}

impl<'dec> Decode<'dec> for ObjectData0 {
    fn decode(cursor: &mut std::io::Cursor<&'dec [u8]>) -> decode::Result<Self> {
        Ok(match i32::decode(cursor)? {
            v if v <= 0 => ObjectData0::ZeroOrLess(v),
            value => ObjectData0::Extra {
                value,
                x: i16::decode(cursor)?,
                y: i16::decode(cursor)?,
                z: i16::decode(cursor)?,
            },
        })
    }
}

impl Encode for ObjectData0 {
    fn encode(&self, writer: &mut impl std::io::Write) -> encode::Result<()> {
        match self {
            ObjectData0::ZeroOrLess(value) => {
                #[cfg(debug_assertions)]
                if *value > 0 {
                    return Err(encode::Error::Custom(
                        "Invalid int value > 0 in ObjectData::ZeroOrLess",
                    ));
                }
                value.encode(writer)?;
            }
            ObjectData0::Extra { value, x, y, z } => {
                #[cfg(debug_assertions)]
                if *value <= 0 {
                    return Err(encode::Error::Custom(
                        "Invalid int value <= 0 in ObjectData::Extra",
                    ));
                }
                value.encode(writer)?;
                x.encode(writer)?;
                y.encode(writer)?;
                z.encode(writer)?;
            }
        };
        Ok(())
    }
}

#[derive(Encoding, ToStatic, Debug)]
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

#[derive(Encoding, ToStatic, Debug)]
/// Spawn Mob
///
/// Sent by the server when a Mob Entity is Spawned.
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5007#Spawn_Mob)
/// [burger](https://rob9315.github.io/mcpackets/13w41b.html#packets:play_clientbound_0f)
pub struct SpawnMob0<'a> {
    #[encoding(varint)]
    pub entity_id: i32,
    pub kind: EntityKind0,
    #[encoding(fixed(5, "i32"))]
    pub x: f64,
    #[encoding(fixed(5, "i32"))]
    pub y: f64,
    #[encoding(fixed(5, "i32"))]
    pub z: f64,
    pub pitch: Angle,
    pub head_pitch: Angle,
    pub yaw: Angle,
    /// X Velocity
    ///
    /// believed to be in units of 1/8000 of a block per server tick (50ms);
    pub velocity_x: i16,
    /// Y Velocity
    ///
    /// believed to be in units of 1/8000 of a block per server tick (50ms);
    pub velocity_y: i16,
    /// Z Velocity
    ///
    /// believed to be in units of 1/8000 of a block per server tick (50ms);
    pub velocity_z: i16,
    pub metadata: PackedEntityMetadata0<'a>,
}

#[derive(Encoding, ToStatic, Debug)]
/// Spawn Painting
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5007#Spawn_Painting)
/// [burger](https://rob9315.github.io/mcpackets/13w41b.html#packets:play_clientbound_10)
pub struct SpawnPainting0<'a> {
    #[encoding(varint)]
    pub entity_id: i32,
    // TODO: #[max_len(13)]
    /// Name of the painting. Max length 13
    pub title: Cow<'a, str>,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub direction: Direction0,
}
#[derive(Encoding, ToStatic, Debug)]
/// Spawn Painting
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5408#Spawn_Painting)
/// [burger diff](https://rob9315.github.io/mcpackets/diff_7_8.html#packets:play_clientbound_10)
pub struct SpawnPainting8<'a> {
    #[encoding(varint)]
    pub entity_id: i32,
    // TODO: #[max_len(13)]
    /// Name of the painting. Max length 13
    pub title: Cow<'a, str>,
    pub location: Position6,
    pub direction: Direction0,
}

#[derive(Encoding, ToStatic, Debug)]
#[encoding(from = "u32")]
pub enum Direction0 {
    NegZ = 0,
    NegX,
    PosZ,
    PosX,
}

#[derive(Encoding, ToStatic, Debug)]
/// Spawn Experience Orb
///
/// Spawns one or more experience orbs.
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5007#Spawn_Experience_Orb)
/// [burger](https://rob9315.github.io/mcpackets/13w41b.html#packets:play_clientbound_11)
pub struct SpawnExpOrb0 {
    #[encoding(varint)]
    pub entity_id: i32,
    #[encoding(fixed(5, "i32"))]
    pub x: f64,
    #[encoding(fixed(5, "i32"))]
    pub y: f64,
    #[encoding(fixed(5, "i32"))]
    pub z: f64,
    /// The amount of experience this orb will reward once collected
    pub count: i16,
}

#[derive(ToStatic, Debug)]
/// Entity Velocity
///
/// Sets the Velocity of an Entity
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5007#Entity_Velocity)
/// [burger](https://rob9315.github.io/mcpackets/13w41b.html#packets:play_clientbound_12)
pub struct EntityVelocity0 {
    pub entity_id: i32,
    // TODO: decide for or against converting to and from f32 here
    // if deciding FOR it, also update SpawnMob0
    /// value is clamped to +3.9 and -3.9 in the notchian client
    pub x: f32,
    /// value is clamped to +3.9 and -3.9 in the notchian client
    pub y: f32,
    /// value is clamped to +3.9 and -3.9 in the notchian client
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
#[derive(ToStatic, Debug)]
/// Entity Velocity
///
/// Sets the Velocity of an Entity
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5392#Entity_Velocity)
/// [burger diff](https://rob9315.github.io/mcpackets/diff_6_7.html#packets:play_clientbound_12)
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

#[derive(Encoding, ToStatic, Debug)]
/// Destroy Entities
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5007#Destroy_Entities)
/// [burger](https://rob9315.github.io/mcpackets/13w41b.html#packets:play_clientbound_13)
pub struct DestroyEntities0 {
    #[encoding(counted = "u8")]
    pub entities: Vec<i32>,
}

#[derive(ToStatic, Debug)]
/// Destroy Entities
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5392#Destroy_Entities)
/// [burger diff](https://rob9315.github.io/mcpackets/diff_6_7.html#packets:play_clientbound_13)
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

#[derive(Encoding, ToStatic, Debug)]
/// Entity Update/Init (No Move, No Look)
///
/// This packet may be used to initialize an entity.
///
/// For player entities, either this packet or any move/look packet is sent
/// every game tick by the notchian server. So the meaning of this packet is
/// basically that the entity did not move/look since the last such packet.
///
/// This Packet represents a move/look packet without any changes.
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5007#Entity)
/// [burger](https://rob9315.github.io/mcpackets/13w41b.html#packets:play_clientbound_14)
pub struct Entity0 {
    pub entity_id: i32,
}

#[derive(Encoding, ToStatic, Debug)]
/// Entity Update/Init (No Move, No Look)
///
/// This packet may be used to initialize an entity.
///
/// For player entities, either this packet or any move/look packet is sent
/// every game tick by the notchian server. So the meaning of this packet is
/// basically that the entity did not move/look since the last such packet.
///
/// This Packet represents a move/look packet without any changes.
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5392#Entity)
/// [burger diff](https://rob9315.github.io/mcpackets/diff_6_7.html#packets:play_clientbound_14)
pub struct Entity7 {
    #[encoding(varint)]
    pub entity_id: i32,
}

#[derive(Encoding, ToStatic, Debug)]
/// Entity Update (Relative Move)
///
/// Relative entity movement of at most 4 blocks.
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5007#Entity_Relative_Move)
/// [burger](https://rob9315.github.io/mcpackets/13w41b.html#packets:play_clientbound_15)
pub struct EntityRelativeMove0 {
    pub entity_id: i32,
    // TODO: round x and z but floor y
    /// watch out, this must satisfy -4.0 <= x < 4.0
    /// if it can't, use [`EntityTeleport0`] instead
    #[encoding(fixed(5, "i8"))]
    pub dx: f32,
    /// watch out, this must satisfy -4.0 <= x < 4.0
    /// if it can't, use [`EntityTeleport0`] instead
    #[encoding(fixed(5, "i8"))]
    pub dy: f32,
    /// watch out, this must satisfy -4.0 <= x < 4.0
    /// if it can't, use [`EntityTeleport0`] instead
    #[encoding(fixed(5, "i8"))]
    pub dz: f32,
}

#[derive(Encoding, ToStatic, Debug)]
/// Entity Update (Relative Move)
///
/// Relative entity movement of at most 4 blocks.
///
/// no wiki.vg
/// [burger diff](https://rob9315.github.io/mcpackets/diff_6_7.html#packets:play_clientbound_15)
pub struct EntityRelativeMove7 {
    #[encoding(varint)]
    pub entity_id: i32,
    // TODO: round x and z but floor y
    /// watch out, this must satisfy -4.0 <= x < 4.0
    /// if it can't, use [`EntityTeleport7`] instead
    #[encoding(fixed(5, "i8"))]
    pub dx: f32,
    /// watch out, this must satisfy -4.0 <= x < 4.0
    /// if it can't, use [`EntityTeleport7`] instead
    #[encoding(fixed(5, "i8"))]
    pub dy: f32,
    /// watch out, this must satisfy -4.0 <= x < 4.0
    /// if it can't, use [`EntityTeleport7`] instead
    #[encoding(fixed(5, "i8"))]
    pub dz: f32,
}

#[derive(Encoding, ToStatic, Debug)]
/// Entity Update (Relative Move)
///
/// Relative entity movement of at most 4 blocks.
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5706#Entity_Relative_Move)
/// [burger diff](https://rob9315.github.io/mcpackets/diff_21_22.html#packets:play_clientbound_15)
pub struct EntityRelativeMove22 {
    #[encoding(varint)]
    pub entity_id: i32,
    // TODO: round x and z but floor y
    /// watch out, this must satisfy -4.0 <= x < 4.0
    /// if it can't, use [`EntityTeleport22`] instead
    #[encoding(fixed(5, "i8"))]
    pub dx: f32,
    /// watch out, this must satisfy -4.0 <= x < 4.0
    /// if it can't, use [`EntityTeleport22`] instead
    #[encoding(fixed(5, "i8"))]
    pub dy: f32,
    /// watch out, this must satisfy -4.0 <= x < 4.0
    /// if it can't, use [`EntityTeleport22`] instead
    #[encoding(fixed(5, "i8"))]
    pub dz: f32,
    /// whether or not the entity is currently located on the ground
    pub on_ground: bool,
}

#[derive(Encoding, ToStatic, Debug)]
/// Entity Update (Look)
///
/// This packet is sent by the server when an entity rotates.
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5007#Entity_Look)
/// [burger](https://rob9315.github.io/mcpackets/13w41b.html#packets:play_clientbound_16)
pub struct EntityLook0 {
    pub entity_id: i32,
    pub yaw: Angle,
    pub pitch: Angle,
}

#[derive(Encoding, ToStatic, Debug)]
/// Entity Update (Look)
///
/// This packet is sent by the server when an entity rotates.
///
/// no wiki.vg
/// [burger diff](https://rob9315.github.io/mcpackets/diff_6_7.html#packets:play_clientbound_16)
pub struct EntityLook7 {
    #[encoding(varint)]
    pub entity_id: i32,
    pub yaw: Angle,
    pub pitch: Angle,
}

#[derive(Encoding, ToStatic, Debug)]
/// Entity Update (Look)
///
/// This packet is sent by the server when an entity rotates.
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5706#Entity_Look)
/// [burger diff](https://rob9315.github.io/mcpackets/diff_21_22.html#packets:play_clientbound_16)
pub struct EntityLook22 {
    #[encoding(varint)]
    pub entity_id: i32,
    pub yaw: Angle,
    pub pitch: Angle,
    /// whether or not the entity is currently located on the ground
    pub on_ground: bool,
}

#[derive(Encoding, ToStatic, Debug)]
/// Entity Update (Relative Move, Look)
///
/// This packet is sent by the server when an entity rotates and moves less
/// than 4 blocks.
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5007#Entity_Look_and_Relative_Move)
/// [burger](https://rob9315.github.io/mcpackets/13w41b.html#packets:play_clientbound_17)
pub struct EntityLookAndRelativeMove0 {
    pub entity_id: i32,
    // TODO: round x and z but floor y
    /// watch out, this must satisfy -4.0 <= x < 4.0
    /// if it can't, use [`EntityTeleport0`] instead
    #[encoding(fixed(5, "i8"))]
    pub dx: f32,
    /// watch out, this must satisfy -4.0 <= x < 4.0
    /// if it can't, use [`EntityTeleport0`] instead
    #[encoding(fixed(5, "i8"))]
    pub dy: f32,
    /// watch out, this must satisfy -4.0 <= x < 4.0
    /// if it can't, use [`EntityTeleport0`] instead
    #[encoding(fixed(5, "i8"))]
    pub dz: f32,
    pub yaw: Angle,
    pub pitch: Angle,
}

#[derive(Encoding, ToStatic, Debug)]
/// Entity Update (Relative Move, Look)
///
/// This packet is sent by the server when an entity rotates and moves less
/// than 4 blocks.
///
/// no wiki.vg
/// [burger diff](https://rob9315.github.io/mcpackets/diff_6_7.html#packets:play_clientbound_17)
pub struct EntityLookAndRelativeMove7 {
    #[encoding(varint)]
    pub entity_id: i32,
    // TODO: round x and z but floor y
    /// watch out, this must satisfy -4.0 <= x < 4.0
    /// if it can't, use [`EntityTeleport7`] instead
    #[encoding(fixed(5, "i8"))]
    pub dx: f32,
    /// watch out, this must satisfy -4.0 <= x < 4.0
    /// if it can't, use [`EntityTeleport7`] instead
    #[encoding(fixed(5, "i8"))]
    pub dy: f32,
    /// watch out, this must satisfy -4.0 <= x < 4.0
    /// if it can't, use [`EntityTeleport7`] instead
    #[encoding(fixed(5, "i8"))]
    pub dz: f32,
    pub yaw: Angle,
    pub pitch: Angle,
}

#[derive(Encoding, ToStatic, Debug)]
/// Entity Update (Relative Move, Look)
///
/// This packet is sent by the server when an entity rotates and moves less
/// than 4 blocks.
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5706#Entity_Look_and_Relative_Move)
/// [burger diff](https://rob9315.github.io/mcpackets/diff_21_22.html#packets:play_clientbound_17)
pub struct EntityLookAndRelativeMove22 {
    #[encoding(varint)]
    pub entity_id: i32,
    // TODO: round x and z but floor y
    /// watch out, this must satisfy -4.0 <= x < 4.0
    /// if it can't, use [`EntityTeleport22`] instead
    #[encoding(fixed(5, "i8"))]
    pub dx: f32,
    /// watch out, this must satisfy -4.0 <= x < 4.0
    /// if it can't, use [`EntityTeleport22`] instead
    #[encoding(fixed(5, "i8"))]
    pub dy: f32,
    /// watch out, this must satisfy -4.0 <= x < 4.0
    /// if it can't, use [`EntityTeleport22`] instead
    #[encoding(fixed(5, "i8"))]
    pub dz: f32,
    pub yaw: Angle,
    pub pitch: Angle,
    /// whether or not the entity is currently located on the ground
    pub on_ground: bool,
}

#[derive(Encoding, ToStatic, Debug)]
/// Entity Update (Absolute Move, Look)
///
/// This packet is sent by the server when an entity moves more than 4 blocks.
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5007#Entity_Teleport)
/// [burger](https://rob9315.github.io/mcpackets/diff_6_7.html#packets:play_clientbound_18)
pub struct EntityTeleport0 {
    pub entity_id: i32,
    // TODO: round x and z but floor y
    #[encoding(fixed(5, "i32"))]
    pub x: f64,
    #[encoding(fixed(5, "i32"))]
    pub y: f64,
    #[encoding(fixed(5, "i32"))]
    pub z: f64,
    pub yaw: Angle,
    pub pitch: Angle,
}

#[derive(Encoding, ToStatic, Debug)]
/// Entity Update (Absolute Move, Look)
///
/// This packet is sent by the server when an entity moves more than 4 blocks.
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5392#Entity_Teleport)
/// [burger diff](https://rob9315.github.io/mcpackets/diff_6_7.html#packets:play_clientbound_18)
pub struct EntityTeleport7 {
    #[encoding(varint)]
    pub entity_id: i32,
    // TODO: round x and z but floor y
    #[encoding(fixed(5, "i32"))]
    pub x: f64,
    #[encoding(fixed(5, "i32"))]
    pub y: f64,
    #[encoding(fixed(5, "i32"))]
    pub z: f64,
    pub yaw: Angle,
    pub pitch: Angle,
}

#[derive(Encoding, ToStatic, Debug)]
/// Entity Update (Absolute Move, Look)
///
/// This packet is sent by the server when an entity moves more than 4 blocks.
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5706#Entity_Teleport)
/// [burger diff](https://rob9315.github.io/mcpackets/diff_21_22.html#packets:play_clientbound_18)
pub struct EntityTeleport22 {
    #[encoding(varint)]
    pub entity_id: i32,
    // TODO: round x and z but floor y
    #[encoding(fixed(5, "i32"))]
    pub x: f64,
    #[encoding(fixed(5, "i32"))]
    pub y: f64,
    #[encoding(fixed(5, "i32"))]
    pub z: f64,
    pub yaw: Angle,
    pub pitch: Angle,
    /// whether or not the entity is currently located on the ground
    pub on_ground: bool,
}

#[derive(Encoding, ToStatic, Debug)]
/// Entity Update (Head Look)
///
/// Changes the direction an entity's head is facing.
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5007#Entity_Head_Look)
/// [burger](https://rob9315.github.io/mcpackets/13w41b.html#packets:play_clientbound_19)
pub struct EntityHeadLook0 {
    pub entity_id: i32,
    pub head_yaw: Angle,
}

#[derive(Encoding, ToStatic, Debug)]
/// Entity Update (Head Look)
///
/// Changes the direction an entity's head is facing.
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5392#Entity_Head_Look)
/// [burger](https://rob9315.github.io/mcpackets/diff_6_7.html#packets:play_clientbound_19)
pub struct EntityHeadLook7 {
    #[encoding(varint)]
    pub entity_id: i32,
    pub head_yaw: Angle,
}

#[derive(Encoding, ToStatic, Debug)]
/// Entity Status
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5007#Entity_Status)
/// [burger](https://rob9315.github.io/mcpackets/13w41b.html#packets:play_clientbound_1a)
pub struct EntityStatus0 {
    pub entity_id: i32,
    pub entity_status: Status0,
}

#[derive(Encoding, ToStatic, Debug)]
#[encoding(from = "u8")]
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

#[derive(Encoding, ToStatic, Debug)]
/// Attach Entity
///
/// This packet is sent when an entity is attached to another entity.
/// (e.g. a player entering a minecart, a sheep being leashed)
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5007#Attach_Entity)
/// [burger](https://rob9315.github.io/mcpackets/13w41b.html#packets:play_clientbound_1b)
pub struct AttachEntity0 {
    pub entity_id: i32,
    pub vehicle_id: i32,
    /// If true leashes the entity to the vehicle
    pub leash: bool,
}

#[derive(Encoding, ToStatic, Debug)]
/// Entity Metadata
///
/// Updates one or more metadata properties for an existing entity. Any
/// properties not included in the Metadata field are left unchanged.
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5007#Entity_Metadata)
/// [burger](https://rob9315.github.io/mcpackets/13w41b.html#packets:play_clientbound_1c)
pub struct EntityMetadata0<'a> {
    pub entity_id: i32,
    pub metadata: PackedEntityMetadata0<'a>,
}

#[derive(Encoding, ToStatic, Debug)]
/// Entity Metadata
///
/// Updates one or more metadata properties for an existing entity. Any
/// properties not included in the Metadata field are left unchanged.
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5392#Entity_Metadata)
/// [burger diff](https://rob9315.github.io/mcpackets/diff_6_7.html#packets:play_clientbound_1c)
pub struct EntityMetadata7<EntityMetadata> {
    #[encoding(varint)]
    pub entity_id: i32,
    pub metadata: EntityMetadata,
}

#[derive(Encoding, ToStatic, Debug)]
/// Entity Effect
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5007#Entity_Effect)
/// [burger](https://rob9315.github.io/mcpackets/13w41b.html#packets:play_clientbound_1d)
pub struct EntityEffect0 {
    pub entity_id: i32,
    // TODO: effect ids
    pub effect_id: i8,
    /// Effect Level - 1
    pub amplifier: i8,
    /// in seconds
    pub duration: i16,
}

#[derive(Encoding, ToStatic, Debug)]
/// Entity Effect
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5392#Entity_Effect)
/// [burger diff](https://rob9315.github.io/mcpackets/diff_6_7.html#packets:play_clientbound_1d)
pub struct EntityEffect7 {
    #[encoding(varint)]
    pub entity_id: i32,
    // TODO: effect ids
    pub effect_id: i8,
    /// Effect Level - 1
    pub amplifier: i8,
    #[encoding(varint)]
    /// in seconds
    pub duration: i32,
}

#[derive(Encoding, ToStatic, Debug)]
/// Entity Effect
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5422#Entity_Effect)
/// [burger diff](https://rob9315.github.io/mcpackets/diff_9_10.html#packets:play_clientbound_1d)
pub struct EntityEffect10 {
    #[encoding(varint)]
    pub entity_id: i32,
    // TODO: effect ids
    pub effect_id: i8,
    /// Effect Level - 1
    pub amplifier: i8,
    #[encoding(varint)]
    /// in seconds
    pub duration: i32,
    pub hide_particles: bool,
}

#[derive(Encoding, ToStatic, Debug)]
/// Remove Entity Effect
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5007#Remove_Entity_Effect)
/// [burger](https://rob9315.github.io/mcpackets/13w41b.html#packets:play_clientbound_1e)
pub struct RemoveEntityEffect0 {
    pub entity_id: i32,
    pub effect_id: i8,
}
#[derive(Encoding, ToStatic, Debug)]
/// Remove Entity Effect
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5392#Remove_Entity_Effect)
/// [burger diff](https://rob9315.github.io/mcpackets/diff_6_7.html#packets:play_clientbound_1e)
pub struct RemoveEntityEffect7 {
    #[encoding(varint)]
    pub entity_id: i32,
    pub effect_id: i8,
}

#[derive(Encoding, ToStatic, Debug)]
/// Set Experience
///
/// Sent by the server when the client should change experience levels.
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5007#Set_Experience)
/// [burger](https://rob9315.github.io/mcpackets/13w41b.html#packets:play_clientbound_1f)
pub struct SetExperience0 {
    pub experience_bar: f32,
    pub level: i16,
    pub total_exp: i16,
}

#[derive(Encoding, ToStatic, Debug)]
/// Set Experience
///
/// Sent by the server when the client should change experience levels.
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5392#Set_Experience)
/// [burger diff](https://rob9315.github.io/mcpackets/diff_6_7.html#packets:play_clientbound_1f)
pub struct SetExperience7 {
    pub experience_bar: f32,
    #[encoding(varint)]
    pub level: i32,
    #[encoding(varint)]
    pub total_exp: i32,
}

// TODO: Add more documentation on this, maybe a list of known properties
#[derive(Encoding, ToStatic, Debug)]
/// Entity Update (Properties)
///
/// Sets attributes on the given entity.
///
/// Attributes are a system of buffs/debuffs that are properties on mobs and
/// players. Attributes also have modifiers that adjust the strength of their
/// effect.
///
/// [Minecraft Wiki](https://minecraft.fandom.com/wiki/Attribute)
///
/// [wiki.vg](https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5007#Entity_Properties)
/// [burger](https://rob9315.github.io/mcpackets/13w41b.html#packets:play_clientbound_20)
pub struct EntityProperties0<'a> {
    pub entity_id: i32,
    #[encoding(counted = "u32")]
    pub properties: Vec<EntityProperty0<'a>>,
}
#[derive(Encoding, ToStatic, Debug)]
pub struct EntityProperty0<'a> {
    pub key: Cow<'a, str>,
    pub value: f64,
    #[encoding(counted = "u16")]
    pub modifiers: Vec<Modifier0>,
}
#[derive(Encoding, ToStatic, Debug)]
pub struct Modifier0 {
    pub uuid: Uuid,
    pub amount: f64,
    pub operation: ModifierOperation0,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct EntityProperties7<'a> {
    pub entity_id: i32,
    #[encoding(counted = "u32")]
    pub properties: Vec<EntityProperty7<'a>>,
}
#[derive(Encoding, ToStatic, Debug)]
pub struct EntityProperty7<'a> {
    pub key: Cow<'a, str>,
    pub value: f64,
    pub modifiers: Vec<Modifier0>,
}

#[derive(Encoding, ToStatic, Debug)]
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
/// <https://minecraft.fandom.com/wiki/Attribute#Vanilla_modifiers>
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

#[derive(Encoding, ToStatic, Debug)]
// TODO: make this nice to interact with
pub struct ChunkData0<'a> {
    pub chunk_x: i32,
    pub chunk_y: i32,
    /// This is True if the packet represents all sections in this vertical
    /// column, where the primary bit map specifies exactly which sections are
    /// included, and which are air
    pub continuous: bool,
    /// Bitmask with 1 for every 16x16x16 section which data follows in [`compressed_data`](#structfield.compressed_data).
    pub primary_bitmap: u16,
    // TODO: what is this for?
    /// Same as [`primary_bitmap`](#structfield.primary_bitmap), but this is used exclusively for the 'add' portion of the payload
    pub add_bitmap: u16,
    #[encoding(counted = "u32")]
    /// The chunk data is compressed using Zlib Deflate
    pub compressed_data: Cow<'a, [u8]>,
}

#[derive(Encoding, ToStatic, Debug)]
// TODO: make this nice to interact with
pub struct ChunkData23<'a> {
    pub chunk_x: i32,
    pub chunk_y: i32,
    /// This is True if the packet represents all sections in this vertical
    /// column, where the primary bit map specifies exactly which sections are
    /// included, and which are air
    pub continuous: bool,
    /// Bitmask with 1 for every 16x16x16 section which data follows in the compressed data.
    pub primary_bitmap: u16,
    #[encoding(counted = "u32")]
    pub compressed_data: Cow<'a, [u8]>, //TODO: Verify if the data is compressed in pv23 or not and change naming to `data` if not.
}

#[derive(Encoding, ToStatic, Debug)]
// TODO: make this nice to interact with
pub struct ChunkData27<'a> {
    pub chunk_x: i32,
    pub chunk_y: i32,
    /// This is True if the packet represents all sections in this vertical
    /// column, where the primary bit map specifies exactly which sections are
    /// included, and which are air
    pub continuous: bool,
    /// Bitmask with 1 for every 16x16x16 section which data follows in the compressed data.
    pub primary_bitmap: u16,
    pub data: Cow<'a, [u8]>,
}

#[derive(ToStatic, Debug)]
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
            // TODO: different error
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

#[derive(ToStatic, Debug)]
pub struct MultiBlockChange4 {
    pub chunk_x: i32,
    pub chunk_z: i32,
    // count(u16)
    pub records: Vec<Record>,
}

impl<'dec> Decode<'dec> for MultiBlockChange4 {
    fn decode(cursor: &mut std::io::Cursor<&'dec [u8]>) -> decode::Result<Self> {
        let chunk_x = i32::decode(cursor)?;
        let chunk_z = i32::decode(cursor)?;
        let record_count = u16::decode(cursor)?;
        let data_size: i32 = i32::decode(cursor)?;
        if data_size != record_count as i32 * 4 {
            // TODO: different error
            return Err(decode::Error::InvalidId);
        }
        let records: Vec<_> = (0..record_count)
            .map(|_| Record::decode(cursor))
            .collect::<Result<_, _>>()?;
        Ok(Self {
            chunk_x,
            chunk_z,
            records,
        })
    }
}

impl Encode for MultiBlockChange4 {
    fn encode(&self, writer: &mut impl std::io::Write) -> Result<(), encode::Error> {
        self.chunk_x.encode(writer)?;
        self.chunk_z.encode(writer)?;
        (self.records.len() as u16).encode(writer)?;
        (self.records.len() as i32 * 4).encode(writer)?;
        for record in &self.records {
            record.encode(writer)?;
        }
        Ok(())
    }
}

#[derive(ToStatic, Debug)]
pub struct Record {
    pub block_state: u16,
    pub y: u8,
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

#[derive(Encoding, ToStatic, Debug)]
pub struct MultiBlockChange25 {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub records: Vec<Record25>,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct Record25 {
    pub rel_pos: RecordRelativePosition25,
    #[encoding(varint)]
    pub block_id: i32,
}

#[derive(Bitfield, ToStatic, Debug)]
pub struct RecordRelativePosition25 {
    #[encoding(bits = 4)]
    pub x: u8,
    #[encoding(bits = 4)]
    pub z: u8,
    #[encoding(bits = 8)]
    pub y: u8,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct BlockChange0 {
    pub x: i32,
    pub y: u8,
    pub z: i32,
    #[encoding(varint)]
    // TODO: extract the next two variables into concrete types for the version
    pub block_type: i32,
    pub block_data: u8,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct BlockChange6 {
    pub location: Position6,
    #[encoding(varint)]
    pub block_type: i32,
    pub block_data: u8,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct BlockChange25 {
    pub location: Position6,
    // TODO: global palette block id, maybe separate into id and type?
    // https://wiki.vg/index.php?title=Protocol&oldid=7368#Block_Change
    #[encoding(varint)]
    pub block_id: i32,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct BlockAction0 {
    pub x: i32,
    pub y: i16,
    pub z: i32,
    pub action_id: u8,
    pub action_param: u8,
    /// The block type ID for the block, not including metadata/damage value
    #[encoding(varint)]
    pub block_type: i32,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct BlockAction6 {
    pub location: Position6,
    pub action_id: u8,
    pub action_param: u8,
    /// The block type ID for the block, not including metadata/damage value
    #[encoding(varint)]
    pub block_type: i32,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct BlockBreakAnimation0 {
    #[encoding(varint)]
    pub entity_id: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    /// 0-9
    pub destroy_stage: u8,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct BlockBreakAnimation6 {
    #[encoding(varint)]
    pub entity_id: i32,
    pub location: Position6,
    /// 0-9
    pub destroy_stage: u8,
}

#[derive(ToStatic, Debug)]
pub struct MapChunkBulk0<'a> {
    /// Whether or not the chunk data contains a light nibble array. This is
    /// true in the main world, false in the end + nether
    pub skylight_sent: bool,
    pub data: Cow<'a, [u8]>,
    pub column_metas: Vec<ChunkMeta0>,
}
impl<'dec: 'a, 'a> Decode<'dec> for MapChunkBulk0<'a> {
    fn decode(cursor: &'_ mut std::io::Cursor<&'dec [u8]>) -> decode::Result<Self> {
        let column_count = u16::decode(cursor)?;
        let data_len = u32::decode(cursor)?;
        let skylight_sent = bool::decode(cursor)?;
        let pos = cursor.position();
        let data = cursor
            .get_ref()
            .get(pos as usize..data_len as usize + pos as usize)
            // TODO: different error
            .ok_or(decode::Error::InvalidId)?;
        cursor.set_position(data_len as u64 + pos);
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

#[derive(Encoding, ToStatic, Debug)]
pub struct ChunkMeta0 {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub primary_bitmap: u16,
    pub add_bitmap: u16,
}

#[derive(ToStatic, Debug)]
pub struct MapChunkBulk23<'a> {
    /// Whether or not the chunk data contains a light nibble array. This is
    /// true in the main world, false in the end + nether
    pub skylight_sent: bool,
    pub data: Cow<'a, [u8]>,
    pub column_metas: Vec<ChunkMeta23>,
}

impl<'dec: 'a, 'a> Decode<'dec> for MapChunkBulk23<'a> {
    fn decode(cursor: &'_ mut std::io::Cursor<&'dec [u8]>) -> decode::Result<Self> {
        let column_count = u16::decode(cursor)?;
        let data_len = u32::decode(cursor)?;
        let skylight_sent = bool::decode(cursor)?;
        let pos = cursor.position();
        let data = cursor
            .get_ref()
            .get(pos as usize..data_len as usize + pos as usize)
            // TODO: different error
            .ok_or(decode::Error::InvalidId)?;
        cursor.set_position(data_len as u64 + pos);
        let column_metas = (0..column_count)
            .map(|_| ChunkMeta23::decode(cursor))
            .collect::<Result<_, _>>()?;
        Ok(Self {
            skylight_sent,
            data: Cow::Borrowed(data),
            column_metas,
        })
    }
}
impl<'a> Encode for MapChunkBulk23<'a> {
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

#[derive(Encoding, ToStatic, Debug)]
pub struct ChunkMeta23 {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub primary_bitmap: u16,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct MapChunkBulk27<'a> {
    /// Whether or not the chunk data contains a light nibble array. This is
    /// true in the main world, false in the end + nether
    pub skylight_sent: bool,
    #[encoding(counted = "u32")]
    pub column_metas: Vec<ChunkMeta23>,
    #[encoding(rest)]
    pub data: Cow<'a, [u8]>,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct Explosion0 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub radius: f32,
    #[encoding(counted = "u32")]
    pub records: Vec<ExplosionRecord>,
    pub motion_x: f32,
    pub motion_y: f32,
    pub motion_z: f32,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct ExplosionRecord {
    pub dx: i8,
    pub dy: i8,
    pub dz: i8,
}

#[derive(Encoding, ToStatic, Debug)]
// TODO: more detailed data using #[separated]
pub struct Effect0 {
    pub effect_id: i32,
    // TODO: relative? fixed point?
    /// The X location of the effect multiplied by 8
    pub x: i32,
    /// The Y location of the effect multiplied by 8
    pub y: i8,
    /// The Z location of the effect multiplied by 8
    pub z: i32,
    pub effect_data: i32,
    pub disable_rel_volume: bool,
}

#[derive(Encoding, ToStatic, Debug)]
// TODO: see above
pub struct Effect6 {
    pub effect_id: i32,
    pub location: Position6,
    pub effect_data: i32,
    pub disable_rel_volume: bool,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct SoundEffect0<'a> {
    pub effect_id: Cow<'a, str>,
    // TODO: relative? fixed point?
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

#[derive(Encoding, ToStatic, Debug)]
#[encoding(from = "u8")]
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

#[derive(Encoding, ToStatic, Debug)]
pub struct SoundEffect1<'a> {
    pub effect_id: Cow<'a, str>,
    // TODO: relative? fixed point?
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

#[derive(Encoding, ToStatic, Debug)]
pub struct Particle0<'a> {
    // TODO: specific strings into enum
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

#[derive(Encoding, ToStatic, Debug)]
pub struct Particle17<'a> {
    // TODO: specific strings into enum
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
    // TODO: read exact number of varints using enum of possible names
    // https://wiki.vg/index.php?title=Protocol&oldid=7368#Particle_2
    #[encoding(rest)]
    pub data: Cow<'a, [u8]>,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct Particle29<'a> {
    // TODO: specific strings into enum
    pub name: Cow<'a, str>,
    /// If true, particle distance increases from 256 to 65536
    pub long_distance: bool,
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
    // TODO: read exact number of varints using enum of possible names
    // https://wiki.vg/index.php?title=Protocol&oldid=7368#Particle_2
    #[encoding(rest)]
    pub data: Cow<'a, [u8]>,
}

// #[derive(Encoding, ToStatic, Debug)]
// struct ChangeGameState0 {
//     reason: GameStateChangeReason,
// }

// #[derive(Encoding, ToStatic, Debug)]
// #[encoding(from = "u8")]
#[derive(ToStatic, Debug)]
pub enum ChangeGameState0 {
    // #[encoding(case = "0")]
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

#[derive(Encoding, ToStatic, Clone, Copy, Debug)]
#[encoding(from = "u8")]
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

#[derive(Encoding, ToStatic, Debug)]
pub struct SpawnGlobalEntity0 {
    #[encoding(varint)]
    pub entity_id: i32,
    /// The global entity type, currently always 1 for thunderbolt.
    pub kind: u8,
    #[encoding(fixed(5, "i32"))]
    pub x: f64,
    #[encoding(fixed(5, "i32"))]
    pub y: f64,
    #[encoding(fixed(5, "i32"))]
    pub z: f64,
}

#[derive(ToStatic, Debug)]
pub struct OpenWindow0<'a> {
    pub window_id: u8,
    pub kind: InventoryKind0,
    pub title: Cow<'a, str>,
    pub slot_count: u8,
    pub use_title: bool,
}

impl<'dec: 'a, 'a> Decode<'dec> for OpenWindow0<'a> {
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

#[derive(ToStatic, Debug)]
pub struct OpenWindow6<'a> {
    pub window_id: u8,
    pub kind: InventoryKind6,
    // chat component? at least starting pv13
    pub title: Cow<'a, str>,
    pub slot_count: u8,
}

impl<'dec: 'a, 'a> Decode<'dec> for OpenWindow6<'a> {
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

// #[derive(Encoding, ToStatic, Debug)]
#[derive(ToStatic, Debug)]
// TODO: very good place for #[separate]
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

// #[derive(Encoding, ToStatic, Debug)]
#[derive(ToStatic, Debug)]
// TODO: very good place for #[separate]
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

#[derive(Encoding, ToStatic, Debug)]
pub struct CloseWindow0 {
    /// This is the id of the window that was closed. 0 for inventory.
    pub window_id: u8,
}

#[derive(Encoding, ToStatic, Debug)]
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
    // TODO: slot data
    // data: Slot
}

#[derive(Encoding, ToStatic, Debug)]
pub struct WindowItems0 {
    /// The id of window which items are being sent for. 0 for player inventory.
    pub window_id: u8,
    // #[encoding(counted = "u16")]
    // TODO: slot data
    // slots: Vec<Slot>
}

#[derive(Encoding, ToStatic, Debug)]
/// see <https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5007#Window_Property>
pub struct WindowProperty0 {
    pub window_id: u8,
    pub property: u16,
    pub value: u16,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct ConfirmTransaction0 {
    pub window_id: u8,
    pub action_number: i16,
    pub accepted: bool,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct UpdateSign0<'a> {
    pub x: i32,
    pub y: i16,
    pub z: i32,
    pub line1: Cow<'a, str>,
    pub line2: Cow<'a, str>,
    pub line3: Cow<'a, str>,
    pub line4: Cow<'a, str>,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct UpdateSign6<'a> {
    pub location: Position6,
    pub line1: Cow<'a, str>,
    pub line2: Cow<'a, str>,
    pub line3: Cow<'a, str>,
    pub line4: Cow<'a, str>,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct Maps0 {
    #[encoding(varint)]
    pub item_damage: i32,
    // TODO: #[encoding(rest)]
    // TODO: impl MapData
    // map_data: MapData<'a>,
}

// TODO: WTF
// https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5007#Maps
// enum MapData<'a> {
//     ColorColumn{
//         start_x: u8,
//         start_y: u8,
//         color: &'a [u8]
//     },
//     MapScale(u8)
// }

#[derive(Encoding, ToStatic, Debug)]
pub struct UpdateBlockEntity0 {
    pub x: i32,
    pub y: i16,
    pub z: i32,
    /// The type of update to perform
    pub action: u8,
    /// varies
    pub data_length: u16,
    // Present if data length > 0. Compressed with gzip. Varies
    // TODO: nbt
    // data: Nbt
}

#[derive(Encoding, ToStatic, Debug)]
pub struct UpdateBlockEntity6 {
    pub location: Position6,
    /// The type of update to perform
    pub action: u8,
    /// varies
    pub data_length: u16,
    // Present if data length > 0. Compressed with gzip. Varies
    // TODO: nbt
    // data: Nbt
}

#[derive(Encoding, ToStatic, Debug)]
pub struct SignEditorOpen0 {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Bitfield, ToStatic)]
pub struct SignEditorOpen6 {
    #[encoding(bits = 26)]
    pub x: i32,
    #[encoding(bits = 26)]
    pub z: i32,
    #[encoding(bits = 12)]
    pub y: i16,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct Statistics0<'a> {
    pub entries: Vec<Statistic0<'a>>,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct Statistic0<'a> {
    pub name: Cow<'a, str>,
    #[encoding(varint)]
    /// The amount to increase by
    pub amount: i32,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct PlayerListItem0<'a> {
    /// Supports chat colouring, limited to 16 characters.
    pub name: Cow<'a, str>,
    /// The client will remove the user from the list if false.
    pub online: bool,
    /// Ping, presumably in ms.
    pub ping: i16,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct PlayerListItem7<'a> {
    /// Supports chat colouring, limited to 16 characters.
    pub name: Cow<'a, str>,
    /// The client will remove the user from the list if false.
    pub online: bool,
    /// Ping, presumably in ms.
    #[encoding(varint)]
    pub ping: i32,
}

#[derive(Encoding, ToStatic, Debug)]
pub enum PlayerListItem17<'a> {
    #[encoding(case = "0")]
    AddPlayers(Vec<PlayerListAddPlayer17<'a>>),
    UpdateGamemode(Vec<PlayerListUpdateGamemode17>),
    UpdateLatency(Vec<PlayerListUpdateLatency17>),
}

#[derive(Encoding, ToStatic, Debug)]
pub struct PlayerListAddPlayer17<'a> {
    pub uuid: Uuid,
    pub name: Cow<'a, str>,
    pub gamemode: GameMode17,
    #[encoding(varint)]
    pub ping: i32,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct PlayerListUpdateGamemode17 {
    pub uuid: Uuid,
    pub gamemode: GameMode17,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct PlayerListUpdateLatency17 {
    pub uuid: Uuid,
    #[encoding(varint)]
    pub ping: i32,
}

#[derive(Encoding, ToStatic, Debug)]
pub enum PlayerListItem19<'a> {
    #[encoding(case = "0")]
    AddPlayers(Vec<PlayerListAddPlayer19<'a>>),
    UpdateGamemode(Vec<PlayerListUpdateGamemode17>),
    UpdateLatency(Vec<PlayerListUpdateLatency17>),
    RemovePlayers(Vec<Uuid>),
}

#[derive(Encoding, ToStatic, Debug)]
pub enum PlayerListItem28<'a> {
    #[encoding(case = "0")]
    AddPlayers(Vec<PlayerListAddPlayer28<'a>>),
    UpdateGamemode(Vec<PlayerListUpdateGamemode17>),
    UpdateLatency(Vec<PlayerListUpdateLatency17>),
    UpdateDisplayName(Vec<PlayerListUpdateDisplayName28>),
    RemovePlayers(Vec<Uuid>),
}

#[derive(Encoding, ToStatic, Debug)]
pub struct PlayerListAddPlayer19<'a> {
    pub uuid: Uuid,
    pub name: Cow<'a, str>,
    pub properties: Vec<PlayerProperty19<'a>>,
    pub gamemode: GameMode17,
    #[encoding(varint)]
    pub ping: i32,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct PlayerListAddPlayer28<'a> {
    pub uuid: Uuid,
    pub name: Cow<'a, str>,
    pub properties: Vec<PlayerProperty19<'a>>,
    pub gamemode: GameMode17,
    // varint
    pub ping: i32,
    // TODO: chat
    pub display_name: Option<Cow<'a, str>>,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct PlayerListUpdateDisplayName28 {
    pub uuid: Uuid,
    pub display_name: Option<Uuid>,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct PlayerProperty19<'a> {
    pub name: Cow<'a, str>,
    pub value: Cow<'a, str>,
    pub signature: Option<Cow<'a, str>>,
}

#[derive(Encoding, ToStatic, Clone, Copy, Debug)]
pub enum GameMode17 {
    Survival = 0,
    Creative,
    Adventure,
}

#[derive(ToStatic, Debug)]
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

#[derive(Encoding, ToStatic, Debug)]
pub struct TabComplete0<'a> {
    /// One eligible command
    pub matches: Vec<Cow<'a, str>>,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct ScoreboardObjective0<'a> {
    pub name: Cow<'a, str>,
    pub value: Cow<'a, str>,
    pub action: ScoreboardAction0,
}
#[derive(Encoding, ToStatic, Debug)]
#[encoding(from = "u8")]
pub enum ScoreboardAction0 {
    #[encoding(case = "0")]
    Create,
    Remove,
    Update,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct ScoreboardObjective12<'a> {
    pub name: Cow<'a, str>,
    pub value: Cow<'a, str>,
    pub action: ScoreboardAction0,
}

#[derive(Encoding, ToStatic, Debug)]
#[encoding(from = "u8")]
pub enum ScoreboardObjectiveAction12<'a> {
    #[encoding(case = "0")]
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

// TODO: check that there aren't any other cases in later supported protocol versions
// support up to pv66
#[derive(Encoding, ToStatic, Debug)]
#[encoding(from = "&str")]
pub enum ScoreboardObjectiveKind12 {
    #[encoding(case = "\"integer\"")]
    Integer,
    #[encoding(case = "\"hearts\"")]
    Hearts,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct UpdateScore0<'a> {
    /// The name of the score to be updated or removed
    pub name: Cow<'a, str>,
    pub action: UpdateScoreAction0<'a>,
}

#[derive(Encoding, ToStatic, Debug)]
#[encoding(from = "u8")]
pub enum UpdateScoreAction0<'a> {
    #[encoding(case = "0")]
    Update {
        /// The name of the objective the score belongs to
        text: Cow<'a, str>,
        /// The score to be displayed next to the entry
        kind: i32,
    },
    Remove,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct UpdateScore7<'a> {
    /// The name of the score to be updated or removed
    pub name: Cow<'a, str>,
    pub action: UpdateScoreAction7<'a>,
}

#[derive(Encoding, ToStatic, Debug)]
#[encoding(from = "u8")]
pub enum UpdateScoreAction7<'a> {
    #[encoding(case = "0")]
    Update {
        /// The name of the objective the score belongs to
        text: Cow<'a, str>,
        /// The score to be displayed next to the entry
        #[encoding(varint)]
        kind: i32,
    },
    Remove,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct UpdateScore21<'a> {
    /// The name of the score to be updated or removed
    pub name: Cow<'a, str>,
    pub action: UpdateScoreAction21<'a>,
}

#[derive(Encoding, ToStatic, Debug)]
#[encoding(from = "u8")]
pub enum UpdateScoreAction21<'a> {
    #[encoding(case = "0")]
    Update {
        /// The name of the objective the score belongs to
        text: Cow<'a, str>,
        /// The score to be displayed next to the entry
        #[encoding(varint)]
        kind: i32,
    },
    Remove {
        /// The name of the objective the score belongs to
        text: Cow<'a, str>,
    },
}

#[derive(Encoding, ToStatic, Debug)]
pub struct DisplayScoreboard0<'a> {
    pub position: ScoreboardPosition,
    pub name: Cow<'a, str>,
}

#[derive(Encoding, ToStatic, Debug)]
#[encoding(from = "u8")]
pub enum ScoreboardPosition {
    List = 0,
    Sidebar,
    BelowName,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct Teams0<'a> {
    pub name: Cow<'a, str>,
    pub action: TeamAction0<'a>,
}

#[derive(Encoding, ToStatic, Debug)]
pub enum TeamAction0<'a> {
    #[encoding(case = "0")]
    Create {
        display_name: Cow<'a, str>,
        prefix: Cow<'a, str>,
        suffix: Cow<'a, str>,
        friendly_fire: TeamFriendlyFire,
        #[encoding(counted = "u16")]
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
        #[encoding(counted = "u16")]
        players: Vec<Cow<'a, str>>,
    },
    RemovePlayers {
        #[encoding(counted = "u16")]
        players: Vec<Cow<'a, str>>,
    },
}

#[derive(Encoding, ToStatic, Debug)]
pub struct Teams7<'a> {
    pub name: Cow<'a, str>,
    pub action: TeamAction7<'a>,
}

#[derive(Encoding, ToStatic, Debug)]
pub enum TeamAction7<'a> {
    #[encoding(case = "0")]
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

#[derive(Encoding, ToStatic, Debug)]
pub struct Teams11<'a> {
    pub name: Cow<'a, str>,
    pub action: TeamAction11<'a>,
}

#[derive(Encoding, ToStatic, Debug)]
pub enum TeamAction11<'a> {
    #[encoding(case = "0")]
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

#[derive(Encoding, ToStatic, Debug)]
#[encoding(from = "u8")]
pub enum TeamFriendlyFire {
    Off = 0,
    On,
    FriendliesVisible = 3,
}

#[derive(Encoding, ToStatic, Debug)]
#[encoding(from = "&str")]
pub enum NameTagVisibility11 {
    #[encoding(case = "\"always\"")]
    Always,
    #[encoding(case = "\"hideForOtherTeams\"")]
    HideForOtherTeams,
    #[encoding(case = "\"hideForOwnTeam\"")]
    HideForOwnTeam,
    #[encoding(case = "\"never\"")]
    Never,
}

#[derive(Encoding, ToStatic, Debug)]
// https://dinnerbone.com/blog/2012/01/13/minecraft-plugin-channels-messaging/
pub struct PluginMessage0<'a> {
    pub channel: Cow<'a, str>,
    #[encoding(counted = "u16")]
    pub data: Cow<'a, [u8]>,
}

#[derive(Encoding, ToStatic, Debug)]
// https://dinnerbone.com/blog/2012/01/13/minecraft-plugin-channels-messaging/
pub struct PluginMessage29<'a> {
    pub channel: Cow<'a, str>,
    pub data: Cow<'a, [u8]>,
}

#[derive(Encoding, ToStatic, Debug)]
// https://dinnerbone.com/blog/2012/01/13/minecraft-plugin-channels-messaging/
pub struct PluginMessage32<'a> {
    pub channel: Cow<'a, str>,
    #[encoding(rest)]
    pub data: Cow<'a, [u8]>,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct Disconnect0<'a> {
    // chatcomponent, at least starting pv13
    pub reason: Cow<'a, str>,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct ServerDifficulty6 {
    pub difficulty: Difficulty0,
}

#[derive(Encoding, ToStatic, Debug)]
#[encoding(from = "i32")]
pub enum CombatEvent7<'a> {
    #[encoding(case = "0")]
    EnterCombat,
    EndCombat {
        #[encoding(varint)]
        duration: i32,
        entity_id: i32,
    },
    EntityDead {
        #[encoding(varint)]
        player_id: i32,
        entity_id: i32,
        message: Cow<'a, str>,
    },
}

#[derive(Encoding, ToStatic, Debug)]
#[encoding(from = "u8")]
pub enum CombatEvent8<'a> {
    #[encoding(case = "0")]
    EnterCombat,
    EndCombat {
        #[encoding(varint)]
        duration: i32,
        entity_id: i32,
    },
    EntityDead {
        #[encoding(varint)]
        player_id: i32,
        entity_id: i32,
        message: Cow<'a, str>,
    },
}

#[derive(Encoding, ToStatic, Debug)]
pub struct Camera9 {
    #[encoding(varint)]
    pub entity_id: i32,
}

#[derive(Encoding, ToStatic, Debug)]
pub enum WorldBorder15 {
    #[encoding(case = "0")]
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
        #[encoding(varint)]
        speed: i32,
    },
    SetCenter {
        x: f64,
        z: f64,
    },
}

#[derive(Encoding, ToStatic, Debug)]
pub enum WorldBorder16 {
    #[encoding(case = "0")]
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
        #[encoding(varint)]
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
        #[encoding(varint)]
        speed: i32,
        /// Resulting coordinates from a portal teleport are limited to +-value. Usually 29999984.
        #[encoding(varint)]
        portal_tp_boundary: i32,
    },
}

#[derive(Encoding, ToStatic, Debug)]
pub enum WorldBorder17 {
    #[encoding(case = "0")]
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
        #[encoding(varint)]
        speed: i32,
    },
    SetCenter {
        x: f64,
        z: f64,
    },
    SetWarningTime {
        #[encoding(varint)]
        warning_time: i32,
    },
    SetWarningBlocks {
        #[encoding(varint)]
        warning_blocks: i32,
    },
    Initialize {
        x: f64,
        z: f64,
        old_radius: f64,
        new_radius: f64,
        #[encoding(varint)]
        speed: i32,
        /// Resulting coordinates from a portal teleport are limited to +-value. Usually 29999984.
        #[encoding(varint)]
        portal_tp_boundary: i32,
        #[encoding(varint)]
        warning_time: i32,
        #[encoding(varint)]
        warning_blocks: i32,
    },
}

#[derive(Encoding, ToStatic, Debug)]
pub enum WorldBorder32 {
    #[encoding(case = "0")]
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
        #[encoding(varint)]
        speed: i64,
    },
    SetCenter {
        x: f64,
        z: f64,
    },
    SetWarningTime {
        #[encoding(varint)]
        warning_time: i32,
    },
    SetWarningBlocks {
        #[encoding(varint)]
        warning_blocks: i32,
    },
    Initialize {
        x: f64,
        z: f64,
        old_radius: f64,
        new_radius: f64,
        #[encoding(varint)]
        speed: i64,
        /// Resulting coordinates from a portal teleport are limited to +-value. Usually 29999984.
        #[encoding(varint)]
        portal_tp_boundary: i32,
        #[encoding(varint)]
        warning_time: i32,
        #[encoding(varint)]
        warning_blocks: i32,
    },
}

#[derive(Encoding, ToStatic, Debug)]
#[encoding(from = "u8")]
pub enum Title18<'a> {
    #[encoding(case = "0")]
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

#[derive(Encoding, ToStatic, Debug)]
pub struct SetCompression27 {
    #[encoding(varint)]
    pub threshold: i32,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct PlayerListHeaderAndFooter28<'a> {
    pub header: Cow<'a, str>,
    pub footer: Cow<'a, str>,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct ResourcePackSend32<'a> {
    pub url: Cow<'a, str>,
    /// A 40 character hexadecimal and lowercase SHA-1 hash of the resource
    /// pack file. (must be lower case in order to work)
    /// If it's not a 40 character hexadecimal string, the client will not use
    /// it for hash verification and likely waste bandwidth — but it will still
    /// treat it as a unique id
    pub hash: Cow<'a, str>,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct SetCoolDown48 {
    /// it's not clear if this field is cooldown
    /// or item id
    ///
    /// if it's cooldown, that's in ticks
    /// item id would be well... the item id
    #[encoding(varint)]
    pub cooldown: i32,
}
