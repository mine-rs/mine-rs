use aes::cipher::InvalidLength;
use futures_lite::io::{AsyncRead, AsyncWrite};
use futures_lite::io::{BufReader, BufWriter};
mod readhalf;
mod writehalf;
pub use readhalf::ReadHalf;
use writehalf::Compression;
pub use writehalf::WriteHalf;

const INITIAL_BUF_SIZE: usize = 1024;

/// A united connection.
/// After compression and encryption are set, `Connection` should be split into `ReadHalf` and `WriteHalf`.
pub struct Connection<R, W> {
    pub read_half: ReadHalf<R>,
    pub write_half: WriteHalf<W>,
}

impl<R: AsyncRead + Unpin, W: AsyncWrite + Unpin> Connection<R, W> {
    pub fn new(reader: R, writer: W) -> Self {
        Connection {
            read_half: ReadHalf::new(None, None, BufReader::new(reader)),
            write_half: WriteHalf::new(None, Compression::new(), BufWriter::new(writer)),
        }
    }
}
impl<R, W> Connection<R, W> {
    pub fn split(self) -> (ReadHalf<R>, WriteHalf<W>) {
        (self.read_half, self.write_half)
    }

    pub fn set_compression(&mut self, threshold: i32, compression: flate2::Compression) {
        self.write_half
            .compression
            .set_compression(threshold, compression);
        self.read_half.compression = Some(Vec::with_capacity(INITIAL_BUF_SIZE));
    }

    pub fn enable_encryption(
        &mut self,
        read_key: &[u8],
        write_key: &[u8],
    ) -> Result<(), InvalidLength> {
        self.read_half.enable_encryption(read_key)?;
        self.write_half.enable_encryption(write_key)?;
        Ok(())
    }

    #[cfg(feature = "blocking")]
    /// sets the threshold which determines if to offload packet
    /// encryption and decryption using cfb8/aes128 to the threadpool
    pub fn set_blocking_threshold(&mut self, threshold: Option<u32>) {
        //self.read_half.set_blocking_threshold(threshold);
        self.write_half.set_blocking_threshold(threshold);
    }

    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.read_half.shrink_to(min_capacity);
        self.write_half.shrink_to(min_capacity);
    }
}

impl<T: AsyncRead + AsyncWrite + Sized + Unpin>
    Connection<futures_lite::io::ReadHalf<T>, futures_lite::io::WriteHalf<T>>
{
    /// Don't use this if there are alternative split methods available for the
    /// stream you're using
    ///
    /// # Safety
    ///
    /// This isn't unsafe, you just probably should not be using it as there is
    /// likely a better alternative
    pub unsafe fn split_io(rw: T) -> Self {
        let (reader, writer) = futures_lite::io::split(rw);
        Connection::new(reader, writer)
    }
}

impl<R: AsyncRead + Sized + Unpin, W: AsyncWrite + Sized + Unpin> From<(R, W)>
    for Connection<R, W>
{
    fn from(v: (R, W)) -> Self {
        Connection::new(v.0, v.1)
    }
}

/*
/// The readable half of a connection returned from `Connection::split()`.
pub struct ReadHalf<R> {
    /// The buffer incoming packets are written to.
    pub buf: Vec<u8>,
    reader: BufReader<R>,
    pub(super) threshold: i32,
}

impl<R: AsyncRead> ReadHalf<R> {
    pub(super) fn new(reader: R, threshold: i32) -> Self {
        Self {
            buf: Vec::new(),
            reader: BufReader::new(reader),
            threshold,
        }
    }
}

/// The writable half of a connection returned from `Connection::split()`.
pub struct WriteHalf<W> {
    bufs: (Vec<u8>, Vec<u8>),
    writer: BufWriter<W>,
    pub(super) threshold: i32,
}

impl<W: AsyncWrite + Unpin> WriteHalf<W> {
    pub(super) fn new(writer: W, threshold: i32) -> Self {
        Self {
            bufs: (
                Vec::new(),
                Vec::new(),
            ),
            writer: BufWriter::new(writer),
            threshold
        }
    }

    pub async fn send_raw_packet<'a>(&mut self, packet: RawPacket<'a>) -> Result<()> {
        packet.pack(&mut self.writer, &mut self.bufs, self.threshold).await
    }
}
*/
