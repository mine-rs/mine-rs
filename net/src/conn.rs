use aes::cipher::{InvalidLength, KeyIvInit};
use futures_lite::io::{AsyncRead, AsyncWrite};
use futures_lite::io::{BufReader, BufWriter};
mod readhalf;
mod writehalf;
pub use readhalf::ReadHalf;
// use writehalf::Compression;
pub use writehalf::WriteHalf;

const INITIAL_BUF_SIZE: usize = 1024;

/// A united connection.
/// After compression and encryption are set, `Connection` should be split into `ReadHalf` and `WriteHalf`.
pub struct Connection<R, W> {
    pub read_half: ReadHalf<R>,
    pub write_half: WriteHalf<W>,
}

impl<R: AsyncRead + Unpin, W: AsyncWrite + Unpin> Connection<R, W> {
    pub fn new(reader: R, writer: W) -> Connection<BufReader<R>, BufWriter<W>> {
        Connection {
            read_half: ReadHalf::new(BufReader::new(reader)),
            write_half: WriteHalf::new(BufWriter::new(writer)),
        }
    }
    pub fn unbuffered(reader: R, writer: W) -> Self {
        Connection {
            read_half: ReadHalf::new(reader),
            write_half: WriteHalf::new(writer),
        }
    }
    pub fn split(self) -> (ReadHalf<R>, WriteHalf<W>) {
        (self.read_half, self.write_half)
    }

    // pub fn set_compression(&mut self, threshold: i32, compression: flate2::Compression) {
    //     self.write_half
    //         .compression
    //         .set_compression(threshold, compression);
    //     self.read_half.compression = Some(Vec::with_capacity(INITIAL_BUF_SIZE));
    // }

    pub fn enable_encryption(
        &mut self,
        read_key: &[u8],
        write_key: &[u8],
    ) -> Result<(), InvalidLength> {
        self.read_half.enable_encryption(read_key)?;
        self.write_half
            .enable_encryption(cfb8::Encryptor::new_from_slices(write_key, write_key)?);
        Ok(())
    }
}

// impl<T: AsyncRead + AsyncWrite + Sized + Unpin>
//     Connection<futures_lite::io::ReadHalf<T>, futures_lite::io::WriteHalf<T>>
// {
//     /// Don't use this if there are alternative split methods available for the
//     /// stream you're using.
//     /// Seriously, don't.
//     pub fn split_io(rw: T) -> Self {
//         let (reader, writer) = futures_lite::io::split(rw);
//         Connection::new(reader, writer)
//     }
// }

// impl<R: AsyncRead + Sized + Unpin, W: AsyncWrite + Sized + Unpin> From<(R, W)>
//     for Connection<R, W>
// {
//     fn from(v: (R, W)) -> Self {
//         Connection::new(v.0, v.1)
//     }
// }
