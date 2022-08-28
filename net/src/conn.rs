use futures::io::{BufReader, BufWriter};
use futures::{AsyncRead, AsyncWrite, AsyncReadExt};

/// A united connection.
/// After compression and encryption are enabled/kept disabled, `Connection` should be split into `ReadHalf` and `WriteHalf`.
pub struct Connection<R, W> {
    reader: BufReader<R>,
    writer: BufWriter<W>,
    threshold: i32,
}

impl<R: AsyncRead, W: AsyncWrite> Connection<R, W> {
    pub fn new(reader: R, writer: W) -> Self {
        Connection {
            reader: BufReader::new(reader),
            writer: BufWriter::new(writer),
            threshold: -1,
        }
    }

    pub fn split(self) -> (ReadHalf<R>, WriteHalf<W>) {
        (
            ReadHalf::<R>::new(self.reader, self.threshold.clone()),
            WriteHalf::<W>::new(self.writer, self.threshold),
        )
    }
}

/// Don't use this if there are alternative split methods available for the stream you're using
impl<T: AsyncRead + AsyncWrite + Sized> From<T> for Connection<futures::io::ReadHalf<T>, futures::io::WriteHalf<T>> {
    fn from(rw: T) -> Self {
        let (reader, writer) = rw.split();
        Connection::new(reader, writer)
    }
}


/// The readable half of a connection returned from `Connection::split()`.
pub struct ReadHalf<R> {
    /// The buffer incoming packets are written to.
    pub buf: Vec<u8>,
    reader: BufReader<R>,
    threshold: i32,
}

impl<R> ReadHalf<R> {
    pub(super) fn new(reader: BufReader<R>, threshold: i32) -> Self {
        Self {
            buf: Vec::new(),
            reader,
            threshold,
        }
    }
}

/// The writable half of a connection returned from `Connection::split()`.
pub struct WriteHalf<W> {
    bufs: (Vec<u8>, Vec<u8>),
    writer: BufWriter<W>,
    threshold: i32,
}

impl<W> WriteHalf<W> {
    pub(super) fn new(writer: BufWriter<W>, threshold: i32) -> Self {
        Self {
            bufs: (
                Vec::new(),
                Vec::new(),
            ),
            writer,
            threshold
        }
    }
}
