#![deny(clippy::undocumented_unsafe_blocks)]
pub mod conn;
pub mod encoding;
pub mod packing;
#[cfg(feature = "workpool")]
pub(crate) mod workpool;
pub mod writer;

#[cfg(feature = "workpool")]
const DEFAULT_UNBLOCK_THRESHOLD: u32 = 4096;

pub(crate) mod helpers {

    pub(crate) fn encrypt(data: &mut [u8], encryptor: &mut cfb8::Encryptor<aes::Aes128>) {
        let (chunks, rest) = aes::cipher::inout::InOutBuf::from(data).into_chunks();
        debug_assert!(rest.is_empty());
        aes::cipher::BlockEncryptMut::encrypt_blocks_inout_mut(encryptor, chunks);
    }

    pub(crate) fn varint_slice(mut num: u32, buf: &mut [u8; 5]) -> &mut [u8] {
        for i in 0..5 {
            let next_val = num >> 7;
            if next_val == 0 {
                buf[i] = num as u8;
                return &mut buf[..i + 1];
            }
            buf[i] = num as u8 | 0x80;
            num = next_val;
        }
        &mut buf[..]
    }

    pub(crate) fn varint_vec(mut num: u32, vec: &mut Vec<u8>) {
        for _ in 0..5 {
            let next_val = num >> 7;
            if next_val == 0 {
                vec.push(num as u8);
                break;
            }
            vec.push(num as u8 | 0x80);
            num = next_val;
        }
    }

    #[cfg(test)]
    mod varint {
        use super::*;
        const TESTS: &[(i32, &[u8])] = &[
            (0, &[0x00]),
            (1, &[0x01]),
            (127, &[0x7f]),
            (128, &[0x80, 1]),
            (255, &[0xff, 0x01]),
            (25565, &[0xdd, 0xc7, 0x01]),
            (2097151, &[0xff, 0xff, 0x7f]),
            (2147483647, &[0xff, 0xff, 0xff, 0xff, 0x07]),
            (-1, &[0xff, 0xff, 0xff, 0xff, 0x0f]),
            (-2147483648, &[0x80, 0x80, 0x80, 0x80, 0x08]),
        ];
        #[test]
        fn write() {
            for (num, res) in TESTS {
                let mut buf = [0u8; 5];
                let varbuf = varint_slice(*num as u32, &mut buf);
                assert_eq!(*res, varbuf)
            }
        }
    }

    #[derive(Debug)]
    pub(crate) struct AsyncCancelled;
    impl std::fmt::Display for AsyncCancelled {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("async cancelled")
        }
    }
    impl std::error::Error for AsyncCancelled {}
    impl From<AsyncCancelled> for std::io::Error {
        fn from(_: AsyncCancelled) -> Self {
            std::io::Error::new(std::io::ErrorKind::Other, AsyncCancelled)
        }
    }
}
