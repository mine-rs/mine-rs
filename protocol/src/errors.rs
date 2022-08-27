use std::io::Error as IoError;
use std::num::TryFromIntError as InvalidCount;
use std::{str::Utf8Error, string::FromUtf8Error};
use uuid::Error as UuidError;

#[derive(Debug)]
#[non_exhaustive]
pub enum WriteError {
    IoError(std::io::Error),
    StringTooLong,
    InvalidCount,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum ReadError {
    IoError(std::io::Error),
    InvalidEnumId,
    Utf8Error(Utf8Error),
    FromUtf8Error(FromUtf8Error),
    UuidError(uuid::Error),
    ReadPastEnd,
    InvalidProtocolVersionIdCombination,
}

pub struct InvalidEnumId;

macro_rules! from {
    ($RW:ident: [$($Err:ident),*]) => {$(
        impl From<$Err> for $RW {
            fn from(e: $Err) -> Self {
                Self::$Err(e)
            }
        }
    )*};
}
macro_rules! from_unit {
    ($RW:ident: [$($Err:ident),*]) => {$(
        impl From<$Err> for $RW {
            fn from(_: $Err) -> Self {
                Self::$Err
            }
        }
    )*};
}

from!(WriteError: [IoError]);
from_unit!(WriteError: [InvalidCount]);

from!(ReadError: [IoError, Utf8Error, FromUtf8Error, UuidError]);
from_unit!(ReadError: [InvalidEnumId]);
