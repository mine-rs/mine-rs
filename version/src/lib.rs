use std::{fmt::Display, ops::Deref};

const SNAPSHOT: i32 = 0x40000000;

#[derive(thiserror::Error, Debug)]
#[error("invalid protocol version {0}")]
pub struct InvalidVersion(i32);
impl InvalidVersion {
    /// Returns the inner (invalid) version.
    pub fn inner(&self) -> i32 {
        self.0
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct ProtocolVersion(i32);

impl Display for ProtocolVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Deref for ProtocolVersion {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<i32> for ProtocolVersion {
    type Error = InvalidVersion;

    fn try_from(version: i32) -> Result<Self, InvalidVersion> {
        ProtocolVersion::new(version)
    }
}

impl From<ProtocolVersion> for i32 {
    fn from(val: ProtocolVersion) -> Self {
        *val
    }
}

impl ProtocolVersion {
    /// Constructs a new `ProtocolVersion` if the provided version is valid.
    /// # Errors
    /// If the supplied version is invalid, an `InvalidVersion` error is returned.
    pub fn new(version: i32) -> Result<Self, InvalidVersion> {
        if ((0..=760).contains(&version)
            && !((111..201).contains(&version))
            && !((211..301).contains(&version))
            && !((405..441).contains(&version))
            && version != 499
            && !((501..550).contains(&version))
            && !((578..701).contains(&version))
            && version != 720
            && !((723..725).contains(&version))
            && version != 726
            && version != 728
            && version != 731
            && version != 737
            && version != 739
            && version != 742
            && version != 745
            && version != 747)
            || ((SNAPSHOT & 1..=SNAPSHOT & 101).contains(&version)
                && !((SNAPSHOT & 71..SNAPSHOT & 704).contains(&version))
                && version != SNAPSHOT & 24
                && version != SNAPSHOT & 4)
        {
            Ok(Self(version))
        } else {
            Err(InvalidVersion(0))
        }
    }

    /// Constructs a new `ProtocolVersion` without checking if the provided version is valid
    /// # Safety
    /// This method is only safe if the supplied version is a valid version that has been implemented by mine-rs
    pub unsafe fn new_unchecked(version: i32) -> Self {
        Self(version)
    }
}
