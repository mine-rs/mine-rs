use crate::errors::InvalidEnumId;
use crate::*;

use packets_derive::Protocol;
use std::borrow::Cow;

#[derive(Protocol)]
pub struct KeepAlive0 {
    pub id: i32,
}

#[derive(Protocol)]
pub struct ChatMessage0<'a> {
    // todo! add ChatMessage json thing
    pub message: Cow<'a, str>,
}

#[derive(Protocol)]
pub struct UseEntity0 {
    pub target_id: i32,
    pub mouse: i8,
}

#[derive(Protocol)]
pub struct Player0 {
    pub on_ground: bool,
}

#[derive(Protocol)]
pub struct PlayerPosition0 {
    pub x: f64,
    pub y: f64,
    /// Used to modify the players bounding box when going up stairs, crouching, etc…
    pub stance: f64,
    pub z: f64,
    pub on_ground: bool,
}

#[derive(Protocol)]
pub struct PlayerLook0 {
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
}

#[derive(Protocol)]
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

impl<'read> ProtocolRead<'read> for PlayerDigging0 {
    fn read(buf: &mut std::io::Cursor<&'read [u8]>) -> Result<Self, ReadError> {
        let action = u8::read(buf)?;
        use PlayerDigging0::*;
        Ok(match action {
            0 => Started {
                x: i32::read(buf)?,
                y: u8::read(buf)?,
                z: i32::read(buf)?,
                face: BlockFace0::read(buf)?,
            },
            1 => Cancelled {
                x: i32::read(buf)?,
                y: u8::read(buf)?,
                z: i32::read(buf)?,
                face: BlockFace0::read(buf)?,
            },
            2 => Finished {
                x: i32::read(buf)?,
                y: u8::read(buf)?,
                z: i32::read(buf)?,
                face: BlockFace0::read(buf)?,
            },
            3 => {
                if !(i32::read(buf)? == 0
                    && u8::read(buf)? == 0
                    && i32::read(buf)? == 0
                    && u8::read(buf)? == 0)
                {
                    return Err(ReadError::InvalidEnumId);
                }
                DropItemStack
            }
            4 => {
                if !(i32::read(buf)? == 0
                    && u8::read(buf)? == 0
                    && i32::read(buf)? == 0
                    && u8::read(buf)? == 0)
                {
                    return Err(ReadError::InvalidEnumId);
                }
                DropItem
            }
            5 => {
                if !(i32::read(buf)? == 0
                    && u8::read(buf)? == 0
                    && i32::read(buf)? == 0
                    && u8::read(buf)? == 255)
                {
                    return Err(ReadError::InvalidEnumId);
                }
                FinishRightClick
            }
            _ => return Err(ReadError::InvalidEnumId),
        })
    }
}
impl ProtocolWrite for PlayerDigging0 {
    fn write(self, writer: &mut impl ::std::io::Write) -> Result<(), WriteError> {
        use PlayerDigging0::*;
        let (action, x, y, z, face) = match self {
            Started { x, y, z, face } => (0, x, y, z, face as u8),
            Cancelled { x, y, z, face } => (1, x, y, z, face as u8),
            Finished { x, y, z, face } => (2, x, y, z, face as u8),
            DropItemStack => (3, 0, 0, 0, 0),
            DropItem => (4, 0, 0, 0, 0),
            FinishRightClick => (5, 0, 0, 0, 255),
        };
        action.write(writer)?;
        x.write(writer)?;
        y.write(writer)?;
        z.write(writer)?;
        face.write(writer)?;
        Ok(())
    }
    #[inline(always)]
    fn size_hint() -> usize {
        11
    }
}

#[derive(Protocol)]
#[from(u8)]
pub enum DiggingAction0 {
    Started = 0,
    Cancelled,
    Finished,
    DropItemStack,
    DropItem,
    /// Shoot arrow / finish eating
    FinishRightClick,
}

#[derive(Protocol)]
#[from(u8)]
pub enum BlockFace0 {
    NegY = 0,
    PosY,
    NegZ,
    PosZ,
    NegX,
    PosX,
}

#[derive(Protocol)]
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
    // todo! WTF
}

#[derive(Protocol)]
pub struct HeldItemChange0 {
    /// The slot which the player has selected (0-8)
    pub slot: u16,
}

#[derive(Protocol)]
pub struct Animation0 {
    pub entity_id: i32,
    animation: super::AnimationId0,
}

#[derive(Protocol)]
pub struct EntityAction0 {
    pub entity_id: i32,
    pub action: EntityAction,
    /// Horse jump boost. Ranged from 0 -> 100.
    pub jump_boost: i32,
}

#[derive(Protocol)]
#[from(u8)]
pub enum EntityAction {
    Crouch = 1,
    Uncrouch,
    LeaveBed,
    StartSprinting,
    StopSprinting,
}

#[derive(Protocol)]
pub struct SteerVehicle0 {
    pub sideways: f32,
    pub forward: f32,
    pub jump: bool,
    pub unmount: bool,
}

#[derive(Protocol)]
pub struct CloseWindow0 {
    /// This is the id of the window that was closed. 0 for inventory.
    pub window_id: u8,
}

pub struct ClickWindow0 {
    pub window_id: u8,
    pub action: ClickAction0,
    pub action_id: i16,
    // todo! slot type
    // item: Slot
}
impl<'read> ProtocolRead<'read> for ClickWindow0 {
    fn read(buf: &mut std::io::Cursor<&'read [u8]>) -> Result<Self, ReadError> {
        let window_id = u8::read(buf)?;
        let slot = i16::read(buf)?;
        let button = u8::read(buf)?;
        let action_id = i16::read(buf)?;
        let mode = u8::read(buf)?;

        fn mouse_button(button: u8) -> Result<MouseButton, ReadError> {
            Ok(match button {
                0 => MouseButton::Left,
                1 => MouseButton::Right,
                _ => return Err(ReadError::InvalidEnumId),
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
                    _ => return Err(ReadError::InvalidEnumId),
                },
                slot,
            },
            3 => match button {
                2 => MiddleClick { slot },
                _ => return Err(ReadError::InvalidEnumId),
            },
            4 => Drop(match slot {
                -999 => match button {
                    0 => DropKind::LeftNoOp,
                    1 => DropKind::RightNoOp,
                    _ => return Err(ReadError::InvalidEnumId),
                },
                _ => match button {
                    0 => DropKind::Q { slot },
                    1 => DropKind::CtrlQ { slot },
                    _ => return Err(ReadError::InvalidEnumId),
                },
            }),
            5 => {
                let (button, change) = match slot {
                    -999 => match button {
                        0 => (MouseButton::Left, DragChange::Start),
                        4 => (MouseButton::Right, DragChange::Start),
                        2 => (MouseButton::Left, DragChange::End),
                        6 => (MouseButton::Right, DragChange::End),
                        _ => return Err(ReadError::InvalidEnumId),
                    },
                    _ => match button {
                        1 => (MouseButton::Left, DragChange::Add { slot }),
                        5 => (MouseButton::Right, DragChange::Add { slot }),
                        _ => return Err(ReadError::InvalidEnumId),
                    },
                };
                Drag { button, change }
            }
            6 => match button {
                0 => DoubleClick { slot },
                _ => return Err(ReadError::InvalidEnumId),
            },
            _ => return Err(ReadError::InvalidEnumId),
        };

        Ok(Self {
            window_id,
            action,
            action_id,
            // slot: ProtocolRead::read(buf)?,
        })
    }
}
impl ProtocolWrite for ClickWindow0 {
    fn write(self, writer: &mut impl ::std::io::Write) -> Result<(), WriteError> {
        self.window_id.write(writer)?;
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
        slot.write(writer)?;
        button.write(writer)?;
        self.action_id.write(writer)?;
        mode.write(writer)?;
        // self.item.write(writer)?;
        Ok(())
    }
    #[inline(always)]
    fn size_hint() -> usize {
        7
        // + <Slot as ProtocolWrite>::size_hint()
    }
}

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

pub enum MouseButton {
    Left,
    Right,
}

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
pub enum DropKind {
    Q { slot: i16 },
    CtrlQ { slot: i16 },
    LeftNoOp,
    RightNoOp,
}

pub enum DragChange {
    Start,
    Add { slot: i16 },
    End,
}

#[derive(Protocol)]
pub struct ConfirmTransaction0 {
    pub window_id: u8,
    pub action_id: i16,
    pub accepted: bool,
}

#[derive(Protocol)]
pub struct CreativeInventoryAction0 {
    pub slot: u16,
    // todo! slot type
    // item: Slot
}

#[derive(Protocol)]
pub struct EnchantItem0 {
    pub window_id: u8,
    /// The position of the enchantment on the enchantment table window, starting with 0 as the topmost one.
    pub enchantment: u8,
}

#[derive(Protocol)]
pub struct UpdateSign0<'a> {
    pub x: i32,
    pub y: i16,
    pub z: i32,
    pub line1: Cow<'a, str>,
    pub line2: Cow<'a, str>,
    pub line3: Cow<'a, str>,
    pub line4: Cow<'a, str>,
}

pub use super::PlayerAbilities0;

#[derive(Protocol)]
pub struct TabComplete0<'a> {
    pub text: Cow<'a, str>,
}

#[derive(Protocol)]
pub struct ClientSettings0<'a> {
    pub locale: Cow<'a, str>,
    pub view_distance: ViewDistance0,
    // todo! custom chat flags
    // https://wiki.vg/index.php?title=Pre-release_protocol&oldid=5007#Client_Settings
    pub chat_flags: u8,
    /// ????
    ___: bool,
    difficulty: super::Difficulty0,
    pub show_cape: bool,
}
#[derive(Protocol)]
#[from(u8)]
pub enum ViewDistance0 {
    Far = 0,
    Normal,
    Short,
    Tiny,
}

#[derive(Protocol)]
#[from(u8)]
pub enum ClientStatus0 {
    Respawn = 0,
    RequestStats,
    InventoryAchievement,
}

pub use super::PluginMessage0;
