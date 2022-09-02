use crate::*;

pub mod handshaking;
pub use handshaking::*;
pub mod login;
pub use login::*;
pub mod play;
pub use play::*;
pub mod status;
pub use status::*;

#[derive(Encoding, ToStatic, Clone, Copy)]
#[bitfield]
pub struct Position {
    #[bits(26)]
    pub x: i32,
    #[bits(26)]
    pub z: i32,
    #[bits(12)]
    pub y: i16,
}

#[test]
fn position() {
    #[allow(clippy::unusual_byte_groupings, clippy::type_complexity)]
    const TESTS: &[([u8; 8], (i32, i16, i32))] = &[
        (
            0b01000110_00000111_01100011_00__010011_11101010_01001011_1000__0011_00111111_u64
                .to_be_bytes(),
            (18357644, 831, 20882616),
        ),
        (
            0b00000000_00000000_00000000_00__000000_00000000_00000000_0000__0000_00000000_u64
                .to_be_bytes(),
            (0, 0, 0),
        ),
        (
            0b11111111_11111111_11111111_11__111111_11111111_11111111_1111__1111_11111111_u64
                .to_be_bytes(),
            (-1, -1, -1),
        ),
    ];

    for (bytes, (x, y, z)) in TESTS.iter().copied() {
        let mut cursor = std::io::Cursor::new(&bytes[..]);
        #[allow(clippy::unwrap_used)]
        let pos = Position::decode(&mut cursor).unwrap();
        assert_eq!(pos.x, x);
        assert_eq!(pos.y, y);
        assert_eq!(pos.z, z);
        let mut out = vec![];
        #[allow(clippy::unwrap_used)]
        pos.encode(&mut out).unwrap();
        assert_eq!(&bytes[..], &out[..]);
    }
}
