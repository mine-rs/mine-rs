use crate::netty::types::position::Position6;

use ::miners_encoding::{decode, encode, Decode, Encode};

use std::borrow::Cow;
use uuid::Uuid;

#[derive(Encoding, ToStatic, Debug)]
pub struct KeepAlive0 {
    pub id: i32,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct KeepAlive7 {
    #[encoding(varint)]
    pub id: i32,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct ChatMessage0<'a> {
    // TODO: add ChatMessage json thing
    pub message: Cow<'a, str>,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct UseEntity0 {
    pub target_id: i32,
    pub mouse: i8,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct UseEntity7 {
    #[encoding(varint)]
    pub target_id: i32,
    pub mouse: i8,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct UseEntity33 {
    #[encoding(varint)]
    pub target_id: i32,
    pub kind: UseEntityKind33,
}

#[derive(Encoding, ToStatic, Debug)]
#[encoding(varint)]
pub enum UseEntityKind33 {
    #[encoding(case = "0")]
    Interact,
    Attack,
    InteractAt {
        x: f32,
        y: f32,
        z: f32,
    },
}

#[derive(Encoding, ToStatic, Debug)]
pub struct Player0 {
    pub on_ground: bool,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct PlayerPosition0 {
    pub x: f64,
    pub y: f64,
    /// Used to modify the players bounding box when going up stairs, crouching, etc…
    pub stance: f64,
    pub z: f64,
    pub on_ground: bool,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct PlayerPosition10 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub on_ground: bool,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct PlayerLook0 {
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct PlayerPositionAndLook0 {
    pub x: f64,
    pub y: f64,
    /// Used to modify the players bounding box when going up stairs, crouching, etc…
    pub stance: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct PlayerPositionAndLook10 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
}

/// Notchian clients send a 0 (started digging) when they start digging and a 2 (finished digging) once they think they are finished. If digging is aborted, the client simply send a 1 (Cancel digging).
///
/// Status code 4 (drop item) is a special case. In-game, when you use the Drop Item command (keypress 'q'), a dig packet with a status of 4, and all other values set to 0, is sent from client to server. Status code 3 is similar, but drops the entire stack.
///
/// Status code 5 (shoot arrow / finish eating) is also a special case. The x, y and z fields are all set to 0 like above, with the exception of the face field, which is set to 255.
///
/// The face can be one of six values, representing the face being hit:
/// Value   0   1   2   3   4   5
/// Offset -Y  +Y  -Z  +Z  -X  +X
///
/// In 1.7.3, when a player opens a door with left click the server receives Packet 0xE+start digging and opens the door.
#[derive(ToStatic, Debug)]
pub enum PlayerDigging0 {
    Started {
        x: i32,
        y: u8,
        z: i32,
        face: BlockFace0,
    },
    Cancelled {
        x: i32,
        y: u8,
        z: i32,
        face: BlockFace0,
    },
    Finished {
        x: i32,
        y: u8,
        z: i32,
        face: BlockFace0,
    },
    DropItemStack,
    DropItem,
    FinishRightClick,
}

impl<'dec> Decode<'dec> for PlayerDigging0 {
    fn decode(cursor: &mut std::io::Cursor<&'dec [u8]>) -> Result<Self, decode::Error> {
        let action = u8::decode(cursor)?;
        use PlayerDigging0::*;
        Ok(match action {
            0 => Started {
                x: i32::decode(cursor)?,
                y: u8::decode(cursor)?,
                z: i32::decode(cursor)?,
                face: BlockFace0::decode(cursor)?,
            },
            1 => Cancelled {
                x: i32::decode(cursor)?,
                y: u8::decode(cursor)?,
                z: i32::decode(cursor)?,
                face: BlockFace0::decode(cursor)?,
            },
            2 => Finished {
                x: i32::decode(cursor)?,
                y: u8::decode(cursor)?,
                z: i32::decode(cursor)?,
                face: BlockFace0::decode(cursor)?,
            },
            3 => {
                if !(i32::decode(cursor)? == 0
                    && u8::decode(cursor)? == 0
                    && i32::decode(cursor)? == 0
                    && u8::decode(cursor)? == 0)
                {
                    return Err(decode::Error::InvalidId);
                }
                DropItemStack
            }
            4 => {
                if !(i32::decode(cursor)? == 0
                    && u8::decode(cursor)? == 0
                    && i32::decode(cursor)? == 0
                    && u8::decode(cursor)? == 0)
                {
                    return Err(decode::Error::InvalidId);
                }
                DropItem
            }
            5 => {
                if !(i32::decode(cursor)? == 0
                    && u8::decode(cursor)? == 0
                    && i32::decode(cursor)? == 0
                    && u8::decode(cursor)? == 255)
                {
                    return Err(decode::Error::InvalidId);
                }
                FinishRightClick
            }
            _ => return Err(decode::Error::InvalidId),
        })
    }
}
impl Encode for PlayerDigging0 {
    fn encode(&self, writer: &mut impl ::std::io::Write) -> Result<(), encode::Error> {
        use PlayerDigging0::*;
        let (action, x, y, z, face) = match *self {
            Started { x, y, z, face } => (0, x, y, z, face as u8),
            Cancelled { x, y, z, face } => (1, x, y, z, face as u8),
            Finished { x, y, z, face } => (2, x, y, z, face as u8),
            DropItemStack => (3, 0, 0, 0, 0),
            DropItem => (4, 0, 0, 0, 0),
            FinishRightClick => (5, 0, 0, 0, 255),
        };
        action.encode(writer)?;
        x.encode(writer)?;
        y.encode(writer)?;
        z.encode(writer)?;
        face.encode(writer)?;
        Ok(())
    }
}

#[derive(ToStatic, Debug)]
pub enum PlayerDigging6 {
    Started {
        location: Position6,
        face: BlockFace0,
    },
    Cancelled {
        location: Position6,
        face: BlockFace0,
    },
    Finished {
        location: Position6,
        face: BlockFace0,
    },
    DropItemStack,
    DropItem,
    FinishRightClick,
}
impl<'dec> Decode<'dec> for PlayerDigging6 {
    fn decode(cursor: &mut std::io::Cursor<&'dec [u8]>) -> decode::Result<Self> {
        let action = u8::decode(cursor)?;
        use PlayerDigging6::*;
        Ok(match action {
            0 => Started {
                location: Position6::decode(cursor)?,
                face: BlockFace0::decode(cursor)?,
            },
            1 => Cancelled {
                location: Position6::decode(cursor)?,
                face: BlockFace0::decode(cursor)?,
            },
            2 => Finished {
                location: Position6::decode(cursor)?,
                face: BlockFace0::decode(cursor)?,
            },
            3 => {
                if !(Position6::decode(cursor)?.is_0() && u8::decode(cursor)? == 0) {
                    return Err(decode::Error::InvalidId);
                }
                DropItemStack
            }
            4 => {
                if !(Position6::decode(cursor)?.is_0() && u8::decode(cursor)? == 0) {
                    return Err(decode::Error::InvalidId);
                }
                DropItem
            }
            5 => {
                if !(Position6::decode(cursor)?.is_0() && u8::decode(cursor)? == 255) {
                    return Err(decode::Error::InvalidId);
                }
                FinishRightClick
            }
            _ => return Err(decode::Error::InvalidId),
        })
    }
}
impl Encode for PlayerDigging6 {
    fn encode(&self, writer: &mut impl std::io::Write) -> encode::Result<()> {
        use PlayerDigging6::*;
        let (action, location, face) = match *self {
            Started { location, face } => (0, location, face as u8),
            Cancelled { location, face } => (1, location, face as u8),
            Finished { location, face } => (2, location, face as u8),
            DropItemStack => (3, Position6 { x: 0, y: 0, z: 0 }, 0),
            DropItem => (4, Position6 { x: 0, y: 0, z: 0 }, 0),
            FinishRightClick => (5, Position6 { x: 0, y: 0, z: 0 }, 255),
        };
        action.encode(writer)?;
        location.encode(writer)?;
        face.encode(writer)?;
        Ok(())
    }
}

#[derive(Encoding, ToStatic, Debug)]
#[encoding(from = "u8")]
pub enum DiggingAction0 {
    Started = 0,
    Cancelled,
    Finished,
    DropItemStack,
    DropItem,
    /// Shoot arrow / finish eating
    FinishRightClick,
}

#[derive(Encoding, ToStatic, Clone, Copy, Debug)]
#[encoding(from = "u8")]
pub enum BlockFace0 {
    NegY = 0,
    PosY,
    NegZ,
    PosZ,
    NegX,
    PosX,
}

#[derive(Encoding, ToStatic, Debug)]
// In normal operation (ie placing a block), this packet is sent once, with the values set normally.
//
// This packet has a special case where X, Y, Z, and Direction are all -1. (Note that Y is unsigned so set to 255.) This special packet indicates that the currently held item for the player should have its state updated such as eating food, shooting bows, using buckets, etc.
//
// In a Notchian Beta client, the block or item ID corresponds to whatever the client is currently holding, and the client sends one of these packets any time a right-click is issued on a surface, so no assumptions can be made about the safety of the ID. However, with the implementation of server-side inventory, a Notchian server seems to ignore the item ID, instead operating on server-side inventory information and holding selection. The client has been observed (1.2.5 and 1.3.2) to send both real item IDs and -1 in a single session.
//
// Special note on using buckets: When using buckets, the Notchian client might send two packets: first a normal and then a special case. The first normal packet is sent when you're looking at a block (e.g. the water you want to scoop up). This normal packet does not appear to do anything with a Notchian server. The second, special case packet appears to perform the action - based on current position/orientation and with a distance check - it appears that buckets can only be used within a radius of 6 units.
pub struct PlayerBlockPlacement0 {
    pub x: i32,
    pub y: u8,
    pub z: i32,
    // TODO: WTF
}

#[derive(Encoding, ToStatic, Debug)]
pub struct PlayerBlockPlacement6 {
    pub location: Position6,
    // TODO: WTF (see above)
}

#[derive(Encoding, ToStatic, Debug)]
pub struct HeldItemChange0 {
    /// The slot which the player has selected (0-8)
    pub slot: u16,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct Animation0 {
    pub entity_id: i32,
    animation: super::AnimationId0,
}

/// Sent when the player's arm swings
#[derive(Encoding, ToStatic, Debug)]
pub struct Animation7 {}

#[derive(Encoding, ToStatic, Debug)]
pub struct EntityAction0 {
    pub entity_id: i32,
    pub action: EntityAction,
    /// Horse jump boost. Ranged from 0 -> 100.
    pub jump_boost: i32,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct EntityAction7 {
    #[encoding(varint)]
    pub entity_id: i32,
    pub action: EntityAction,
    /// Horse jump boost. Ranged from 0 -> 100.
    #[encoding(varint)]
    pub jump_boost: i32,
}

#[derive(Encoding, ToStatic, Debug)]
#[encoding(from = "u8")]
pub enum EntityAction {
    Crouch = 0,
    Uncrouch,
    LeaveBed,
    StartSprinting,
    StopSprinting,
    JumpWithHorse,
    OpenHorseInventory,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct SteerVehicle0 {
    pub sideways: f32,
    pub forward: f32,
    pub jump: bool,
    pub unmount: bool,
}
#[derive(Encoding, ToStatic, Debug)]
pub struct SteerVehicle7 {
    pub sideways: f32,
    pub forward: f32,
    pub flags: SteerVehicleFlags7,
}
#[derive(Bitfield, ToStatic, Debug)]
#[encoding(typ = "u8", reverse)]
pub struct SteerVehicleFlags7 {
    #[encoding(bool)]
    pub jump: bool,
    #[encoding(bool)]
    pub unmount: bool,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct CloseWindow0 {
    /// This is the id of the window that was closed. 0 for inventory.
    pub window_id: u8,
}

#[derive(ToStatic, Debug)]
pub struct ClickWindow0 {
    pub window_id: u8,
    pub action: ClickAction0,
    pub action_id: i16,
    // TODO: slot type
    // item: Slot
}
impl<'dec> Decode<'dec> for ClickWindow0 {
    fn decode(cursor: &mut std::io::Cursor<&'dec [u8]>) -> decode::Result<Self> {
        let window_id = u8::decode(cursor)?;
        let slot = i16::decode(cursor)?;
        let button = u8::decode(cursor)?;
        let action_id = i16::decode(cursor)?;
        let mode = u8::decode(cursor)?;

        fn mouse_button(button: u8) -> Result<MouseButton, decode::Error> {
            Ok(match button {
                0 => MouseButton::Left,
                1 => MouseButton::Right,
                _ => return Err(decode::Error::InvalidId),
            })
        }

        use ClickAction0::*;
        let action = match mode {
            0 => Click {
                button: mouse_button(button)?,
                slot,
            },
            1 => ShiftClick {
                button: mouse_button(button)?,
                slot,
            },
            2 => Number {
                number: match button {
                    0 => NumberKey::Key1,
                    1 => NumberKey::Key2,
                    2 => NumberKey::Key3,
                    3 => NumberKey::Key4,
                    4 => NumberKey::Key5,
                    5 => NumberKey::Key6,
                    6 => NumberKey::Key7,
                    7 => NumberKey::Key8,
                    8 => NumberKey::Key9,
                    _ => return Err(decode::Error::InvalidId),
                },
                slot,
            },
            3 => match button {
                2 => MiddleClick { slot },
                _ => return Err(decode::Error::InvalidId),
            },
            4 => Drop(match slot {
                -999 => match button {
                    0 => DropKind::LeftNoOp,
                    1 => DropKind::RightNoOp,
                    _ => return Err(decode::Error::InvalidId),
                },
                _ => match button {
                    0 => DropKind::Q { slot },
                    1 => DropKind::CtrlQ { slot },
                    _ => return Err(decode::Error::InvalidId),
                },
            }),
            5 => {
                let (button, change) = match slot {
                    -999 => match button {
                        0 => (MouseButton::Left, DragChange::Start),
                        4 => (MouseButton::Right, DragChange::Start),
                        2 => (MouseButton::Left, DragChange::End),
                        6 => (MouseButton::Right, DragChange::End),
                        _ => return Err(decode::Error::InvalidId),
                    },
                    _ => match button {
                        1 => (MouseButton::Left, DragChange::Add { slot }),
                        5 => (MouseButton::Right, DragChange::Add { slot }),
                        _ => return Err(decode::Error::InvalidId),
                    },
                };
                Drag { button, change }
            }
            6 => match button {
                0 => DoubleClick { slot },
                _ => return Err(decode::Error::InvalidId),
            },
            _ => return Err(decode::Error::InvalidId),
        };

        Ok(Self {
            window_id,
            action,
            action_id,
            // slot: Decode::read(cursor)?,
        })
    }
}
impl Encode for ClickWindow0 {
    fn encode(&self, writer: &mut impl ::std::io::Write) -> Result<(), encode::Error> {
        self.window_id.encode(writer)?;
        let (mode, button, slot) = match self.action {
            ClickAction0::Click { button, slot } => (0, button as u8, slot),
            ClickAction0::ShiftClick { button, slot } => (1, button as u8, slot),
            ClickAction0::Number { number, slot } => (2, number as u8, slot),
            ClickAction0::MiddleClick { slot } => (3, 2, slot),
            ClickAction0::Drop(kind) => match kind {
                DropKind::Q { slot } => (4, 0, slot),
                DropKind::CtrlQ { slot } => (4, 1, slot),
                DropKind::LeftNoOp => (4, 0, -999),
                DropKind::RightNoOp => (4, 1, -999),
            },
            ClickAction0::Drag { button, change } => match change {
                DragChange::Start => (
                    5,
                    match button {
                        MouseButton::Left => 0,
                        MouseButton::Right => 4,
                    },
                    -999,
                ),
                DragChange::Add { slot } => (
                    5,
                    match button {
                        MouseButton::Left => 1,
                        MouseButton::Right => 5,
                    },
                    slot,
                ),
                DragChange::End => (
                    5,
                    match button {
                        MouseButton::Left => 0,
                        MouseButton::Right => 6,
                    },
                    -999,
                ),
            },
            ClickAction0::DoubleClick { slot } => (6, 0, slot),
        };
        slot.encode(writer)?;
        button.encode(writer)?;
        self.action_id.encode(writer)?;
        mode.encode(writer)?;
        // self.item.write(writer)?;
        Ok(())
    }
}

#[derive(ToStatic, Clone, Copy, Debug)]
pub enum ClickAction0 {
    Click {
        button: MouseButton,
        slot: i16,
    },
    ShiftClick {
        button: MouseButton,
        slot: i16,
    },
    Number {
        number: NumberKey,
        slot: i16,
    },
    MiddleClick {
        slot: i16,
    },
    Drop(DropKind),
    Drag {
        button: MouseButton,
        change: DragChange,
    },
    DoubleClick {
        slot: i16,
    },
}

#[derive(ToStatic, Clone, Copy, Debug)]
pub enum MouseButton {
    Left,
    Right,
}

#[derive(ToStatic, Clone, Copy, Debug)]
pub enum NumberKey {
    Key1 = 0,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
}

#[derive(ToStatic, Clone, Copy, Debug)]
pub enum DropKind {
    Q { slot: i16 },
    CtrlQ { slot: i16 },
    LeftNoOp,
    RightNoOp,
}

#[derive(ToStatic, Clone, Copy, Debug)]
pub enum DragChange {
    Start,
    Add { slot: i16 },
    End,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct ConfirmTransaction0 {
    pub window_id: u8,
    pub action_id: i16,
    pub accepted: bool,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct CreativeInventoryAction0 {
    pub slot: u16,
    // TODO: slot type
    // item: Slot
}

#[derive(Encoding, ToStatic, Debug)]
pub struct EnchantItem0 {
    pub window_id: u8,
    /// The position of the enchantment on the enchantment table window, starting with 0 as the topmost one.
    pub enchantment: u8,
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
    pub text: Cow<'a, str>,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct TabComplete37<'a> {
    pub text: Cow<'a, str>,
    pub targeted_block: Option<Position6>,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct ClientSettings0<'a> {
    pub locale: Cow<'a, str>,
    // TODO: find out exactly when this field changed to the pv6's version of it
    // instead of being an enum there it is plainly the exact view distance in chunks
    pub view_distance: ViewDistance0,
    // TODO: custom chat flags
    // https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5007#Client_Settings
    pub chat_flags: u8,
    /// ????
    pub ___: bool,
    pub difficulty: super::Difficulty0,
    pub show_cape: bool,
}

#[derive(Encoding, ToStatic, Debug)]
#[encoding(from = "u8")]
pub enum ViewDistance0 {
    Far = 0,
    Normal,
    Short,
    Tiny,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct ClientSettings6<'a> {
    pub locale: Cow<'a, str>,
    /// Client-side render distance, in chunks
    pub view_distance: u8,
    // TODO: custom chat flags
    // https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5007#Client_Settings
    pub chat_mode: ChatMode6,
    /// ????
    pub ___: bool,
    pub difficulty: DisplayedSkinParts6,
}

#[derive(Encoding, ToStatic, Debug)]
#[encoding(from = "u8")]
pub enum ChatMode6 {
    Enabled = 0,
    CommandsOnly,
    Hidden,
}

#[derive(Bitfield, ToStatic, Debug)]
#[encoding(typ = "u8", reverse)]
pub struct DisplayedSkinParts6 {
    #[encoding(bool)]
    pub cape: bool,
    #[encoding(bool)]
    pub jacket: bool,
    #[encoding(bool)]
    pub left_sleeve: bool,
    #[encoding(bool)]
    pub right_sleeve: bool,
    #[encoding(bool)]
    pub left_pants: bool,
    #[encoding(bool)]
    pub right_pants: bool,
    #[encoding(bool)]
    pub hat: bool,
}

#[derive(Encoding, ToStatic, Debug)]
#[encoding(from = "u8")]
pub enum ClientStatus0 {
    Respawn = 0,
    RequestStats,
    InventoryAchievement,
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
pub struct Spectate17 {
    pub target_player: Uuid,
}

#[derive(Encoding, ToStatic, Debug)]
pub struct ResourcePackStatus32<'a> {
    pub hash: Cow<'a, str>,
    pub result: ResourcePackStatusResult32,
}

#[derive(Encoding, ToStatic, Debug)]
#[encoding(varint)]
pub enum ResourcePackStatusResult32 {
    SuccessfullyLoaded = 0,
    Declined,
    FailedDownload,
    Accepted,
}
