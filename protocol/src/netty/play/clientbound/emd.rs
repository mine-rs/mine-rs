// TODO: Implement a new BtreeMap type that uses a u8 for the key: 5 bits for the index and 3 bits for the type and unions for the values
// this is definitely not necessary and should be pretty low on the todo list
use std::collections::BTreeMap;
use std::io::Read;

use miners_encoding::attrs::Var;
use uuid::Uuid;

use crate::{
    decode::{self, Decode},
    encode::{self, Encode},
    netty::Position,
};

pub enum Value0 {
    Byte(i8),
    Short(i16),
    Int(i32),
    Float(f32),
    String(String),
    Slot(), //TODO: Add an actual slot data type
    Position((i32, i32, i32)),
    Look((f32, f32, f32)),
}

impl Encode for Value0 {
    fn encode(&self, writer: &mut impl std::io::Write) -> encode::Result<()> {
        match self {
            Self::Byte(v) => v.encode(writer),
            Self::Short(v) => v.encode(writer),
            Self::Int(v) => v.encode(writer),
            Self::Float(v) => v.encode(writer),
            Self::String(v) => v.encode(writer),
            Self::Slot() => {
                todo!()
            }
            Self::Position(v) => {
                v.0.encode(writer)?;
                v.1.encode(writer)?;
                v.2.encode(writer)
            }
            Self::Look(v) => {
                v.0.encode(writer)?;
                v.1.encode(writer)?;
                v.2.encode(writer)
            }
        }
    }
}

impl Value0 {
    pub fn id(&self) -> u8 {
        match self {
            Self::Byte(_) => 0,
            Self::Short(_) => 1,
            Self::Int(_) => 2,
            Self::Float(_) => 3,
            Self::String(_) => 4,
            Self::Slot() => 5,
            Self::Position(_) => 6,
            Self::Look(_) => 7,
        }
    }
}

pub struct EntityMetadata0(pub BTreeMap<u8, Value0>);

impl Encode for EntityMetadata0 {
    fn encode(&self, writer: &mut impl std::io::Write) -> encode::Result<()> {
        for (k, v) in self.0.iter() {
            //shouldn't be necessary
            //let k = k & 0b00011111; // ignore the top 3 bits.

            let id = v.id() << 5;
            let key = k & id;
            writer.write_all(&[key])?;
            v.encode(writer)?;
        }
        writer.write_all(&[127])?;
        Ok(())
    }
}

impl<'dec> Decode<'dec> for EntityMetadata0 {
    fn decode(cursor: &mut std::io::Cursor<&'dec [u8]>) -> decode::Result<Self> {
        let mut map = BTreeMap::<u8, Value0>::new();
        let mut buf = [0u8];
        loop {
            cursor.read_exact(&mut buf)?;
            let i = buf[0];
            if i == 127 {
                break;
            }
            let key = i & 0x1F;
            let r#type = i & 0xE0;
            let value = match r#type {
                0 => Value0::Byte(i8::decode(cursor)?),
                1 => Value0::Short(i16::decode(cursor)?),
                2 => Value0::Int(i32::decode(cursor)?),
                3 => Value0::Float(f32::decode(cursor)?),
                4 => Value0::String(String::decode(cursor)?),
                5 => {
                    todo!()
                }
                6 => {
                    let x = i32::decode(cursor)?;
                    let y = i32::decode(cursor)?;
                    let z = i32::decode(cursor)?;
                    Value0::Position((x, y, z))
                }
                7 => {
                    let pitch = f32::decode(cursor)?;
                    let yaw = f32::decode(cursor)?;
                    let roll = f32::decode(cursor)?;
                    Value0::Look((pitch, yaw, roll))
                }
                _ => {
                    // unreachable
                    loop {}
                }
            };
            map.insert(key, value);
        }
        Ok(Self(map))
    }
}

// TODO: Rename this to the first snapshot that this format appeared
pub enum Value107 {
    Byte(i8),
    VarInt(Var<i32>),
    Float(f32),
    String(String),
    Chat(String), //TODO: Replace with actual chat type
    Slot(),       //TODO: Add an actual slot data type
    Boolean(bool),
    Look((f32, f32, f32)),
    Position(Position),
    OptPosition(Option<Position>),
    //TODO: This is probably an enum but 1.9 wiki.vg just says "???"
    Direction(Var<i32>),
    OptUuid(Option<Uuid>),
    BlockId(Var<i32>),
}

impl Value107 {
    pub fn id(&self) -> u8 {
        match self {
            Self::Byte(_) => 0,
            Self::VarInt(_) => 1,
            Self::Float(_) => 2,
            Self::String(_) => 3,
            Self::Chat(_) => 4,
            Self::Slot() => 5,
            Self::Boolean(_) => 6,
            Self::Look(_) => 7,
            Self::Position(_) => 8,
            Self::OptPosition(_) => 9,
            Self::Direction(_) => 10,
            Self::OptUuid(_) => 11,
            Self::BlockId(_) => 12,
        }
    }
}

impl Encode for Value107 {
    fn encode(&self, writer: &mut impl std::io::Write) -> encode::Result<()> {
        match self {
            Self::Byte(v) => {
                v.encode(writer)?;
            }
            Self::VarInt(v) => {
                v.encode(writer)?;
            }
            Self::Float(v) => {
                v.encode(writer)?;
            }
            Self::String(v) => {
                v.encode(writer)?;
            }
            Self::Chat(v) => {
                v.encode(writer)?;
            }
            Self::Slot() => {
                todo!()
            }
            Self::Boolean(v) => {
                v.encode(writer)?;
            }
            Self::Look(v) => {
                v.0.encode(writer)?;
                v.1.encode(writer)?;
                v.2.encode(writer)?;
            }
            Self::Position(v) => {
                v.encode(writer)?;
            }
            Self::OptPosition(v) => match v {
                None => {
                    false.encode(writer)?;
                }
                Some(v) => {
                    true.encode(writer)?;
                    v.encode(writer)?;
                }
            },
            Self::Direction(v) => {
                v.encode(writer)?;
            }
            Self::OptUuid(v) => match v {
                None => {
                    false.encode(writer)?;
                }
                Some(v) => {
                    true.encode(writer)?;
                    v.encode(writer)?;
                }
            },
            Self::BlockId(v) => {
                v.encode(writer)?;
            }
        }
        Ok(())
    }
}

pub struct EntityMetadata107(pub BTreeMap<u8, Value107>);

impl Encode for EntityMetadata107 {
    fn encode(&self, writer: &mut impl std::io::Write) -> encode::Result<()> {
        for (index, value) in self.0.iter() {
            index.encode(writer)?;
            value.id().encode(writer)?;
            value.encode(writer)?;
        }
        0xff.encode(writer)?;
        Ok(())
    }
}

impl<'dec> Decode<'dec> for EntityMetadata107 {
    fn decode(cursor: &mut std::io::Cursor<&'dec [u8]>) -> decode::Result<Self> {
        let mut map = BTreeMap::<u8, Value107>::new();
        let mut buf = [0u8];
        loop {
            cursor.read_exact(&mut buf)?;
            if buf[0] == 0xff {
                break;
            }
            let index = buf[0];
            cursor.read_exact(&mut buf)?;
            let r#type = buf[0];
            let value = match r#type {
                0 => Value107::Byte(i8::decode(cursor)?),
                1 => Value107::VarInt(Var::<i32>::decode(cursor)?),
                2 => Value107::Float(f32::decode(cursor)?),
                3 => Value107::String(String::decode(cursor)?),
                4 => Value107::Chat(String::decode(cursor)?),
                5 => {
                    todo!()
                }
                6 => Value107::Boolean(bool::decode(cursor)?),
                7 => {
                    let pitch = f32::decode(cursor)?;
                    let yaw = f32::decode(cursor)?;
                    let roll = f32::decode(cursor)?;
                    Value107::Look((pitch, yaw, roll))
                }
                8 => Value107::Position(Position::decode(cursor)?),
                9 => {
                    let present = bool::decode(cursor)?;
                    Value107::OptPosition(match present {
                        false => None,
                        true => Some(Position::decode(cursor)?),
                    })
                }
                10 => Value107::Direction(Var::<i32>::decode(cursor)?),
                11 => {
                    let present = bool::decode(cursor)?;
                    Value107::OptUuid(match present {
                        false => None,
                        true => Some(Uuid::decode(cursor)?),
                    })
                }
                12 => Value107::BlockId(Var::<i32>::decode(cursor)?),
                _ => {
                    // unreachable
                    loop {}
                }
            };
            map.insert(index, value);
        }
        Ok(Self(map))
    }
}
