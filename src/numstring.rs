use crate::error::Utf8;
use crate::prelude::OutOfBounds;
use crate::Error;
#[cfg(feature = "logs")]
use log::{debug, trace};
#[cfg(all(feature = "no-panic", not(debug_assertions)))]
use no_panic::no_panic;
use std::borrow::Borrow;
use std::fmt::{Debug, Formatter};
use std::mem;
use std::mem::size_of;
use std::ops::*;
use std::str::from_utf8_unchecked;

macro_rules! make_fn_array {
    ($f:ident; $($e:tt);* $(;)*) => {
        [$($f($e)),*]
    };
    ($f:ident; $($e:tt, $e2:tt);* $(;)*) => {
        [$($f($e, $e2)),*]
    };
}

#[derive(Copy, Clone, Eq, PartialEq, Default)]
pub struct NumString(usize);

#[inline]
const fn from_bytes(bytes: &[u8], to: usize) -> usize {
    #[cfg(target_pointer_width = "64")]
    let bytes = match bytes {
        [a, b, c, d, e, f, g, ..] => usize::from_ne_bytes([*a, *b, *c, *d, *e, *f, *g, 0]),
        [a, b, c, d, e, f] => usize::from_ne_bytes([*a, *b, *c, *d, *e, *f, 0, 0]),
        [a, b, c, d, e] => usize::from_ne_bytes([*a, *b, *c, *d, *e, 0, 0, 0]),
        [a, b, c, d] => usize::from_ne_bytes([*a, *b, *c, *d, 0, 0, 0, 0]),
        [a, b, c] => usize::from_ne_bytes([*a, *b, *c, 0, 0, 0, 0, 0]),
        [a, b] => usize::from_ne_bytes([*a, *b, 0, 0, 0, 0, 0, 0]),
        [a] => usize::from_ne_bytes([*a, 0, 0, 0, 0, 0, 0, 0]),
        [] => 0,
    };
    #[cfg(target_pointer_width = "32")]
    let bytes = match bytes {
        [a, b, c, ..] => usize::from_ne_bytes([*a, *b, *c, 0]),
        [a, b] => usize::from_ne_bytes([*a, *b, 0, 0]),
        [a] => usize::from_ne_bytes([*a, 0, 0, 0]),
        [] => 0,
    };
    if cfg!(target_endian = "big") {
        bytes >> (to * 8)
    } else {
        bytes << (to * 8)
    }
}

#[inline]
const fn aligned_size(size: usize) -> usize {
    if cfg!(target_endian = "big") {
        size
    } else {
        size << (8 * NumString::capacity())
    }
}
#[inline]
const fn start_mask(at: usize) -> usize {
    if cfg!(target_endian = "big") {
        !(usize::MAX >> 8 * at)
    } else {
        !(usize::MAX << 8 * at)
    }
}

#[inline]
const fn end_mask(at: usize, size: usize) -> usize {
    if cfg!(target_endian = "big") {
        start_mask(size) >> (8 * at)
    } else {
        start_mask(size) << (8 * at)
    }
}

#[inline]
const fn offset(this: usize, from: usize, to: usize, size: usize) -> usize {
    let this = this & end_mask(from, size);
    if from > to {
        this >> ((from - to) * 8)
    } else {
        this << ((to - from) * 8)
    }
}

impl NumString {
    #[inline]
    pub const fn capacity() -> usize {
        size_of::<Self>() - 1
    }

    #[inline]
    pub const fn len(self) -> usize {
        if cfg!(target_endian = "big") {
            self.0 & 0xff
        } else {
            if let Some(it) = self.0.checked_shr(8 * Self::capacity() as u32) {
                it
            } else {
                0
            }
        }
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        unsafe {
            let len = self.len();
            let arr: &[u8; size_of::<Self>()] = mem::transmute(self);
            from_utf8_unchecked(arr.get_unchecked(..len))
        }
    }

    #[inline]
    pub const fn byte_at(self, at: usize) -> Option<u8> {
        if at >= Self::capacity() {
            return None;
        }
        let byte = if cfg!(target_endian = "big") {
            if let Some(it) = self.0.checked_shr(8 * (Self::capacity() - at) as u32) {
                it
            } else {
                0
            }
        } else {
            if let Some(it) = self.0.checked_shr(8 * at as u32) {
                it
            } else {
                0
            }
        };
        Some(byte as u8)
    }

    #[inline]
    pub fn is_char_boundary(self, at: usize) -> Result<(), Utf8> {
        trace!("Is char boundary: {} at {}", s.as_str(), idx);
        if self.as_str().is_char_boundary(at) {
            Ok(())
        } else {
            Err(Utf8)
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.0 = 0
    }

    /// Creates new empty string.
    ///
    /// ```rust
    /// # use arraystring::prelude::*;
    /// # #[cfg(not(miri))] let _ = env_logger::try_init();
    /// let string = NumString::new();
    /// assert!(string.is_empty());
    /// ```
    #[inline]
    pub const fn new() -> Self {
        Self(0)
    }

    /// Creates new `NumString` from string slice if length is lower or equal to [`capacity`], otherwise returns an error.
    ///
    /// [`capacity`]: ./struct.NumString.html#method.capacity
    /// ```rust
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # #[cfg(not(miri))] let _ = env_logger::try_init();
    /// let string = NumString::try_from_str("String")?;
    /// assert_eq!(string, "String");
    ///
    /// assert_eq!(NumString::try_from_str("")?, "");
    ///
    /// let out_of_bounds = "0".repeat(NumString::capacity() + 1);
    /// assert!(NumString::try_from_str(out_of_bounds).is_err());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[cfg_attr(all(feature = "no-panic", not(debug_assertions)), no_panic)]
    pub fn try_from_str(string: impl AsRef<str>) -> Result<Self, OutOfBounds> {
        trace!("Try from str: {}", string.as_ref());
        let mut s = Self::new();
        s.try_push_str(string)?;
        Ok(s)
    }

    /// Pushes string slice to the end of the `NumString` if total size is lower or equal to [`capacity`], otherwise returns an error.
    ///
    /// [`capacity`]: ./struct.NumString.html#method.capacity
    ///
    /// ```rust
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # #[cfg(not(miri))] let _ = env_logger::try_init();
    /// let mut s = NumString::try_from_str("My")?;
    /// s.try_push_str(" Str")?;
    /// assert_eq!(s.as_str(), "My Str");
    ///
    /// assert!(s.try_push_str("0".repeat(NumString::capacity())).is_err());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[cfg_attr(all(feature = "no-panic", not(debug_assertions)), no_panic)]
    pub fn try_push_str(&mut self, string: impl AsRef<str>) -> Result<(), OutOfBounds> {
        trace!("Push str: {}", string.as_ref());
        let len = self.len();
        self.replace_range(len..len, string)
            .map_err(|_| OutOfBounds)
    }

    /// Removes the specified range of the `NumString`, and replaces it with the given string. The given string doesn't need to have the same length as the range.
    ///
    /// ```rust
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # #[cfg(not(miri))] let _ = env_logger::try_init();
    /// let mut s = NumString::try_from_str("ABC🤔")?;
    /// s.replace_range(1..3, "E")?;
    /// assert_eq!(s, "AE🤔");
    ///
    /// assert_eq!(s.replace_range(3.., "J"), Err(Error::Utf8));
    /// assert_eq!(s.replace_range(..90, "K"), Err(Error::OutOfBounds));
    /// assert_eq!(s.replace_range(0..1, "0".repeat(NumString::capacity())), Err(Error::OutOfBounds));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[cfg_attr(all(feature = "no-panic", not(debug_assertions)), no_panic)]
    pub fn replace_range(
        &mut self,
        r: impl RangeBounds<usize>,
        with: impl AsRef<str>,
    ) -> Result<(), Error> {
        let start = match r.start_bound() {
            Bound::Included(t) => *t,
            Bound::Excluded(t) => (*t).saturating_add(1),
            Bound::Unbounded => 0,
        };
        let end = match r.end_bound() {
            Bound::Included(t) => (*t).saturating_add(1),
            Bound::Excluded(t) => *t,
            Bound::Unbounded => self.len(),
        };
        debug!(
            "Replace range (len: {}) ({}..{}) with (len: {}) {}",
            self.len(),
            start,
            end,
            with.as_ref().len(),
            with.as_ref()
        );
        let str = with.as_ref().as_bytes();
        let str_len = str.len();
        if start == end && str_len == 0 {
            return Ok(());
        }
        let this_len = self.len();
        if start > end || end > this_len || str_len > Self::capacity() {
            return Err(Error::OutOfBounds);
        }
        self.is_char_boundary(start)?;
        self.is_char_boundary(end)?;
        let end_len = this_len + str_len + start - end;
        if end_len > Self::capacity() {
            return Err(Error::OutOfBounds);
        }
        let size = aligned_size(end_len);
        if start == end && str_len == 0 {
            return Ok(());
        }
        let str = if str_len != 0 {
            from_bytes(str, start)
        } else {
            0
        };
        let this = self.0;
        let back_size = this_len - end;
        let back = if back_size != 0 {
            offset(this, end, start + str_len, back_size)
        } else {
            0
        };
        let front = if start != 0 {
            this & start_mask(start)
        } else {
            0
        };
        self.0 = str | front | back | size;
        return Ok(());
    }
}

impl AsRef<str> for NumString {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl Deref for NumString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl Borrow<str> for NumString {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl PartialEq<&str> for NumString {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<NumString> for &str {
    fn eq(&self, other: &NumString) -> bool {
        self == &other.as_str()
    }
}

impl Debug for NumString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

impl TryFrom<&str> for NumString {
    type Error = OutOfBounds;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        NumString::try_from_str(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::numstring::{from_bytes, offset};
    use crate::prelude::NumString;
    use std::ops::RangeBounds;
    use test_case::test_case;

    #[test_case(b"abcd" => usize::from_ne_bytes([b'a', b'b', b'c', b'd', 0, 0, 0, 0]); "4 bytes")]
    #[test_case(b"" => 0; "0 bytes")]
    #[test_case(b"aaaaaaa" => usize::from_ne_bytes([b'a', b'a', b'a', b'a', b'a', b'a', b'a', 0]); "7 bytes")]
    fn test_from_bytes(bytes: &[u8]) -> usize {
        from_bytes(bytes, 0)
    }

    #[test_case(0xFFFFFFFFFFFFFFFF, 0, 0, 1 => 0xFFusize; "no offset")]
    fn test_offset(this: usize, from: usize, to: usize, size: usize) -> usize {
        offset(this, from, to, size)
    }

    #[test_case("abcd" => "abcd"; "4 ascii")]
    #[test_case("" => ""; "0 ascii")]
    #[test_case("abcdefghi" => panics "called `Result::unwrap()` on an `Err` value: OutOfBounds"; "9 ascii")]
    fn test_try_from(base: &str) -> NumString {
        NumString::try_from_str(base).unwrap()
    }

    #[test_case("abcd", 0..0, "" => "abcd"; "do nothing")]
    #[test_case("abcd", 0..0, "a" => "aabcd"; "insert 1 front")]
    #[test_case("abcd", 4..4, "a" => "abcda"; "insert 1 end")]
    #[test_case("abcd", 2..2, "!!" => "ab!!cd"; "insert 2 middle")]
    #[test_case("abcd", 0..4, "!!" => "!!"; "replace all")]
    #[test_case("abcd", 0..4, "" => ""; "clear")]
    #[test_case("abcdef", 6..6, "a" => "abcdefa"; "insert 1 last")]
    fn test_replace_range(base: &str, range: impl RangeBounds<usize>, insert: &str) -> NumString {
        let mut this = NumString::try_from_str(base).unwrap();
        this.replace_range(range, insert).unwrap();
        this
    }
}
