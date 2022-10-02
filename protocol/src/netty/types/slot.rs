use crate::*;
use attrs::*;
use miners_nbt::Compound;

#[derive(ToStatic, Clone, Debug, PartialEq)]
pub enum Slot0<'a> {
    Empty,
    Item {
        block_id: i16,
        count: u8,
        damage: i16,
        nbt: Compound<'a>,
    },
}
impl<'dec: 'a, 'a> Decode<'dec> for Slot0<'a> {
    fn decode(cursor: &mut std::io::Cursor<&'dec [u8]>) -> decode::Result<Self> {
        let block_id = i16::decode(cursor)?;
        if block_id == -1 {
            return Ok(Self::Empty);
        }
        Ok(Slot0::Item {
            block_id,
            count: u8::decode(cursor)?,
            damage: i16::decode(cursor)?,
            nbt: Decode::decode(cursor)?,
        })
    }
}
impl<'a> Encode for Slot0<'a> {
    fn encode(&self, writer: &mut impl std::io::Write) -> encode::Result<()> {
        match self {
            Slot0::Empty => i16::encode(&-1, writer),
            Slot0::Item {
                block_id,
                count,
                damage,
                nbt,
            } => {
                debug_assert_ne!(*block_id, -1);
                block_id.encode(writer)?;
                count.encode(writer)?;
                damage.encode(writer)?;
                nbt.encode(writer)
            }
        }
    }
}

#[test]
fn slot0() {
    use miners_nbt::*;
    let test3_bytes = &[
        0x01, 0x16, 0x01, 0x00, 0x00, //               start nbt
        0x09, 0x00, 0x04, 0x65, 0x6e, 0x63, 0x68, //   - List "ench"
        0x0a, //                                         type Compound
        0x00, 0x00, 0x00, 0x02, //                       count 2
        0x02, 0x00, 0x02, 0x69, 0x64, 0x00, 0x00, //     - Short "id"  0
        0x02, 0x00, 0x03, 0x6c, 0x76, 0x6c, 0x00, 0x02, // Short "lvl" 2
        0x00, //                                           End
        0x02, 0x00, 0x02, 0x69, 0x64, 0x00, 0x09, //     - Short "id"  9
        0x02, 0x00, 0x03, 0x6c, 0x76, 0x6c, 0x00, 0x04, // Short "lvl" 4
        0x00, //                                           End
        0x00, //                                         End
    ];
    let tests: &[(&[u8], Slot0)] = &[
        (&[0xff, 0xff], Slot0::Empty),
        (
            &[0x01, 0x16, 0x01, 0x00, 0x00, 0x00],
            Slot0::Item {
                block_id: 278,
                count: 1,
                damage: 0,
                nbt: Compound::default(),
            },
        ),
        (
            test3_bytes,
            Slot0::Item {
                block_id: 278,
                count: 1,
                damage: 0,
                nbt: nbt!({
                    "ench": [
                        { "id": 0u16, "lvl": 2u16 },
                        { "id": 9u16, "lvl": 4u16 }
                    ]
                }),
            },
        ),
    ];

    for (bytes, expected) in tests {
        let mut cursor = std::io::Cursor::new(&bytes[..]);
        #[allow(clippy::unwrap_used)]
        let slot = Slot0::decode(&mut cursor).unwrap();
        assert_eq!(&slot, expected);
        assert_eq!(cursor.position() as usize, bytes.len());
        let mut out = vec![];
        #[allow(clippy::unwrap_used)]
        slot.encode(&mut out).unwrap();
        assert_eq!(&bytes[..], &out[..]);
    }
}

#[derive(ToStatic)]
// tree fiddy
pub enum Slot350<'a> {
    Empty,
    Item {
        id: u16,
        count: u8,
        nbt: Compound<'a>,
    },
}
impl<'dec: 'a, 'a> Decode<'dec> for Slot350<'a> {
    fn decode(cursor: &mut std::io::Cursor<&'dec [u8]>) -> decode::Result<Self> {
        match u16::decode(cursor)? {
            0xffff => Ok(Slot350::Empty),
            id => Ok(Slot350::Item {
                id,
                count: u8::decode(cursor)?,
                nbt: Compound::decode(cursor)?,
            }),
        }
    }
}
impl<'a> Encode for Slot350<'a> {
    fn encode(&self, writer: &mut impl std::io::Write) -> encode::Result<()> {
        match self {
            Slot350::Empty => 0xffffu16.encode(writer),
            Slot350::Item { id, count, nbt } => {
                #[cfg(debug_assertions)]
                if *id == 0xffff {
                    return Err(encode::Error::Custom("invalid item id 0xffff (-1)"));
                }
                id.encode(writer)?;
                count.encode(writer)?;
                nbt.encode(writer)
            }
        }
    }
}

#[derive(Encoding, ToStatic)]
#[from(bool)]
pub enum Slot402<'a> {
    #[case(false)]
    Empty,
    #[case(true)]
    Item {
        #[varint]
        id: i32,
        count: u8,
        nbt: Compound<'a>,
    },
}
