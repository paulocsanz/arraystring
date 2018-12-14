//! Traits to implement a String API

use core::char::{decode_utf16, REPLACEMENT_CHARACTER};
use core::str::{from_utf8, from_utf8_unchecked};
use core::{cmp::min, iter::FusedIterator, ops::*, ptr::copy_nonoverlapping};
use utils::{encode_char_utf8_unchecked, from_str, is_char_boundary, never, out_of_bounds};
use utils::{shift_left_unchecked, shift_right_unchecked, truncate_str};
use {prelude::*, Error};

/// Inner trait to abstract buffer handling, you should not use this
///
/// [`StringHandler`] is based in this abstraction
///
/// Use [`impl_string!`] to implement a type with it
///
/// [`StringHandler`]: ./trait.StringHandler.html
/// [`impl_string!`]: ../macro.impl_string.html
pub trait RawStringHandler {
    /// Raw byte slice of the entire buffer
    unsafe fn buffer(&mut self) -> &mut [u8];
    /// Increase string length
    fn add_assign_len(&mut self, val: Size);
    /// Decrease string length
    fn sub_assign_len(&mut self, val: Size);
    /// Replace string length
    fn replace_len(&mut self, val: Size);
    /// Retrieve string length
    fn get_len(&self) -> Size;
}

/// A draining iterator for [`StringHandler`].
///
/// [`StringHandler`]: ./trait.StringHandler.html
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Drain<S: StringHandler>(S, Size);

impl<S: StringHandler> Drain<S> {
    /// Extracts string slice containing the entire `Drain`.
    #[inline]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl<S: StringHandler> Iterator for Drain<S> {
    type Item = char;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0
            .as_str()
            .get(self.1 as usize..)
            .and_then(|s| s.chars().next())
            .map(|c| {
                self.1 += c.len_utf8() as Size;
                c
            })
    }
}

impl<S: StringHandler> DoubleEndedIterator for Drain<S> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }
}

impl<S: StringHandler> FusedIterator for Drain<S> {}

/// String API
pub trait StringHandler:
    AsRef<str> + AsMut<str> + AsRef<[u8]> + Default + RawStringHandler
{
    /// Maximum string size.
    const SIZE: Size;

    /// Creates new empty string.
    ///
    /// ```rust
    /// # use limited_string::prelude::*;
    /// let string = LimitedString::new();
    /// assert!(string.is_empty());
    /// ```
    #[inline]
    fn new() -> Self {
        trace!("New empty StringHandler");
        Self::default()
    }

    /// Creates new string abstraction from string slice truncating size if bigger than [`SIZE`].
    ///
    /// [`SIZE`]: ./trait.StringHandler.html#SIZE
    ///
    /// ```rust
    /// # use limited_string::prelude::*;
    /// let string = LimitedString::from_str_truncate("My String");
    /// # assert_eq!(string.as_str(), "My String");
    /// println!("{}", string);
    ///
    /// let truncate = "0".repeat(LimitedString::SIZE as usize + 10);
    /// let truncated = "0".repeat(LimitedString::SIZE as usize);
    /// let string = LimitedString::from_str_truncate(&truncate);
    /// assert_eq!(string.as_str(), truncated);
    /// ```
    #[inline]
    fn from_str_truncate<S>(string: S) -> Self
    where
        S: AsRef<str>,
    {
        trace!("FromStr truncate");
        unsafe { Self::from_str_unchecked(truncate_str(string.as_ref(), Self::SIZE)) }
    }

    /// Creates new string abstraction from string slice assuming length is appropriate.
    ///
    /// # Safety
    ///
    /// It's UB if `string.len()` > [`SIZE`].
    ///
    /// [`SIZE`]: ./trait.StringHandler.html#SIZE
    ///
    /// ```rust
    /// # use limited_string::prelude::*;
    /// let filled = "0".repeat(LimitedString::SIZE as usize);
    /// let string = unsafe {
    ///     LimitedString::from_str_unchecked(&filled)
    /// };
    /// assert_eq!(string.as_str(), filled.as_str());
    ///
    /// // Undefined behavior, don't do it
    /// // let out_of_bounds = "0".repeat(LimitedString::SIZE as usize + 1);
    /// // let ub = unsafe { LimitedString::from_str_unchecked(&out_of_bounds) };
    /// ```
    #[inline]
    unsafe fn from_str_unchecked<S>(string: S) -> Self
    where
        S: AsRef<str>,
    {
        trace!("FromStr unchecked");
        let mut out = Self::default();
        out.push_str_unchecked(string);
        out
    }

    /// Creates new string abstraction from string slice iterator if total length is lower or equal to [`SIZE`], otherwise returns an error.
    ///
    /// [`SIZE`]: ./trait.StringHandler.html#SIZE
    ///
    /// ```rust
    /// # use limited_string::prelude::*;
    /// # fn main() -> Result<(), OutOfBoundsError> {
    /// let string = LimitedString::from_iterator(&["My String", " My Other String"][..])?;
    /// assert_eq!(string.as_str(), "My String My Other String");
    ///
    /// let out_of_bounds = (0..100).map(|_| "000");
    /// assert!(LimitedString::from_iterator(out_of_bounds).is_err());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn from_iterator<U, I>(iter: I) -> Result<Self, OutOfBoundsError>
    where
        U: AsRef<str>,
        I: IntoIterator<Item = U>,
    {
        trace!("FromIterator");
        let mut out = Self::default();
        iter.into_iter()
            .map(|s| out.push_str(s))
            .collect::<Result<(), _>>()?;
        Ok(out)
    }

    /// Creates new string abstraction from string slice iterator truncating size if bigger than [`SIZE`].
    ///
    /// [`SIZE`]: ./trait.StringHandler.html#SIZE
    ///
    /// ```rust
    /// # use limited_string::prelude::*;
    /// # fn main() -> Result<(), OutOfBoundsError> {
    /// let string = LimitedString::from_iterator_truncate(&["My String", " Other String"][..]);
    /// assert_eq!(string.as_str(), "My String Other String");
    ///
    /// let out_of_bounds = (0..400).map(|_| "000");
    /// let truncated = "0".repeat(LimitedString::SIZE as usize);
    ///
    /// let truncate = LimitedString::from_iterator_truncate(out_of_bounds);
    /// assert_eq!(truncate.as_str(), truncated.as_str());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn from_iterator_truncate<U, I>(iter: I) -> Self
    where
        U: AsRef<str>,
        I: IntoIterator<Item = U>,
    {
        trace!("FromIterator truncate");
        let mut out = Self::default();
        let _ = iter
            .into_iter()
            .map(|s| out.push_str(s.as_ref()).map_err(|_| s))
            .collect::<Result<(), _>>()
            .map_err(|s| out.push_str_truncate(s));
        out
    }

    /// Creates new string abstraction from string slice iterator assuming length is appropriate.
    ///
    /// # Safety
    ///
    /// It's UB if `iter.map(|c| c.len()).sum()` > [`SIZE`].
    ///
    /// [`SIZE`]: ./trait.StringHandler.html#SIZE
    ///
    /// ```rust
    /// # use limited_string::prelude::*;
    /// let string = unsafe {
    ///     LimitedString::from_iterator_unchecked(&["My String", " My Other String"][..])
    /// };
    /// assert_eq!(string.as_str(), "My String My Other String");
    ///
    /// // Undefined behavior, don't do it
    /// // let out_of_bounds = (0..400).map(|_| "000");
    /// // let undefined_behavior = unsafe {
    /// //     LimitedString::from_iterator_unchecked(out_of_bounds)
    /// // };
    /// ```
    #[inline]
    unsafe fn from_iterator_unchecked<U, I>(iter: I) -> Self
    where
        U: AsRef<str>,
        I: IntoIterator<Item = U>,
    {
        trace!("FromIterator unchecked");
        let mut out = Self::default();
        let _ = iter.into_iter().map(|s| out.push_str_unchecked(s)).count();
        out
    }

    /// Creates new string abstraction from char iterator if total length is lower or equal to [`SIZE`], otherwise returns an error.
    ///
    /// [`SIZE`]: ./trait.StringHandler.html#SIZE
    ///
    /// ```rust
    /// # use limited_string::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// let string = LimitedString::from_chars("My String".chars())?;
    /// assert_eq!(string.as_str(), "My String");
    ///
    /// let out_of_bounds = "0".repeat(LimitedString::SIZE as usize + 1);
    /// assert!(LimitedString::from_chars(out_of_bounds.chars()).is_err());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn from_chars<I>(iter: I) -> Result<Self, OutOfBoundsError>
    where
        I: IntoIterator<Item = char>,
    {
        trace!("From chars");
        let mut out = Self::default();
        iter.into_iter()
            .map(|c| out.push(c))
            .collect::<Result<(), _>>()?;
        Ok(out)
    }

    /// Creates new string abstraction from char iterator truncating size if bigger than [`SIZE`].
    ///
    /// [`SIZE`]: ./trait.StringHandler.html#SIZE
    ///
    /// ```rust
    /// # use limited_string::prelude::*;
    /// let string = LimitedString::from_chars_truncate("My String".chars());
    /// assert_eq!(string.as_str(), "My String");
    ///
    /// let out_of_bounds = "0".repeat(LimitedString::SIZE as usize + 1);
    /// let truncated = "0".repeat(LimitedString::SIZE as usize);
    ///
    /// let truncate = LimitedString::from_chars_truncate(out_of_bounds.chars());
    /// assert_eq!(truncate.as_str(), truncated.as_str());
    /// ```
    #[inline]
    fn from_chars_truncate<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = char>,
    {
        trace!("From chars truncate");
        let mut out = Self::default();
        let _: Result<(), _> = iter.into_iter().map(|c| out.push(c)).collect();
        out
    }

    /// Creates new string abstraction from char iterator assuming length is appropriate.
    ///
    /// # Safety
    ///
    /// It's UB if `iter.map(|c| c.len_utf8()).sum()` > [`SIZE`].
    ///
    /// [`SIZE`]: ./trait.StringHandler.html#SIZE
    ///
    /// ```rust
    /// # use limited_string::prelude::*;
    /// let string = unsafe { LimitedString::from_chars_unchecked("My String".chars()) };
    /// assert_eq!(string.as_str(), "My String");
    ///
    /// // Undefined behavior, don't do it
    /// // let out_of_bounds = "000".repeat(400);
    /// // let undefined_behavior = unsafe { LimitedString::from_chars_unchecked(out_of_bounds.chars()) };
    /// ```
    #[inline]
    unsafe fn from_chars_unchecked<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = char>,
    {
        trace!("From chars unchecked");
        let mut out = Self::default();
        let _: () = iter.into_iter().map(|c| out.push_unchecked(c)).collect();
        out
    }

    /// Creates new string abstraction from byte slice, returning [`FromUtf8`] on invalid utf-8 data or [`OutOfBounds`] if bigger than [`SIZE`]
    ///
    /// [`FromUtf8`]: ../enum.Error.html#FromUtf8
    /// [`OutOfBounds`]: ../enum.Error.html#OutOfBounds
    /// [`SIZE`]: ./trait.StringHandler.html#SIZE
    ///
    /// ```rust
    /// # use limited_string::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// let string = LimitedString::from_utf8("My String")?;
    /// assert_eq!(string.as_str(), "My String");
    ///
    /// let invalid_utf8 = [0, 159, 146, 150];
    /// assert_eq!(LimitedString::from_utf8(invalid_utf8), Err(Error::FromUtf8));
    ///
    /// let out_of_bounds = "0000".repeat(400);
    /// assert_eq!(LimitedString::from_utf8(out_of_bounds.as_bytes()), Err(Error::OutOfBounds));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn from_utf8<B>(slice: B) -> Result<Self, Error>
    where
        B: AsRef<[u8]>,
    {
        debug!("From utf8: {:?}", slice.as_ref());
        Ok(from_str(from_utf8(slice.as_ref())?)?)
    }

    /// Creates new string abstraction from byte slice, returning [`FromUtf8Error`] on invalid utf-8 data, truncating if bigger than [`SIZE`].
    ///
    /// [`FromUtf8Error`]: ../struct.FromUtf8Error.html
    /// [`SIZE`]: ./trait.StringHandler.html#SIZE
    ///
    /// ```rust
    /// # use limited_string::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// let string = LimitedString::from_utf8_truncate("My String")?;
    /// assert_eq!(string.as_str(), "My String");
    ///
    /// let invalid_utf8 = [0, 159, 146, 150];
    /// assert_eq!(LimitedString::from_utf8_truncate(invalid_utf8), Err(FromUtf8Error));
    ///
    /// let out_of_bounds = "0".repeat(300);
    /// assert_eq!(LimitedString::from_utf8_truncate(out_of_bounds.as_bytes())?.as_str(),
    ///            "0".repeat(LimitedString::SIZE as usize).as_str());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn from_utf8_truncate<B>(slice: B) -> Result<Self, FromUtf8Error>
    where
        B: AsRef<[u8]>,
    {
        debug!("From utf8: {:?}", slice.as_ref());
        Ok(Self::from_str_truncate(from_utf8(slice.as_ref())?))
    }

    /// TODO
    #[inline]
    #[doc(hidden)]
    fn from_utf8_lossy<B>(_slice: B) -> Self
    where
        B: AsRef<[u8]>,
    {
        unimplemented!();
    }

    /// Creates new string abstraction from `u16` slice, returning [`FromUtf16`] on invalid utf-16 data or [`OutOfBounds`] if bigger than [`SIZE`]
    ///
    /// [`FromUtf16`]: ../enum.Error.html#FromUtf16
    /// [`OutOfBounds`]: ../enum.Error.html#OutOfBounds
    /// [`SIZE`]: ./trait.StringHandler.html#SIZE
    ///
    /// ```rust
    /// # use limited_string::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// let music = [0xD834, 0xDD1E, 0x006d, 0x0075, 0x0073, 0x0069, 0x0063];
    /// let string = LimitedString::from_utf16_truncate(music)?;
    /// assert_eq!(string.as_str(), "𝄞music");
    ///
    /// let invalid_utf16 = [0xD834, 0xDD1E, 0x006d, 0x0075, 0xD800, 0x0069, 0x0063];
    /// assert_eq!(LimitedString::from_utf16(invalid_utf16), Err(Error::FromUtf16));
    ///
    /// let out_of_bounds: Vec<_> = (0..300).map(|_| 0).collect();
    /// assert_eq!(LimitedString::from_utf16(out_of_bounds), Err(Error::OutOfBounds));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn from_utf16<B>(slice: B) -> Result<Self, Error>
    where
        B: AsRef<[u16]>,
    {
        debug!("From utf16: {:?}", slice.as_ref());
        let mut out = Self::default();
        decode_utf16(slice.as_ref().iter().cloned())
            .map(|c| c.map_err(|_| Error::FromUtf16))
            .map(|c| c.and_then(|c| Ok(out.push(c)?)))
            .collect::<Result<(), _>>()?;
        Ok(out)
    }

    /// Creates new string abstraction from u16 slice, returning [`FromUtf16Error`] on invalid utf-16 data, truncating if bigger than [`SIZE`].
    ///
    /// [`FromUtf16Error`]: ../struct.FromUtf16Error.html
    /// [`SIZE`]: ./trait.StringHandler.html#SIZE
    ///
    /// ```rust
    /// # use limited_string::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// let music = [0xD834, 0xDD1E, 0x006d, 0x0075, 0x0073, 0x0069, 0x0063];
    /// let string = LimitedString::from_utf16_truncate(music)?;
    /// assert_eq!(string.as_str(), "𝄞music");
    ///
    /// let invalid_utf16 = [0xD834, 0xDD1E, 0x006d, 0x0075, 0xD800, 0x0069, 0x0063];
    /// assert_eq!(LimitedString::from_utf16_truncate(invalid_utf16), Err(FromUtf16Error));
    ///
    /// let out_of_bounds: Vec<u16> = (0..300).map(|_| 0).collect();
    /// assert_eq!(LimitedString::from_utf16_truncate(out_of_bounds)?.as_str(),
    ///            "\0".repeat(LimitedString::SIZE as usize).as_str());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn from_utf16_truncate<B>(slice: B) -> Result<Self, FromUtf16Error>
    where
        B: AsRef<[u16]>,
    {
        debug!("From utf16: {:?}", slice.as_ref());
        let mut out = Self::default();
        decode_utf16(slice.as_ref().iter().cloned())
            .map(|c| c.map_err(|_| FromUtf16Error))
            .map(|c| c.map(|c| out.push(c).unwrap_or(())))
            .collect::<Result<(), _>>()?;
        Ok(out)
    }

    /// Creates new string abstraction from `u16` slice, replacing invalid utf-16 data with `REPLACEMENT_CHARACTER` (\u{FFFD}) and truncating size if bigger than [`SIZE`]
    ///
    /// [`SIZE`]: ./trait.StringHandler.html#SIZE
    ///
    /// ```rust
    /// # use limited_string::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// let music = [0xD834, 0xDD1E, 0x006d, 0x0075, 0x0073, 0x0069, 0x0063];
    /// let string = LimitedString::from_utf16_truncate(music)?;
    /// assert_eq!(string.as_str(), "𝄞music");
    ///
    /// let invalid_utf16 = [0xD834, 0xDD1E, 0x006d, 0x0075, 0xD800, 0x0069, 0x0063];
    /// assert_eq!(LimitedString::from_utf16_lossy(invalid_utf16).as_str(), "𝄞mu\u{FFFD}ic");
    ///
    /// let out_of_bounds: Vec<u16> = (0..300).map(|_| 0).collect();
    /// assert_eq!(LimitedString::from_utf16_lossy(&out_of_bounds).as_str(),
    ///            "\0".repeat(LimitedString::SIZE as usize).as_str());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn from_utf16_lossy<B>(slice: B) -> Self
    where
        B: AsRef<[u16]>,
    {
        debug!("From utf16 lossy: {:?}", slice.as_ref());
        let mut out = Self::default();
        let _ = decode_utf16(slice.as_ref().iter().cloned())
            .map(|c| out.push(c.unwrap_or(REPLACEMENT_CHARACTER)))
            .collect::<Result<(), _>>();
        out
    }

    /// Creates new string abstraction from byte slice assuming it's utf-8 and of a appropriate size.
    ///
    /// # Safety
    ///
    /// It's UB if `slice` is not a valid utf-8 string or `slice.len()` > [`SIZE`].
    ///
    /// [`SIZE`]: ./trait.StringHandler.html#SIZE
    ///
    /// ```rust
    /// # use limited_string::prelude::*;
    /// let string = unsafe { LimitedString::from_utf8_unchecked("My String") };
    /// assert_eq!(string.as_str(), "My String");
    ///
    /// // Undefined behavior, don't do it
    /// // let out_of_bounds = "0".repeat(300);
    /// // let ub = unsafe { LimitedString::from_utf8_unchecked(out_of_bounds)) };
    /// ```
    #[inline]
    unsafe fn from_utf8_unchecked<B>(slice: B) -> Self
    where
        B: AsRef<[u8]>,
    {
        trace!("From utf8 unchecked");
        debug_assert!(from_utf8(slice.as_ref()).is_ok());
        Self::from_str_unchecked(from_utf8_unchecked(slice.as_ref()))
    }

    /// Extracts a string slice containing the entire string abstraction
    ///
    /// ```rust
    /// # use std::str::FromStr;
    /// # use limited_string::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// let s = LimitedString::from_str("My String")?;
    /// assert_eq!(s.as_str(), "My String");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn as_str(&self) -> &str {
        trace!("As str: {}", <Self as AsRef<str>>::as_ref(self));
        self.as_ref()
    }

    /// Extracts a mutable string slice containing the entire string abstraction
    ///
    /// ```rust
    /// # use std::str::FromStr;
    /// # use limited_string::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// let mut s = LimitedString::from_str("My String")?;
    /// assert_eq!(s.as_str_mut(), "My String");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn as_str_mut(&mut self) -> &mut str {
        trace!("As mut str: {}", self.as_mut());
        self.as_mut()
    }

    /// Extracts a byte slice containing the entire string abstraction
    ///
    /// ```rust
    /// # use std::str::FromStr;
    /// # use limited_string::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// let s = LimitedString::from_str("My String")?;
    /// assert_eq!(s.as_bytes(), "My String".as_bytes());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn as_bytes(&self) -> &[u8] {
        trace!("As str: {}", self.as_str());
        self.as_ref()
    }

    /// Extracts a mutable string slice containing the entire string abstraction
    ///
    /// ```rust
    /// # use std::str::FromStr;
    /// # use limited_string::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// let mut s = LimitedString::from_str("My String")?;
    /// assert_eq!(unsafe { s.as_bytes_mut() }, "My String".as_bytes());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    unsafe fn as_bytes_mut(&mut self) -> &mut [u8] {
        trace!("As mut str: {}", self.as_str());
        let len = self.len();
        self.buffer().get_unchecked_mut(..len as usize)
    }

    /// Pushes string slice to the end of the string abstraction if total size is lower or equal to [`SIZE`], otherwise returns an error.
    ///
    /// [`SIZE`]: ./trait.StringHandler.html#SIZE
    ///
    /// ```rust
    /// # use std::str::FromStr;
    /// # use limited_string::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// let mut s = LimitedString::from_str("My String")?;
    /// s.push_str(" My other String")?;
    /// assert_eq!(s.as_str(), "My String My other String");
    ///
    /// assert!(s.push_str("0".repeat(LimitedString::SIZE as usize)).is_err());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn push_str<S>(&mut self, string: S) -> Result<(), OutOfBoundsError>
    where
        S: AsRef<str>,
    {
        trace!("Push str");
        out_of_bounds(self.len() + string.as_ref().len() as Size, Self::SIZE)?;
        Ok(unsafe { self.push_str_unchecked(string) })
    }

    /// Pushes string slice to the end of the string abstraction truncating total size if bigger than [`SIZE`].
    ///
    /// [`SIZE`]: ./trait.StringHandler.html#SIZE
    ///
    /// ```rust
    /// # use std::str::FromStr;
    /// # use limited_string::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// let mut s = LimitedString::from_str("My String")?;
    /// s.push_str_truncate(" My other String");
    /// assert_eq!(s.as_str(), "My String My other String");
    ///
    /// s.clear();
    /// s.push_str_truncate("0".repeat(LimitedString::SIZE as usize + 10));
    /// assert_eq!(s.as_str(), "0".repeat(LimitedString::SIZE as usize).as_str());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn push_str_truncate<S>(&mut self, string: S)
    where
        S: AsRef<str>,
    {
        trace!("Push str truncate");
        let size = Self::SIZE - self.len();
        unsafe { self.push_str_unchecked(truncate_str(string.as_ref(), size)) }
    }

    /// Pushes string slice to the end of the string abstraction assuming total size is appropriate.
    ///
    /// # Safety
    ///
    /// It's UB if `self.len() + string.len()` > [`SIZE`].
    ///
    /// [`SIZE`]: ./trait.StringHandler.html#SIZE
    ///
    /// ```rust
    /// # use std::str::FromStr;
    /// # use limited_string::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// let mut s = LimitedString::from_str("My String")?;
    /// unsafe { s.push_str_unchecked(" My other String") };
    /// assert_eq!(s.as_str(), "My String My other String");
    ///
    /// // Undefined behavior, don't do it
    /// // let mut undefined_behavior = LimitedString::default();
    /// // undefined_behavior.push_str_unchecked("0".repeat(LimitedString::SIZE as usize + 10));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    unsafe fn push_str_unchecked<S>(&mut self, string: S)
    where
        S: AsRef<str>,
    {
        let (string, len) = (string.as_ref(), string.as_ref().len());
        debug!(
            "PushStr unchecked: {} ({})",
            string,
            self.len() + len as Size
        );
        debug_assert!(self.len() + len as Size <= Self::SIZE);
        let dest = self.as_bytes_mut().as_mut_ptr().add(self.len() as usize);
        copy_nonoverlapping(string.as_ptr(), dest, len);
        self.add_assign_len(len as Size);
    }

    /// Inserts character to the end of the `LimitedList` erroring if total size if bigger than [`SIZE`].
    ///
    /// [`SIZE`]: ./trait.StringHandler.html#SIZE
    ///
    /// ```rust
    /// # use std::str::FromStr;
    /// # use limited_string::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// let mut s = LimitedString::from_str("My String")?;
    /// s.push('!')?;
    /// assert_eq!(s.as_str(), "My String!");
    ///
    /// let mut s = LimitedString::from_str(&"0".repeat(LimitedString::SIZE as usize))?;
    /// assert!(s.push('!').is_err());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn push(&mut self, character: char) -> Result<(), OutOfBoundsError> {
        trace!("Push: {}", character);
        out_of_bounds(self.len() + character.len_utf8() as Size, Self::SIZE)?;
        Ok(unsafe { self.push_unchecked(character) })
    }

    /// Inserts character to the end of the `LimitedList` assuming length is appropriate
    ///
    /// # Safety
    ///
    /// It's UB if `self.len() + character.len_utf8()` > [`SIZE`]
    ///
    /// [`SIZE`]: ./trait.StringHandler.html#SIZE
    ///
    /// ```rust
    /// # use std::str::FromStr;
    /// # use limited_string::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// let mut s = LimitedString::from_str("My String")?;
    /// unsafe { s.push_unchecked('!') };
    /// assert_eq!(s.as_str(), "My String!");
    ///
    /// // s = LimitedString::from_str(&"0".repeat(LimitedString::SIZE as usize))?;
    /// // Undefined behavior, don't do it
    /// // s.push_unchecked('!');
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    unsafe fn push_unchecked(&mut self, ch: char) {
        let (len, chlen) = (self.len(), ch.len_utf8() as Size);
        debug!("Push unchecked (len: {}): {} (len: {})", len, ch, chlen);
        encode_char_utf8_unchecked(self, ch, len);
        self.add_assign_len(chlen);
    }

    /// Truncates `StringHandler` to specified size (if smaller than current size and a valid utf-8 char index).
    ///
    /// ```rust
    /// # use std::str::FromStr;
    /// # use limited_string::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// let mut s = LimitedString::from_str("My String")?;
    /// s.truncate(5)?;
    /// assert_eq!(s.as_str(), "My St");
    ///
    /// // Does nothing
    /// s.truncate(6)?;
    /// assert_eq!(s.as_str(), "My St");
    ///
    /// // Index is not at a valid char
    /// let mut s = LimitedString::from_str("🤔")?;
    /// assert!(s.truncate(1).is_err());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn truncate(&mut self, size: Size) -> Result<(), FromUtf8Error> {
        debug!("Truncate: {}", size);
        let len = min(self.len(), size);
        is_char_boundary(self, len).map(|()| self.replace_len(len))
    }

    /// Removes last character from `StringHandler`, if any.
    ///
    /// ```rust
    /// # use std::str::FromStr;
    /// # use limited_string::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// let mut s = LimitedString::from_str("A🤔")?;
    /// assert_eq!(s.pop(), Some('🤔'));
    /// assert_eq!(s.pop(), Some('A'));
    /// assert_eq!(s.pop(), None);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn pop(&mut self) -> Option<char> {
        debug!("Pop");
        self.as_str().chars().last().map(|ch| {
            self.sub_assign_len(ch.len_utf8() as Size);
            ch
        })
    }

    /// Removes spaces from the beggining and end of the string
    ///
    /// ```rust
    /// # use std::str::FromStr;
    /// # use limited_string::prelude::*;
    /// # fn main() -> Result<(), OutOfBoundsError> {
    /// let mut string = LimitedString::from_str("   to be trimmed     ")?;
    /// string.trim();
    /// assert_eq!(string.as_str(), "to be trimmed");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn trim(&mut self) {
        let is_whitespace = |s: &str, index| unsafe { s.get_unchecked(index..index + 1) == " " };
        let mut start = 0;
        let mut end = self.len().saturating_sub(1);
        let mut leave = 0;
        while start < end && leave < 2 {
            leave = 0;

            if is_whitespace(self.as_str(), start as usize) {
                start += 1;
            } else {
                leave += 1;
            }

            if is_whitespace(self.as_str(), end as usize) {
                end -= 1;
            } else {
                leave += 1;
            }
        }
        unsafe { shift_left_unchecked(self, start, 0) };
        self.replace_len(end - start + 1);
    }

    /// Removes specified char from `StringHandler`
    ///
    /// ```rust
    /// # use std::str::FromStr;
    /// # use limited_string::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// let mut s = LimitedString::from_str("ABCD🤔")?;
    /// assert_eq!(s.remove(10), Err(Error::OutOfBounds));
    /// assert_eq!(s.remove(6), Err(Error::FromUtf8));
    /// assert_eq!(s.remove(0), Ok('A'));
    /// assert_eq!(s.as_str(), "BCD🤔");
    /// assert_eq!(s.remove(2), Ok('D'));
    /// assert_eq!(s.as_str(), "BC🤔");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn remove(&mut self, idx: Size) -> Result<char, Error> {
        debug!("Remove: {}", idx);
        out_of_bounds(idx + 1, self.len())?;
        is_char_boundary(self, idx)?;
        debug_assert!(idx < self.len());
        let ch = unsafe { self.as_str().get_unchecked(idx as usize..).chars().next() };
        let ch = ch.unwrap_or_else(|| unsafe { never("Missing char") });
        self.sub_assign_len(ch.len_utf8() as Size);
        unsafe { shift_left_unchecked(self, idx + ch.len_utf8() as Size, idx) };
        Ok(ch)
    }

    /// Retains only the characters specified by the predicate.
    ///
    /// ```rust
    /// # use std::str::FromStr;
    /// # use limited_string::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// let mut s = LimitedString::from_str("ABCD🤔")?;
    /// s.retain(|c| c != '🤔');
    /// assert_eq!(s.as_str(), "ABCD");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn retain<F: FnMut(char) -> bool>(&mut self, mut f: F) {
        trace!("Retain");
        // Not the most efficient solution, we could shift left during batch mismatch
        *self = unsafe { Self::from_chars_unchecked(self.as_str().chars().filter(|c| f(*c))) };
    }

    /// Inserts character at specified index, returning error if total length is bigger than [`SIZE`].
    ///
    /// Returns [`OutOfBounds`] if `idx` is out of bounds and [`FromUtf8`] if `idx` is not a char position
    ///
    /// [`SIZE`]: ./trait.StringHandler.html#SIZE
    /// [`OutOfBounds`]: ../enum.Error.html#OutOfBounds
    /// [`FromUtf8`]: ../enum.Error.html#FromUtf8
    ///
    /// ```rust
    /// # use std::str::FromStr;
    /// # use limited_string::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// let mut s = LimitedString::from_str("ABCD🤔")?;
    /// s.insert(1, 'A')?;
    /// s.insert(2, 'B')?;
    /// assert_eq!(s.as_str(), "AABBCD🤔");
    /// assert_eq!(s.insert(20, 'C'), Err(Error::OutOfBounds));
    /// assert_eq!(s.insert(8, 'D'), Err(Error::FromUtf8));
    ///
    /// let mut s = LimitedString::from_str(&"0".repeat(LimitedString::SIZE as usize))?;
    /// assert_eq!(s.insert(0, 'C'), Err(Error::OutOfBounds));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn insert(&mut self, idx: Size, ch: char) -> Result<(), Error> {
        trace!("Insert {} to {}", ch, idx);
        out_of_bounds(idx, self.len())?;
        out_of_bounds(self.len() + ch.len_utf8() as Size, Self::SIZE)?;
        is_char_boundary(self, idx)?;
        unsafe { self.insert_unchecked(idx, ch) };
        Ok(())
    }

    /// Inserts character at specified index assuming length is appropriate
    ///
    /// # Safety
    ///
    /// It's UB if `idx` does not lie on a utf-8 `char` boundary
    ///
    /// It's UB if `self.len() + character.len_utf8()` > [`SIZE`]
    ///
    /// [`SIZE`]: ./trait.StringHandler.html#SIZE
    ///
    /// ```rust
    /// # use std::str::FromStr;
    /// # use limited_string::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// let mut s = LimitedString::from_str("ABCD🤔")?;
    /// unsafe { s.insert_unchecked(1, 'A') };
    /// unsafe { s.insert_unchecked(1, 'B') };
    /// assert_eq!(s.as_str(), "ABABCD🤔");
    ///
    /// // Undefined behavior, don't do it
    /// // s.insert(20, 'C');
    /// // s.insert(8, 'D');
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    unsafe fn insert_unchecked(&mut self, idx: Size, ch: char) {
        let (_len, clen) = (self.len(), ch.len_utf8() as Size);
        debug!("Insert unchecked: {} ({}) at {}", ch, _len + clen, idx);
        shift_right_unchecked(self, idx, idx + clen);
        encode_char_utf8_unchecked(self, ch, idx);
        self.add_assign_len(clen);
    }

    /// Inserts string slice at specified index, returning error if total length is bigger than [`SIZE`].
    ///
    /// Returns [`OutOfBounds`] if `idx` is out of bounds
    /// Returns [`FromUtf8`] if `idx` is not a char position
    ///
    /// [`SIZE`]: ./trait.StringHandler.html#SIZE
    /// [`OutOfBounds`]: ../enum.Error.html#OutOfBounds
    /// [`FromUtf8`]: ../enum.Error.html#FromUtf8
    ///
    /// ```rust
    /// # use std::str::FromStr;
    /// # use limited_string::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// let mut s = LimitedString::from_str("ABCD🤔")?;
    /// s.insert_str(1, "AB")?;
    /// s.insert_str(1, "BC")?;
    /// assert_eq!(s.insert_str(1, "0".repeat(LimitedString::SIZE as usize)),
    ///            Err(Error::OutOfBounds));
    /// assert_eq!(s.as_str(), "ABCABBCD🤔");
    /// assert_eq!(s.insert_str(20, "C"), Err(Error::OutOfBounds));
    /// assert_eq!(s.insert_str(10, "D"), Err(Error::FromUtf8));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn insert_str<S>(&mut self, idx: Size, s: S) -> Result<(), Error>
    where
        S: AsRef<str>,
    {
        trace!("Insert str");
        out_of_bounds(idx, self.len())?;
        out_of_bounds(self.len() + s.as_ref().len() as Size, Self::SIZE)?;
        is_char_boundary(self, idx)?;
        unsafe { self.insert_str_unchecked(idx, s.as_ref()) };
        Ok(())
    }

    /// Inserts string slice at specified index, truncating size if bigger than [`SIZE`].
    ///
    /// Returns [`OutOfBounds`] if `idx` is out of bounds and [`FromUtf8`] if `idx` is not a char position
    ///
    /// [`SIZE`]: ./trait.StringHandler.html#SIZE
    /// [`OutOfBounds`]: ../enum.Error.html#OutOfBounds
    /// [`FromUtf8`]: ../enum.Error.html#FromUtf8
    ///
    /// ```rust
    /// # use std::str::FromStr;
    /// # use limited_string::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// let mut s = LimitedString::from_str("ABCD🤔")?;
    /// s.insert_str_truncate(1, "AB")?;
    /// s.insert_str_truncate(1, "BC")?;
    /// assert_eq!(s.as_str(), "ABCABBCD🤔");
    ///
    /// assert_eq!(s.insert_str_truncate(20, "C"), Err(Error::OutOfBounds));
    /// assert_eq!(s.insert_str_truncate(10, "D"), Err(Error::FromUtf8));
    ///
    /// s.clear();
    /// s.insert_str_truncate(0, "0".repeat(LimitedString::SIZE as usize + 10))?;
    /// assert_eq!(s.as_str(), "0".repeat(LimitedString::SIZE as usize).as_str());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn insert_str_truncate<S>(&mut self, idx: Size, string: S) -> Result<(), Error>
    where
        S: AsRef<str>,
    {
        trace!("Insert str truncate");
        out_of_bounds(idx, self.len())?;
        is_char_boundary(self, idx)?;
        let size = Self::SIZE - self.len();
        unsafe { self.insert_str_unchecked(idx, truncate_str(string.as_ref(), size)) };
        Ok(())
    }

    /// Inserts string slice at specified index, assuming total length is appropriate.
    ///
    /// # Safety
    ///
    /// It's UB if `idx` does not lie on a utf-8 `char` boundary
    ///
    /// It's UB if `self.len() + string.len()` > [`SIZE`]
    ///
    /// [`SIZE`]: ./trait.StringHandler.html#SIZE
    ///
    /// ```rust
    /// # use std::str::FromStr;
    /// # use limited_string::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// let mut s = LimitedString::from_str("ABCD🤔")?;
    /// unsafe { s.insert_str_unchecked(1, "AB") };
    /// unsafe { s.insert_str_unchecked(1, "BC") };
    /// assert_eq!(s.as_str(), "ABCABBCD🤔");
    ///
    /// // Undefined behavior, don't do it
    /// // unsafe { s.insert_str_unchecked(20, "C") };
    /// // unsafe { s.insert_str_unchecked(10, "D") };
    /// // unsafe { s.insert_str_unchecked(1, "0".repeat(LimitedString::SIZE as usize)) };
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    unsafe fn insert_str_unchecked<S>(&mut self, idx: Size, string: S)
    where
        S: AsRef<str>,
    {
        let (s, slen) = (string.as_ref(), string.as_ref().len() as Size);
        let (ptr, len) = (s.as_ptr(), self.len());
        trace!("InsertStr unchecked: {} ({}) at {}", s, len + slen, idx);
        debug_assert!(len + slen <= Self::SIZE && idx <= len);

        shift_right_unchecked(self, idx, idx + slen);
        let dest = self.as_bytes_mut().as_mut_ptr().add(idx as usize);
        copy_nonoverlapping(ptr, dest, slen as usize);
        self.add_assign_len(slen);
    }

    /// Returns `StringHandler` length.
    ///
    /// ```rust
    /// # use std::str::FromStr;
    /// # use limited_string::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// let mut s = LimitedString::from_str("ABCD")?;
    /// assert_eq!(s.len(), 4);
    /// s.push('🤔')?;
    /// // Emojis use 4 bytes (this is the default rust behavior, length of u8)
    /// assert_eq!(s.len(), 8);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn len(&self) -> Size {
        trace!("Len");
        self.get_len()
    }

    /// Checks if `StringHandler` is empty.
    ///
    /// ```rust
    /// # use std::str::FromStr;
    /// # use limited_string::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// let mut s = LimitedString::from_str("ABCD")?;
    /// assert!(!s.is_empty());
    /// s.clear();
    /// assert!(s.is_empty());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn is_empty(&self) -> bool {
        trace!("Is empty");
        self.len() == 0
    }

    /// Splits `LimitedSize` in two if `at` is smaller than `self.len()`.
    ///
    /// Returns [`FromUtf8`] if `at` does not lie at a valid utf-8 char boundary and [`OutOfBounds`] if it's out of bounds
    ///
    /// [`OutOfBounds`]: ../enum.Error.html#OutOfBounds
    /// [`FromUtf8`]: ../enum.Error.html#FromUtf8
    ///
    /// ```rust
    /// # use std::str::FromStr;
    /// # use limited_string::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// let mut s = LimitedString::from_str("AB🤔CD")?;
    /// assert_eq!(s.split_off(6)?.as_str(), "CD");
    /// assert_eq!(s.as_str(), "AB🤔");
    /// assert_eq!(s.split_off(20), Err(Error::OutOfBounds));
    /// assert_eq!(s.split_off(4), Err(Error::FromUtf8));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn split_off(&mut self, at: Size) -> Result<Self, Error> {
        debug!("Split off");
        out_of_bounds(at, self.len())?;
        is_char_boundary(self, at)?;
        let new = unsafe { Self::from_utf8_unchecked(self.as_str().get_unchecked(at as usize..)) };
        self.replace_len(at);
        Ok(new)
    }

    /// Empties `LimitedSize`
    ///
    /// ```rust
    /// # use std::str::FromStr;
    /// # use limited_string::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// let mut s = LimitedString::from_str("ABCD")?;
    /// assert!(!s.is_empty());
    /// s.clear();
    /// assert!(s.is_empty());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn clear(&mut self) {
        trace!("Clear");
        self.replace_len(0);
    }

    /// Creates a draining iterator that removes the specified range in the `StringHandler` and yields the removed chars.
    ///
    /// Note: The element range is removed even if the iterator is not consumed until the end.
    ///
    /// ```rust
    /// # use std::str::FromStr;
    /// # use limited_string::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// let mut s = LimitedString::from_str("ABCD🤔")?;
    /// assert_eq!(s.drain(..3)?.collect::<Vec<_>>(), vec!['A', 'B', 'C']);
    /// assert_eq!(s.as_str(), "D🤔");
    ///
    /// assert_eq!(s.drain(3..), Err(Error::FromUtf8));
    /// assert_eq!(s.drain(10..), Err(Error::OutOfBounds));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn drain<R>(&mut self, range: R) -> Result<Drain<Self>, Error>
    where
        R: RangeBounds<Size>,
    {
        let start = match range.start_bound() {
            Bound::Included(t) => *t,
            Bound::Excluded(t) => t.saturating_add(1),
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(t) => t.saturating_add(1),
            Bound::Excluded(t) => *t,
            Bound::Unbounded => self.len(),
        };

        debug!("Drain iterator (len: {}): {}..{}", self.len(), start, end);
        out_of_bounds(start, end)?;
        out_of_bounds(end, self.len())?;
        is_char_boundary(self, start)?;
        is_char_boundary(self, end)?;
        debug_assert!(start <= self.len());
        debug_assert!(end <= self.len());
        let drain = unsafe {
            let slice = self.as_str().get_unchecked(start as usize..end as usize);
            Self::from_str_unchecked(slice)
        };
        self.sub_assign_len(end - start);
        unsafe { shift_left_unchecked(self, end, start) };
        Ok(Drain(drain, 0))
    }

    /// Removes the specified range of the string abstraction, and replaces it with the given string. The given string doesn't need to have the same length as the range.
    ///
    /// ```rust
    /// # use std::str::FromStr;
    /// # use limited_string::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// let mut s = LimitedString::from_str("ABCD🤔")?;
    /// s.replace_range(2..4, "EFGHI")?;
    /// assert_eq!(s.as_str(), "ABEFGHI🤔");
    ///
    /// assert_eq!(s.replace_range(9.., "J"), Err(Error::FromUtf8));
    /// assert_eq!(s.replace_range(..90, "K"), Err(Error::OutOfBounds));
    /// assert_eq!(s.replace_range(0..1, "0".repeat(LimitedString::SIZE as usize)),
    ///            Err(Error::OutOfBounds));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn replace_range<S, R>(&mut self, r: R, with: S) -> Result<(), Error>
    where
        S: AsRef<str>,
        R: RangeBounds<Size>,
    {
        let replace_with = with.as_ref();
        let start = match r.start_bound() {
            Bound::Included(t) => *t,
            Bound::Excluded(t) => t.saturating_add(1),
            Bound::Unbounded => 0,
        };
        let end = match r.end_bound() {
            Bound::Included(t) => t.saturating_add(1),
            Bound::Excluded(t) => *t,
            Bound::Unbounded => self.len(),
        };
        debug!(
            "Replace range (len: {}) ({}..{}) with (len: {}) {}",
            self.len(),
            start,
            end,
            replace_with.len(),
            replace_with
        );
        out_of_bounds(start, end)?;
        out_of_bounds(end, self.len())?;
        out_of_bounds(end - start + replace_with.len() as Size, Self::SIZE)?;
        is_char_boundary(self, start)?;
        is_char_boundary(self, end)?;
        debug_assert!(start <= self.len());
        debug_assert!(end <= self.len());

        let len = replace_with.len() as Size;
        if start + len > end {
            unsafe { shift_right_unchecked(self, end, start + len) };
        } else {
            unsafe { shift_left_unchecked(self, end, start + len) };
        }

        self.add_assign_len(len - end + start);
        let (ptr, len) = (replace_with.as_ptr(), len as usize);
        unsafe {
            copy_nonoverlapping(
                ptr,
                self.as_bytes_mut().as_mut_ptr().add(start as usize),
                len,
            )
        };
        Ok(())
    }
}
