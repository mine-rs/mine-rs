use std::fs;
use std::{
    io::{Cursor, Read, Seek, SeekFrom},
    path,
};

/// A struct for handling region files.
pub struct RegionFile49 {
    header: Box<RegionHeader49>,
    file: fs::File,
    #[allow(dead_code)]
    path: path::PathBuf,
}

impl RegionFile49 {
    pub fn save(&mut self, _region: [Option<&[u8]>; 1024]) -> std::io::Result<()> {
        todo!()
        /*
        let mut size: usize = 0;
        for (i, chunk) in region.iter().enumerate() {
            let location = self.header.locations[i];
            // I would prefer location to be Option but I need the memory layout to be guaranteed.
            if location.0 == 0 {
                return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "chunk was not found"))
            }
            size += match chunk {
                None => {
                    todo!()
                }
                Some(v) => v.len(),
            }
        }
        let mut buf = vec![0; size];
        let mut pos: usize = 0;
        for (i, chunk) in region.iter().enumerate() {
            pos += match chunk {
                Some(v) => {
                    //TODO: Rewrite this so it doesn't need a cursor?
                    let mut cursor = Cursor::new(*v);
                    cursor.read_exact(&mut buf.as_mut_slice()[pos..pos+size])?;
                    v.len()
                }
                None => {
                    let location = self.header.locations[i];
                    // I would prefer location to be Option but I need the memory layout to be guaranteed.
                    if location.0 == 0 {
                        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "chunk was not found"))
                    }
                    let len = todo!();
                    self.file.seek(SeekFrom::Start((location.offset() * 8192) as u64))?;
                    self.file.read_exact(&mut buf[pos..pos+len])?;
                    len
                },
            };
        }
        self.file = fs::File::create(&self.path)?;
        self.file.write_all(&buf)?;

        Ok(())
        */
    }

    pub fn open(path: path::PathBuf) -> std::io::Result<Self> {
        let mut file = fs::File::open(&path)?;
        let mut buf = Box::new([0; 8192]);
        file.read_exact(buf.as_mut_slice())?;
        debug_assert_eq!(std::mem::size_of::<RegionHeader49>(), 8192);
        // Safety: This is safe because `RegionHeader49` has the same size as `[u8; 8192]`
        let header = unsafe { std::mem::transmute(buf) };
        Ok(Self { header, file, path })
    }

    pub fn chunk<'a>(&'a mut self, x: i32, z: i32) -> std::io::Result<Vec<u8>> {
        let location = if let Some(v) = self.header.location(x, z) {
            v
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "chunk was not found",
            ));
        };

        self.file
            .seek(SeekFrom::Start((location.offset() * 8192) as u64))?;

        let len = {
            let mut bytes = [0; 4];
            self.file.read_exact(&mut bytes)?;
            u32::from_be_bytes(bytes) as usize - 1
        };
        let compression_type = {
            let mut bytes = [0; 1];
            self.file.read_exact(&mut bytes)?;
            bytes[0]
        };

        let mut compressed = vec![0; len];
        self.file.read_exact(&mut compressed)?;

        let mut cursor = Cursor::new(compressed);
        let mut buf = Vec::new();

        match compression_type {
            1 => {
                let mut decoder = flate2::read::GzDecoder::new(&mut cursor);
                decoder.read_to_end(&mut buf)?;
                Ok(buf)
            }
            2 => {
                let mut decoder = flate2::read::ZlibDecoder::new(&mut cursor);
                decoder.read_to_end(&mut buf)?;
                Ok(buf)
            }
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid compression type ({compression_type}), must be either 1 or 2"),
            )),
        }
    }

    pub fn timestamp(&self, x: i32, z: i32) -> Option<i32> {
        self.header.timestamp(x, z)
    }
}

/*
struct RegionFile(Vec<u8>);

impl RegionFile {
    pub fn from_reader<T: Read + Seek>(rdr: &mut T) -> std::io::Result<Self>{
        let mut buf =
        {
            const CHUNK_DATA_LEN_POS: usize = 8192;
            let start = rdr.stream_position()?;
            rdr.seek(SeekFrom::Current(CHUNK_DATA_LEN_POS as i64));
            let mut len_buf = [0u8; 4];
            rdr.read_exact(&mut len_buf);
            let size = u32::from_be_bytes(len_buf) as usize;
            rdr.seek(SeekFrom::Start(start));
            vec![0u8; size + CHUNK_DATA_LEN_POS]
        };
        rdr.read_exact(&mut buf)?;

        Ok(Self(buf))
    }
}*/

#[repr(C)]
struct RegionHeader49 {
    locations: [Location49; 1024],
    timestamps: [i32; 1024],
}

impl RegionHeader49 {
    fn index(x: i32, z: i32) -> usize {
        ((x & 0x1f) + (z & 0x1f)) as usize
    }

    pub fn location(&self, x: i32, z: i32) -> Option<Location49> {
        let location = self.locations[Self::index(x, z)];
        if location.0 == 0 {
            None
        } else {
            Some(location)
        }
    }

    pub fn timestamp(&self, x: i32, z: i32) -> Option<i32> {
        let timestamp = self.timestamps[Self::index(x, z)];
        if timestamp == 0 {
            None
        } else {
            Some(timestamp)
        }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy)]
struct Location49(u32);

impl Location49 {
    /// Returns the offset.
    pub fn offset(self) -> usize {
        let bytes = self.0.to_ne_bytes();
        let bytes = [0, bytes[0], bytes[1], bytes[2]];
        u32::from_be_bytes(bytes) as usize
    }

    /// Returns the section count.
    pub fn _count(self) -> u8 {
        self.0.to_ne_bytes()[3]
    }
}
