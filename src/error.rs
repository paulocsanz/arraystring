//! Contains all of this crate errors

use core::fmt::{self, Display, Formatter};
use core::{char::DecodeUtf16Error, str::EncodeUtf16, str::Utf8Error};

/// Every error possible when using the [`ArrayString`]
///
/// [`ArrayString`]: ./array/trait.ArrayString.html
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Error {
    /// Conversion from byte slice to UTF-8 failed (invalid data or invalid index)
    Utf8,
    /// Conversion from `u16` slice to string failed
    Utf16,
    /// Out of bounds access
    OutOfBounds,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::Utf8 => write!(f, "Utf8"),
            Error::Utf16 => write!(f, "Utf16"),
            Error::OutOfBounds => write!(f, "OutOfBounds"),
        }
    }
}

impl From<Utf8Error> for Error {
    fn from(_: Utf8Error) -> Self {
        Error::Utf8
    }
}

impl From<DecodeUtf16Error> for Error {
    fn from(_: DecodeUtf16Error) -> Self {
        Error::Utf16
    }
}

impl<'a> From<EncodeUtf16<'a>> for Error {
    fn from(_: EncodeUtf16) -> Self {
        Error::Utf16
    }
}

/// Error caused by invalid UTF-8 data
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Utf8;

impl From<Utf8Error> for Utf8 {
    fn from(_: Utf8Error) -> Self {
        Utf8
    }
}

impl Display for Utf8 {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Utf8")
    }
}

impl From<Utf8> for Error {
    #[inline]
    fn from(_: Utf8) -> Self {
        trace!("From Utf8");
        Error::Utf8
    }
}

/// Error caused by invalid UTF-16 data
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Utf16;

impl From<DecodeUtf16Error> for Utf16 {
    fn from(_: DecodeUtf16Error) -> Self {
        Utf16
    }
}

impl<'a> From<EncodeUtf16<'a>> for Utf16 {
    fn from(_: EncodeUtf16) -> Self {
        Utf16
    }
}

impl Display for Utf16 {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Utf16")
    }
}

impl From<Utf16> for Error {
    #[inline]
    fn from(_: Utf16) -> Self {
        trace!("From Utf16");
        Error::Utf16
    }
}

/// Error caused by out of bounds access to `LimitedString`
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct OutOfBounds;

impl Display for OutOfBounds {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "OutOfBounds")
    }
}

impl From<OutOfBounds> for Error {
    #[inline]
    fn from(_: OutOfBounds) -> Self {
        trace!("From OutOfBounds");
        Error::OutOfBounds
    }
}
