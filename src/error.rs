//! Contains all of this crate errors

use core::{fmt, fmt::Display, fmt::Formatter, str::EncodeUtf16, str::Utf8Error};

/// Every error possible when using the [`ArrayString`]
///
/// [`ArrayString`]: ./array/trait.ArrayString.html
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Error {
    /// Conversion from byte slice to UTF-8 failed (invalid data or invalid index)
    FromUtf8,
    /// Conversion from `u16` slice to string failed
    FromUtf16,
    /// Out of bounds access
    OutOfBounds,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::FromUtf8 => write!(f, "FromUtf8"),
            Error::FromUtf16 => write!(f, "FromUtf16"),
            Error::OutOfBounds => write!(f, "OutOfBounds"),
        }
    }
}

impl From<Utf8Error> for Error {
    fn from(_: Utf8Error) -> Self {
        Error::FromUtf8
    }
}

impl<'a> From<EncodeUtf16<'a>> for Error {
    fn from(_: EncodeUtf16) -> Self {
        Error::FromUtf16
    }
}

/// Error caused by invalid UTF-8 data
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct FromUtf8;

impl From<Utf8Error> for FromUtf8 {
    fn from(_: Utf8Error) -> Self {
        FromUtf8
    }
}

impl Display for FromUtf8 {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "FromUtf8")
    }
}

impl From<FromUtf8> for Error {
    #[inline]
    fn from(_: FromUtf8) -> Self {
        trace!("From FromUtf8");
        Error::FromUtf8
    }
}

/// Error caused by invalid UTF-16 data
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct FromUtf16;

impl<'a> From<EncodeUtf16<'a>> for FromUtf16 {
    fn from(_: EncodeUtf16) -> Self {
        FromUtf16
    }
}

impl Display for FromUtf16 {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "FromUtf16")
    }
}

impl From<FromUtf16> for Error {
    #[inline]
    fn from(_: FromUtf16) -> Self {
        trace!("From FromUtf16");
        Error::FromUtf16
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
