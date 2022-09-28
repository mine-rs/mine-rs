use std::io::Read;
use std::collections::BTreeMap;

use crate::{encode::{self, Encode}, decode::{self, Decode}};

pub enum Value {
    Byte(i8),
    Short(i16),
    Int(i32),
    Float(f32),
    String(String),
    Slot(), //TODO: Add an actual slot data type
    Position((i32, i32, i32)),
    Look((f32, f32, f32))
}

impl Encode for Value {
    fn encode(&self, writer: &mut impl std::io::Write) -> encode::Result<()> {
        match self {
            Self::Byte(v) => {
                v.encode(writer)
            },
            Self::Short(v) => {
                v.encode(writer)
            },
            Self::Int(v) => {
                v.encode(writer)
            },
            Self::Float(v) => {
                v.encode(writer)
            },
            Self::String(v) => {
                v.encode(writer)
            },
            Self::Slot() => {
                todo!()
            },
            Self::Position(v) => {
                v.0.encode(writer)?;
                v.1.encode(writer)?;
                v.2.encode(writer)
            },
            Self::Look(v) =>{
                v.0.encode(writer)?;
                v.1.encode(writer)?;
                v.2.encode(writer)
            }
        }
    }
}

impl Value {
    pub fn id(&self) -> u8 {
        match self {
            Self::Byte(_) => 0,
            Self::Short(_) => 1,
            Self::Int(_) => 2,
            Self::Float(_) => 3,
            Self::String(_) => 4,
            Self::Slot() => 5,
            Self::Position(_) => 6,
            Self::Look(_) => 7
        }
    }
}

pub struct EntityMetadata0(pub BTreeMap<u8, Value>);

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
        let mut map = BTreeMap::<u8, Value>::new();
        let mut buf = [0u8];
        loop {
            cursor.read_exact(&mut buf)?;
            let i = buf[0];
            if i == 127 {
                break
            }
            let key = i & 0x1F;
            let r#type = i & 0xE0;
            let value = match r#type {
                0 => {
                    Value::Byte(i8::decode(cursor)?)
                },
                1 => {
                    Value::Short(i16::decode(cursor)?)
                },
                2 => {
                    Value::Int(i32::decode(cursor)?)
                },
                3 => {
                    Value::Float(f32::decode(cursor)?)
                },
                4 => {
                    Value::String(String::decode(cursor)?)
                },
                5 => {
                    todo!()
                },
                6 => {
                    let x = i32::decode(cursor)?;
                    let y = i32::decode(cursor)?;
                    let z = i32::decode(cursor)?;
                    Value::Position((x, y, z))
                },
                7 => {
                    let pitch = f32::decode(cursor)?;
                    let yaw = f32::decode(cursor)?;
                    let roll = f32::decode(cursor)?;
                    Value::Look((pitch, yaw, roll))
                },
                _ => {
                    // unreachable
                    loop {}
                }
            };
            map.insert(key, value);
        };
        Ok(Self(map))
    }
}
