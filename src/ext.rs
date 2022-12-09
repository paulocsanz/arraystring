//! Reusable operations for [`ArrayString`], [`CacheString`] and other implementations
//!
//! [`ArrayString`]: ../struct.ArrayString.html

use std::char::{decode_utf16, REPLACEMENT_CHARACTER};
use std::cmp::min;
use std::collections::Bound;
use std::ops::RangeBounds;
use std::ptr::copy_nonoverlapping;
use std::str::{from_utf8, from_utf8_unchecked, from_utf8_unchecked_mut};
use crate::Error;
use crate::error::{OutOfBounds, Utf16, Utf8};
use crate::utils::{encode_char_utf8_unchecked, IntoLossy, is_char_boundary, is_inside_boundary, never, shift_left_unchecked, shift_right_unchecked, truncate_str};

/// Base operations required for [`ArrayStringExt`] to be implemented automatically
pub trait ArrayStringBase {

    /// Returns `ArrayString` length.
    ///
    /// ```rust
    /// # use arraystring::{error::Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let mut s = SmallString::try_from_str("ABCD")?;
    /// assert_eq!(s.len(), 4);
    /// s.try_push('🤔')?;
    /// // Emojis use 4 bytes (this is the default rust behavior, length of u8)
    /// assert_eq!(s.len(), 8);
    /// # Ok(())
    /// # }
    /// ```
    fn len(&self) -> u8;

    /// Returns maximum string capacity, defined at compile time, it will never change
    ///
    /// ```rust
    /// # use arraystring::prelude::*;
    /// # let _ = env_logger::try_init();
    /// assert_eq!(ArrayString::<32>::capacity(), 32);
    /// ```
    fn capacity() -> u8;

    /// Creates new empty string.
    ///
    /// ```rust
    /// # use arraystring::prelude::*;
    /// # let _ = env_logger::try_init();
    /// let string = SmallString::new();
    /// assert!(string.is_empty());
    /// ```
    fn new() -> Self;

    #[doc(hidden)]
    unsafe fn raw_bytes(&self) -> &[u8];

    #[doc(hidden)]
    unsafe fn raw_bytes_mut(&mut self) -> &mut [u8];

    #[doc(hidden)]
    unsafe fn set_len_unchecked(&mut self, len: u8);
}

/// Blanket definition for ArrayString operations
pub trait ArrayStringExt: ArrayStringBase {

    /// Creates new `ArrayString` from string slice if length is lower or equal to [`capacity`], otherwise returns an error.
    ///
    /// [`capacity`]: ./struct.ArrayString.html#method.capacity
    /// ```rust
    /// # use arraystring::{error::Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let string = SmallString::try_from_str("My String")?;
    /// assert_eq!(string.as_str(), "My String");
    ///
    /// assert_eq!(SmallString::try_from_str("")?.as_str(), "");
    ///
    /// let out_of_bounds = "0".repeat(SmallString::capacity() as usize + 1);
    /// assert!(SmallString::try_from_str(out_of_bounds).is_err());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn try_from_str<S>(s: S) -> Result<Self, OutOfBounds>
        where
            S: AsRef<str>, Self: Sized
    {
        let mut string = Self::new();
        string.try_push_str(s)?;
        Ok(string)
    }

    /// Creates new `ArrayString` from string slice truncating size if bigger than [`capacity`].
    ///
    /// [`capacity`]: ./struct.ArrayString.html#method.capacity
    ///
    /// ```rust
    /// # use arraystring::prelude::*;
    /// # let _ = env_logger::try_init();
    /// let string = SmallString::from_str_truncate("My String");
    /// # assert_eq!(string.as_str(), "My String");
    /// println!("{}", string);
    ///
    /// let truncate = "0".repeat(SmallString::capacity() as usize + 1);
    /// let truncated = "0".repeat(SmallString::capacity().into());
    /// let string = SmallString::from_str_truncate(&truncate);
    /// assert_eq!(string.as_str(), truncated);
    /// ```
    #[inline]
    fn from_str_truncate<S>(string: S) -> Self
        where
            S: AsRef<str>,
            Self: Sized
    {
        let mut s = Self::new();
        s.push_str(string);
        s
    }

    /// Creates new `ArrayString` from string slice assuming length is appropriate.
    ///
    /// # Safety
    ///
    /// It's UB if `string.len()` > [`capacity`].
    ///
    /// [`capacity`]: ./struct.ArrayString.html#method.capacity
    ///
    /// ```rust
    /// # use arraystring::prelude::*;
    /// let filled = "0".repeat(SmallString::capacity().into());
    /// let string = unsafe {
    ///     SmallString::from_str_unchecked(&filled)
    /// };
    /// assert_eq!(string.as_str(), filled.as_str());
    ///
    /// // Undefined behavior, don't do it
    /// // let out_of_bounds = "0".repeat(SmallString::capacity().into() + 1);
    /// // let ub = unsafe { SmallString::from_str_unchecked(out_of_bounds) };
    /// ```
    #[inline]
    unsafe fn from_str_unchecked<S>(string: S) -> Self
        where
            S: AsRef<str>,
            Self: Sized
    {
        let mut out = Self::new();
        out.push_str_unchecked(string);
        out
    }

    /// Creates new `ArrayString` from string slice iterator if total length is lower or equal to [`capacity`], otherwise returns an error.
    ///
    /// [`capacity`]: ./struct.ArrayString.html#method.capacity
    ///
    /// ```rust
    /// # use arraystring::prelude::*;
    /// # fn main() -> Result<(), OutOfBounds> {
    /// let string = MaxString::try_from_iterator(&["My String", " My Other String"][..])?;
    /// assert_eq!(string.as_str(), "My String My Other String");
    ///
    /// let out_of_bounds = (0..100).map(|_| "000");
    /// assert!(SmallString::try_from_iterator(out_of_bounds).is_err());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn try_from_iterator<U, I>(iter: I) -> Result<Self, OutOfBounds>
        where
            U: AsRef<str>,
            I: IntoIterator<Item = U>,
            Self: Sized
    {
        let mut out = Self::new();
        for s in iter {
            out.try_push_str(s)?;
        }
        Ok(out)
    }

    /// Creates new `ArrayString` from string slice iterator truncating size if bigger than [`capacity`].
    ///
    /// [`capacity`]: ./struct.ArrayString.html#method.capacity
    ///
    /// ```rust
    /// # use arraystring::prelude::*;
    /// # fn main() -> Result<(), OutOfBounds> {
    /// # let _ = env_logger::try_init();
    /// let string = MaxString::from_iterator(&["My String", " Other String"][..]);
    /// assert_eq!(string.as_str(), "My String Other String");
    ///
    /// let out_of_bounds = (0..400).map(|_| "000");
    /// let truncated = "0".repeat(SmallString::capacity().into());
    ///
    /// let truncate = SmallString::from_iterator(out_of_bounds);
    /// assert_eq!(truncate.as_str(), truncated.as_str());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn from_iterator<U, I>(iter: I) -> Self
        where
            U: AsRef<str>,
            I: IntoIterator<Item = U>,
            Self: Sized
    {
        let mut out = Self::new();
        for s in iter {
            if out.try_push_str(s.as_ref()).is_err() {
                out.push_str(s);
                break;
            }
        }
        out
    }

    /// Creates new `ArrayString` from string slice iterator assuming length is appropriate.
    ///
    /// # Safety
    ///
    /// It's UB if `iter.map(|c| c.len()).sum()` > [`capacity`].
    ///
    /// [`capacity`]: ./struct.ArrayString.html#method.capacity
    ///
    /// ```rust
    /// # use arraystring::prelude::*;
    /// let string = unsafe {
    ///     MaxString::from_iterator_unchecked(&["My String", " My Other String"][..])
    /// };
    /// assert_eq!(string.as_str(), "My String My Other String");
    ///
    /// // Undefined behavior, don't do it
    /// // let out_of_bounds = (0..400).map(|_| "000");
    /// // let undefined_behavior = unsafe {
    /// //     SmallString::from_iterator_unchecked(out_of_bounds)
    /// // };
    /// ```
    #[inline]
    unsafe fn from_iterator_unchecked<U, I>(iter: I) -> Self
        where
            U: AsRef<str>,
            I: IntoIterator<Item = U>,
            Self: Sized
    {
        let mut out = Self::new();
        for s in iter {
            out.push_str_unchecked(s);
        }
        out
    }

    /// Creates new `ArrayString` from char iterator if total length is lower or equal to [`capacity`], otherwise returns an error.
    ///
    /// [`capacity`]: ./struct.ArrayString.html#method.capacity
    ///
    /// ```rust
    /// # use arraystring::{error::Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let string = SmallString::try_from_chars("My String".chars())?;
    /// assert_eq!(string.as_str(), "My String");
    ///
    /// let out_of_bounds = "0".repeat(SmallString::capacity() as usize + 1);
    /// assert!(SmallString::try_from_chars(out_of_bounds.chars()).is_err());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn try_from_chars<I>(iter: I) -> Result<Self, OutOfBounds>
        where
            I: IntoIterator<Item = char>,
            Self: Sized
    {
        let mut out = Self::new();
        for c in iter {
            out.try_push(c)?;
        }
        Ok(out)
    }

    /// Creates new `ArrayString` from char iterator truncating size if bigger than [`capacity`].
    ///
    /// [`capacity`]: ./struct.ArrayString.html#method.capacity
    ///
    /// ```rust
    /// # use arraystring::prelude::*;
    /// # let _ = env_logger::try_init();
    /// let string = SmallString::from_chars("My String".chars());
    /// assert_eq!(string.as_str(), "My String");
    ///
    /// let out_of_bounds = "0".repeat(SmallString::capacity() as usize + 1);
    /// let truncated = "0".repeat(SmallString::capacity().into());
    ///
    /// let truncate = SmallString::from_chars(out_of_bounds.chars());
    /// assert_eq!(truncate.as_str(), truncated.as_str());
    /// ```
    #[inline]
    fn from_chars<I>(iter: I) -> Self
        where
            I: IntoIterator<Item = char>,
            Self: Sized
    {
        let mut out = Self::new();
        for c in iter {
            if out.try_push(c).is_err() {
                break;
            }
        }
        out
    }

    /// Creates new `ArrayString` from char iterator assuming length is appropriate.
    ///
    /// # Safety
    ///
    /// It's UB if `iter.map(|c| c.len_utf8()).sum()` > [`capacity`].
    ///
    /// [`capacity`]: ./struct.ArrayString.html#method.capacity
    ///
    /// ```rust
    /// # use arraystring::prelude::*;
    /// let string = unsafe { SmallString::from_chars_unchecked("My String".chars()) };
    /// assert_eq!(string.as_str(), "My String");
    ///
    /// // Undefined behavior, don't do it
    /// // let out_of_bounds = "000".repeat(400);
    /// // let undefined_behavior = unsafe { SmallString::from_chars_unchecked(out_of_bounds.chars()) };
    /// ```
    #[inline]
    unsafe fn from_chars_unchecked<I>(iter: I) -> Self
        where
            I: IntoIterator<Item = char>,
            Self: Sized
    {
        let mut out = Self::new();
        for c in iter {
            out.push_unchecked(c)
        }
        out
    }

    /// Creates new `ArrayString` from byte slice, returning [`Utf8`] on invalid utf-8 data or [`OutOfBounds`] if bigger than [`capacity`]
    ///
    /// [`Utf8`]: ./error/enum.Error.html#variant.Utf8
    /// [`OutOfBounds`]: ./error/enum.Error.html#variant.OutOfBounds
    /// [`capacity`]: ./struct.ArrayString.html#method.capacity
    ///
    /// ```rust
    /// # use arraystring::{error::Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let string = SmallString::try_from_utf8("My String")?;
    /// assert_eq!(string.as_str(), "My String");
    ///
    /// let invalid_utf8 = [0, 159, 146, 150];
    /// assert_eq!(SmallString::try_from_utf8(invalid_utf8), Err(Error::Utf8));
    ///
    /// let out_of_bounds = "0000".repeat(400);
    /// assert_eq!(SmallString::try_from_utf8(out_of_bounds.as_bytes()), Err(Error::OutOfBounds));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn try_from_utf8<B>(slice: B) -> Result<Self, Error>
        where
            B: AsRef<[u8]>,
            Self: Sized
    {
        Ok(Self::try_from_str(from_utf8(slice.as_ref())?)?)
    }

    /// Creates new `ArrayString` from byte slice, returning [`Utf8`] on invalid utf-8 data, truncating if bigger than [`capacity`].
    ///
    /// [`Utf8`]: ./error/struct.Utf8.html
    /// [`capacity`]: ./struct.ArrayString.html#method.capacity
    ///
    /// ```rust
    /// # use arraystring::{error::Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let string = SmallString::from_utf8("My String")?;
    /// assert_eq!(string.as_str(), "My String");
    ///
    /// let invalid_utf8 = [0, 159, 146, 150];
    /// assert_eq!(SmallString::from_utf8(invalid_utf8), Err(Utf8));
    ///
    /// let out_of_bounds = "0".repeat(300);
    /// assert_eq!(SmallString::from_utf8(out_of_bounds.as_bytes())?.as_str(),
    ///            "0".repeat(SmallString::capacity().into()).as_str());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn from_utf8<B>(slice: B) -> Result<Self, Utf8>
        where
            B: AsRef<[u8]>,
            Self: Sized
    {
        Ok(Self::from_str_truncate(from_utf8(slice.as_ref())?))
    }

    /// Creates new `ArrayString` from byte slice assuming it's utf-8 and of a appropriate size.
    ///
    /// # Safety
    ///
    /// It's UB if `slice` is not a valid utf-8 string or `slice.len()` > [`capacity`].
    ///
    /// [`capacity`]: ./struct.ArrayString.html#method.capacity
    ///
    /// ```rust
    /// # use arraystring::prelude::*;
    /// let string = unsafe { SmallString::from_utf8_unchecked("My String") };
    /// assert_eq!(string.as_str(), "My String");
    ///
    /// // Undefined behavior, don't do it
    /// // let out_of_bounds = "0".repeat(300);
    /// // let ub = unsafe { SmallString::from_utf8_unchecked(out_of_bounds)) };
    /// ```
    #[inline]
    unsafe fn from_utf8_unchecked<B>(slice: B) -> Self
        where
            B: AsRef<[u8]>,
            Self: Sized
    {
        debug_assert!(from_utf8(slice.as_ref()).is_ok());
        Self::from_str_unchecked(from_utf8_unchecked(slice.as_ref()))
    }

    /// Creates new `ArrayString` from `u16` slice, returning [`Utf16`] on invalid utf-16 data or [`OutOfBounds`] if bigger than [`capacity`]
    ///
    /// [`Utf16`]: ./error/enum.Error.html#variant.Utf16
    /// [`OutOfBounds`]: ./error/enum.Error.html#variant.OutOfBounds
    /// [`capacity`]: ./struct.ArrayString.html#method.capacity
    ///
    /// ```rust
    /// # use arraystring::{error::Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let music = [0xD834, 0xDD1E, 0x006d, 0x0075, 0x0073, 0x0069, 0x0063];
    /// let string = SmallString::try_from_utf16(music)?;
    /// assert_eq!(string.as_str(), "𝄞music");
    ///
    /// let invalid_utf16 = [0xD834, 0xDD1E, 0x006d, 0x0075, 0xD800, 0x0069, 0x0063];
    /// assert_eq!(SmallString::try_from_utf16(invalid_utf16), Err(Error::Utf16));
    ///
    /// let out_of_bounds: Vec<_> = (0..300).map(|_| 0).collect();
    /// assert_eq!(SmallString::try_from_utf16(out_of_bounds), Err(Error::OutOfBounds));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn try_from_utf16<B>(slice: B) -> Result<Self, Error>
        where
            B: AsRef<[u16]>,
            Self: Sized
    {
        let mut out = Self::new();
        for c in decode_utf16(slice.as_ref().iter().cloned()) {
            out.try_push(c?)?;
        }
        Ok(out)
    }

    /// Creates new `ArrayString` from `u16` slice, returning [`Utf16`] on invalid utf-16 data, truncating if bigger than [`capacity`].
    ///
    /// [`Utf16`]: ./error/struct.Utf16.html
    /// [`capacity`]: ./struct.ArrayString.html#method.capacity
    ///
    /// ```rust
    /// # use arraystring::{error::Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let music = [0xD834, 0xDD1E, 0x006d, 0x0075, 0x0073, 0x0069, 0x0063];
    /// let string = SmallString::from_utf16(music)?;
    /// assert_eq!(string.as_str(), "𝄞music");
    ///
    /// let invalid_utf16 = [0xD834, 0xDD1E, 0x006d, 0x0075, 0xD800, 0x0069, 0x0063];
    /// assert_eq!(SmallString::from_utf16(invalid_utf16), Err(Utf16));
    ///
    /// let out_of_bounds: Vec<u16> = (0..300).map(|_| 0).collect();
    /// assert_eq!(SmallString::from_utf16(out_of_bounds)?.as_str(),
    ///            "\0".repeat(SmallString::capacity().into()).as_str());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn from_utf16<B>(slice: B) -> Result<Self, Utf16>
        where
            B: AsRef<[u16]>,
            Self: Sized
    {
        let mut out = Self::new();
        for c in decode_utf16(slice.as_ref().iter().cloned()) {
            if out.try_push(c?).is_err() {
                break;
            }
        }
        Ok(out)
    }

    /// Creates new `ArrayString` from `u16` slice, replacing invalid utf-16 data with `REPLACEMENT_CHARACTER` (\u{FFFD}) and truncating size if bigger than [`capacity`]
    ///
    /// [`capacity`]: ./struct.ArrayString.html#method.capacity
    ///
    /// ```rust
    /// # use arraystring::{error::Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let music = [0xD834, 0xDD1E, 0x006d, 0x0075, 0x0073, 0x0069, 0x0063];
    /// let string = SmallString::from_utf16_lossy(music);
    /// assert_eq!(string.as_str(), "𝄞music");
    ///
    /// let invalid_utf16 = [0xD834, 0xDD1E, 0x006d, 0x0075, 0xD800, 0x0069, 0x0063];
    /// assert_eq!(SmallString::from_utf16_lossy(invalid_utf16).as_str(), "𝄞mu\u{FFFD}ic");
    ///
    /// let out_of_bounds: Vec<u16> = (0..300).map(|_| 0).collect();
    /// assert_eq!(SmallString::from_utf16_lossy(&out_of_bounds).as_str(),
    ///            "\0".repeat(SmallString::capacity().into()).as_str());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn from_utf16_lossy<B>(slice: B) -> Self
        where
            B: AsRef<[u16]>,
            Self: Sized
    {
        let mut out = Self::new();
        for c in decode_utf16(slice.as_ref().iter().cloned()) {
            if out.try_push(c.unwrap_or(REPLACEMENT_CHARACTER)).is_err() {
                break;
            }
        }
        out
    }

    /// Extracts a string slice containing the entire `ArrayString`
    ///
    /// ```rust
    /// # use arraystring::{error::Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let s = SmallString::try_from_str("My String")?;
    /// assert_eq!(s.as_str(), "My String");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn as_str(&self) -> &str {
        unsafe { from_utf8_unchecked(self.as_bytes()) }
    }

    /// Extracts a mutable string slice containing the entire `ArrayString`
    ///
    /// ```rust
    /// # use arraystring::{error::Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let mut s = SmallString::try_from_str("My String")?;
    /// assert_eq!(s.as_mut_str(), "My String");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn as_mut_str(&mut self) -> &mut str {
        unsafe { from_utf8_unchecked_mut(self.as_mut_bytes()) }
    }

    /// Extracts a byte slice containing the entire `ArrayString`
    ///
    /// ```rust
    /// # use arraystring::{error::Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let s = SmallString::try_from_str("My String")?;
    /// assert_eq!(s.as_bytes(), "My String".as_bytes());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn as_bytes(&self) -> &[u8] {
        unsafe {self.raw_bytes().get_unchecked(..self.len().into())}
    }

    /// Extracts a mutable string slice containing the entire `ArrayString`
    ///
    /// ```rust
    /// # use arraystring::{error::Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// let mut s = SmallString::try_from_str("My String")?;
    /// assert_eq!(unsafe { s.as_mut_bytes() }, "My String".as_bytes());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    unsafe fn as_mut_bytes(&mut self) -> &mut [u8] {
        let len = self.len().into();
        unsafe {self.raw_bytes_mut().get_unchecked_mut(..len)}
    }

    /// Pushes string slice to the end of the `ArrayString` if total size is lower or equal to [`capacity`], otherwise returns an error.
    ///
    /// [`capacity`]: ./struct.ArrayString.html#method.capacity
    ///
    /// ```rust
    /// # use arraystring::{error::Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let mut s = MaxString::try_from_str("My String")?;
    /// s.try_push_str(" My other String")?;
    /// assert_eq!(s.as_str(), "My String My other String");
    ///
    /// assert!(s.try_push_str("0".repeat(MaxString::capacity().into())).is_err());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn try_push_str<S>(&mut self, string: S) -> Result<(), OutOfBounds>
        where
            S: AsRef<str>,
    {
        let new_end = string.as_ref().len().saturating_add(self.len().into());
        is_inside_boundary(new_end, Self::capacity())?;
        unsafe { self.push_str_unchecked(string) };
        Ok(())
    }

    /// Pushes string slice to the end of the `ArrayString` truncating total size if bigger than [`capacity`].
    ///
    /// [`capacity`]: ./struct.ArrayString.html#method.capacity
    ///
    /// ```rust
    /// # use arraystring::{error::Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let mut s = MaxString::try_from_str("My String")?;
    /// s.push_str(" My other String");
    /// assert_eq!(s.as_str(), "My String My other String");
    ///
    /// let mut s = SmallString::default();
    /// s.push_str("0".repeat(SmallString::capacity() as usize + 1));
    /// assert_eq!(s.as_str(), "0".repeat(SmallString::capacity().into()).as_str());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn push_str<S>(&mut self, string: S)
        where
            S: AsRef<str>,
    {
        let size = Self::capacity().saturating_sub(self.len());
        unsafe { self.push_str_unchecked(truncate_str(string.as_ref(), size)) }
    }

    /// Pushes string slice to the end of the `ArrayString` assuming total size is appropriate.
    ///
    /// # Safety
    ///
    /// It's UB if `self.len() + string.len()` > [`capacity`].
    ///
    /// [`capacity`]: ./struct.ArrayString.html#method.capacity
    ///
    /// ```rust
    /// # use arraystring::{error::Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// let mut s = MaxString::try_from_str("My String")?;
    /// unsafe { s.push_str_unchecked(" My other String") };
    /// assert_eq!(s.as_str(), "My String My other String");
    ///
    /// // Undefined behavior, don't do it
    /// // let mut undefined_behavior = SmallString::default();
    /// // undefined_behavior.push_str_unchecked("0".repeat(SmallString::capacity().into() + 1));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    unsafe fn push_str_unchecked<S>(&mut self, string: S)
        where
            S: AsRef<str>,
    {
        let (s, len) = (string.as_ref(), string.as_ref().len());
        debug_assert!(len.saturating_add(self.len().into()) <= Self::capacity() as usize);

        let dest = self.as_mut_bytes().as_mut_ptr().add(self.len().into());
        copy_nonoverlapping(s.as_ptr(), dest, len);
        self.set_len_unchecked(self.len().saturating_add(len.into_lossy()));
    }

    /// Inserts character to the end of the `ArrayString` erroring if total size if bigger than [`capacity`].
    ///
    /// [`capacity`]: ./struct.ArrayString.html#method.capacity
    ///
    /// ```rust
    /// # use arraystring::{error::Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let mut s = SmallString::try_from_str("My String")?;
    /// s.try_push('!')?;
    /// assert_eq!(s.as_str(), "My String!");
    ///
    /// let mut s = SmallString::try_from_str(&"0".repeat(SmallString::capacity().into()))?;
    /// assert!(s.try_push('!').is_err());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn try_push(&mut self, character: char) -> Result<(), OutOfBounds> {
        let new_end = character.len_utf8().saturating_add(self.len().into());
        is_inside_boundary(new_end, Self::capacity())?;
        unsafe { self.push_unchecked(character) };
        Ok(())
    }

    /// Inserts character to the end of the `ArrayString` assuming length is appropriate
    ///
    /// # Safety
    ///
    /// It's UB if `self.len() + character.len_utf8()` > [`capacity`]
    ///
    /// [`capacity`]: ./struct.ArrayString.html#method.capacity
    ///
    /// ```rust
    /// # use arraystring::{error::Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// let mut s = SmallString::try_from_str("My String")?;
    /// unsafe { s.push_unchecked('!') };
    /// assert_eq!(s.as_str(), "My String!");
    ///
    /// // s = SmallString::try_from_str(&"0".repeat(SmallString::capacity().into()))?;
    /// // Undefined behavior, don't do it
    /// // s.push_unchecked('!');
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    unsafe fn push_unchecked(&mut self, ch: char) {
        let (len, chlen) = (self.len(), ch.len_utf8().into_lossy());
        encode_char_utf8_unchecked(self, ch, len);
        self.set_len_unchecked(self.len().saturating_add(chlen));
    }

    /// Truncates `ArrayString` to specified size (if smaller than current size and a valid utf-8 char index).
    ///
    /// ```rust
    /// # use arraystring::{error::Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let mut s = SmallString::try_from_str("My String")?;
    /// s.truncate(5)?;
    /// assert_eq!(s.as_str(), "My St");
    ///
    /// // Does nothing
    /// s.truncate(6)?;
    /// assert_eq!(s.as_str(), "My St");
    ///
    /// // Index is not at a valid char
    /// let mut s = SmallString::try_from_str("🤔")?;
    /// assert!(s.truncate(1).is_err());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn truncate(&mut self, size: u8) -> Result<(), Utf8> {
        let len = min(self.len(), size);
        unsafe { is_char_boundary(self, len).map(|()| self.set_len_unchecked(len)) }
    }

    /// Removes last character from `ArrayString`, if any.
    ///
    /// ```rust
    /// # use arraystring::{error::Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let mut s = SmallString::try_from_str("A🤔")?;
    /// assert_eq!(s.pop(), Some('🤔'));
    /// assert_eq!(s.pop(), Some('A'));
    /// assert_eq!(s.pop(), None);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn pop(&mut self) -> Option<char> {
        self.as_str().chars().last().map(|ch| unsafe {
            self.set_len_unchecked(self.len().saturating_sub(ch.len_utf8().into_lossy()));
            ch
        })
    }

    /// Removes spaces from the beggining and end of the string
    ///
    /// ```rust
    /// # use arraystring::prelude::*;
    /// # fn main() -> Result<(), OutOfBounds> {
    /// # let _ = env_logger::try_init();
    /// let mut string = MaxString::try_from_str("   to be trimmed     ")?;
    /// string.trim();
    /// assert_eq!(string.as_str(), "to be trimmed");
    ///
    /// let mut string = SmallString::try_from_str("   🤔")?;
    /// string.trim();
    /// assert_eq!(string.as_str(), "🤔");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn trim(&mut self) {
        let is_whitespace = |s: &[u8], index: usize| {
            debug_assert!(index < s.len());
            unsafe { s.get_unchecked(index) == &b' ' }
        };
        let (mut start, mut end, mut leave) = (0_u8, self.len(), 0_u8);
        while start < end && leave < 2 {
            leave = 0;

            if is_whitespace(self.as_bytes(), start.into()) {
                start = start.saturating_add(1);
                if start >= end {
                    continue;
                };
            } else {
                leave = leave.saturating_add(1);
            }

            if start < end && is_whitespace(self.as_bytes(), end.saturating_sub(1).into()) {
                end = end.saturating_sub(1);
            } else {
                leave = leave.saturating_add(1);
            }
        }

        unsafe {
            shift_left_unchecked(self, start, 0u8);
            self.set_len_unchecked(end.saturating_sub(start));
        }
    }

    /// Removes specified char from `ArrayString`
    ///
    /// ```rust
    /// # use arraystring::{error::Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let mut s = SmallString::try_from_str("ABCD🤔")?;
    /// assert_eq!(s.remove("ABCD🤔".len() as u8), Err(Error::OutOfBounds));
    /// assert_eq!(s.remove(10), Err(Error::OutOfBounds));
    /// assert_eq!(s.remove(6), Err(Error::Utf8));
    /// assert_eq!(s.remove(0), Ok('A'));
    /// assert_eq!(s.as_str(), "BCD🤔");
    /// assert_eq!(s.remove(2), Ok('D'));
    /// assert_eq!(s.as_str(), "BC🤔");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn remove(&mut self, idx: u8) -> Result<char, Error> {
        is_inside_boundary(idx, self.len().saturating_sub(1))?;
        is_char_boundary(self, idx)?;
        debug_assert!(idx < self.len() && self.as_str().is_char_boundary(idx.into()));
        let ch = unsafe { self.as_str().get_unchecked(idx.into()..).chars().next() };
        let ch = ch.unwrap_or_else(|| unsafe { never("Missing char") });
        unsafe {
            shift_left_unchecked(self, idx.saturating_add(ch.len_utf8().into_lossy()), idx);
            self.set_len_unchecked(self.len().saturating_sub(ch.len_utf8().into_lossy()));
        }
        Ok(ch)
    }

    /// Retains only the characters specified by the predicate.
    ///
    /// ```rust
    /// # use arraystring::{error::Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let mut s = SmallString::try_from_str("ABCD🤔")?;
    /// s.retain(|c| c != '🤔');
    /// assert_eq!(s.as_str(), "ABCD");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn retain<F: FnMut(char) -> bool>(&mut self, mut f: F) where Self: Sized {
        // Not the most efficient solution, we could shift left during batch mismatch
        *self = unsafe { Self::from_chars_unchecked(self.as_str().chars().filter(|c| f(*c))) };
    }

    /// Inserts character at specified index, returning error if total length is bigger than [`capacity`].
    ///
    /// Returns [`OutOfBounds`] if `idx` is out of bounds and [`Utf8`] if `idx` is not a char position
    ///
    /// [`capacity`]: ./struct.ArrayString.html#method.capacity
    /// [`OutOfBounds`]: ./error/enum.Error.html#variant.OutOfBounds
    /// [`Utf8`]: ./error/enum.Error.html#variant.Utf8
    ///
    /// ```rust
    /// # use arraystring::{error::Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let mut s = SmallString::try_from_str("ABCD🤔")?;
    /// s.try_insert(1, 'A')?;
    /// s.try_insert(2, 'B')?;
    /// assert_eq!(s.as_str(), "AABBCD🤔");
    /// assert_eq!(s.try_insert(20, 'C'), Err(Error::OutOfBounds));
    /// assert_eq!(s.try_insert(8, 'D'), Err(Error::Utf8));
    ///
    /// let mut s = SmallString::try_from_str(&"0".repeat(SmallString::capacity().into()))?;
    /// assert_eq!(s.try_insert(0, 'C'), Err(Error::OutOfBounds));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn try_insert(&mut self, idx: u8, ch: char) -> Result<(), Error> {
        is_inside_boundary(idx, self.len())?;
        let new_end = ch.len_utf8().saturating_add(self.len().into());
        is_inside_boundary(new_end, Self::capacity())?;
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
    /// It's UB if `self.len() + character.len_utf8()` > [`capacity`]
    ///
    /// [`capacity`]: ./struct.ArrayString.html#method.capacity
    ///
    /// ```rust
    /// # use arraystring::{error::Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// let mut s = SmallString::try_from_str("ABCD🤔")?;
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
    unsafe fn insert_unchecked(&mut self, idx: u8, ch: char) {
        let clen = ch.len_utf8().into_lossy();
        shift_right_unchecked(self, idx, idx.saturating_add(clen));
        encode_char_utf8_unchecked(self, ch, idx);
        self.set_len_unchecked(self.len().saturating_add(clen));
    }

    /// Inserts string slice at specified index, returning error if total length is bigger than [`capacity`].
    ///
    /// Returns [`OutOfBounds`] if `idx` is out of bounds
    /// Returns [`Utf8`] if `idx` is not a char position
    ///
    /// [`capacity`]: ./struct.ArrayString.html#method.capacity
    /// [`OutOfBounds`]: ./error/enum.Error.html#variant.OutOfBounds
    /// [`Utf8`]: ./error/enum.Error.html#variant.Utf8
    ///
    /// ```rust
    /// # use arraystring::{error::Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let mut s = SmallString::try_from_str("ABCD🤔")?;
    /// s.try_insert_str(1, "AB")?;
    /// s.try_insert_str(1, "BC")?;
    /// assert_eq!(s.try_insert_str(1, "0".repeat(SmallString::capacity().into())),
    ///            Err(Error::OutOfBounds));
    /// assert_eq!(s.as_str(), "ABCABBCD🤔");
    /// assert_eq!(s.try_insert_str(20, "C"), Err(Error::OutOfBounds));
    /// assert_eq!(s.try_insert_str(10, "D"), Err(Error::Utf8));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn try_insert_str<S>(&mut self, idx: u8, s: S) -> Result<(), Error>
        where
            S: AsRef<str>,
    {
        is_inside_boundary(idx, self.len())?;
        let new_end = s.as_ref().len().saturating_add(self.len().into());
        is_inside_boundary(new_end, Self::capacity())?;
        is_char_boundary(self, idx)?;
        unsafe { self.insert_str_unchecked(idx, s.as_ref()) };
        Ok(())
    }

    /// Inserts string slice at specified index, truncating size if bigger than [`capacity`].
    ///
    /// Returns [`OutOfBounds`] if `idx` is out of bounds and [`Utf8`] if `idx` is not a char position
    ///
    /// [`capacity`]: ./struct.ArrayString.html#method.capacity
    /// [`OutOfBounds`]: ./error/enum.Error.html#variant.OutOfBounds
    /// [`Utf8`]: ./error/enum.Error.html#variant.Utf8
    ///
    /// ```rust
    /// # use arraystring::{error::Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let mut s = SmallString::try_from_str("ABCD🤔")?;
    /// s.insert_str(1, "AB")?;
    /// s.insert_str(1, "BC")?;
    /// assert_eq!(s.as_str(), "ABCABBCD🤔");
    ///
    /// assert_eq!(s.insert_str(20, "C"), Err(Error::OutOfBounds));
    /// assert_eq!(s.insert_str(10, "D"), Err(Error::Utf8));
    ///
    /// s.clear();
    /// s.insert_str(0, "0".repeat(SmallString::capacity() as usize + 10))?;
    /// assert_eq!(s.as_str(), "0".repeat(SmallString::capacity().into()).as_str());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn insert_str<S>(&mut self, idx: u8, string: S) -> Result<(), Error>
        where
            S: AsRef<str>,
    {
        is_inside_boundary(idx, self.len())?;
        is_char_boundary(self, idx)?;
        let size = Self::capacity().saturating_sub(self.len());
        unsafe { self.insert_str_unchecked(idx, truncate_str(string.as_ref(), size)) };
        Ok(())
    }

    /// Inserts string slice at specified index, assuming total length is appropriate.
    ///
    /// # Safety
    ///
    /// It's UB if `idx` does not lie on a utf-8 `char` boundary
    ///
    /// It's UB if `self.len() + string.len()` > [`capacity`]
    ///
    /// [`capacity`]: ./struct.ArrayString.html#method.capacity
    ///
    /// ```rust
    /// # use arraystring::{error::Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// let mut s = SmallString::try_from_str("ABCD🤔")?;
    /// unsafe { s.insert_str_unchecked(1, "AB") };
    /// unsafe { s.insert_str_unchecked(1, "BC") };
    /// assert_eq!(s.as_str(), "ABCABBCD🤔");
    ///
    /// // Undefined behavior, don't do it
    /// // unsafe { s.insert_str_unchecked(20, "C") };
    /// // unsafe { s.insert_str_unchecked(10, "D") };
    /// // unsafe { s.insert_str_unchecked(1, "0".repeat(SmallString::capacity().into())) };
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    unsafe fn insert_str_unchecked<S>(&mut self, idx: u8, string: S)
        where
            S: AsRef<str>,
    {
        let (s, slen) = (string.as_ref(), string.as_ref().len().into_lossy());
        let ptr = s.as_ptr();
        debug_assert!(self.len().saturating_add(slen) <= Self::capacity());
        debug_assert!(idx <= self.len());
        debug_assert!(self.as_str().is_char_boundary(idx.into()));

        shift_right_unchecked(self, idx, idx.saturating_add(slen));
        let dest = self.as_mut_bytes().as_mut_ptr().add(idx.into());
        copy_nonoverlapping(ptr, dest, slen.into());
        self.set_len_unchecked(self.len().saturating_add(slen));
    }

    /// Checks if `ArrayString` is empty.
    ///
    /// ```rust
    /// # use arraystring::{error::Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let mut s = SmallString::try_from_str("ABCD")?;
    /// assert!(!s.is_empty());
    /// s.clear();
    /// assert!(s.is_empty());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Splits `ArrayString` in two if `at` is smaller than `self.len()`.
    ///
    /// Returns [`Utf8`] if `at` does not lie at a valid utf-8 char boundary and [`OutOfBounds`] if it's out of bounds
    ///
    /// [`OutOfBounds`]: ./error/enum.Error.html#variant.OutOfBounds
    /// [`Utf8`]: ./error/enum.Error.html#variant.Utf8
    ///
    /// ```rust
    /// # use arraystring::{error::Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let mut s = SmallString::try_from_str("AB🤔CD")?;
    /// assert_eq!(s.split_off(6)?.as_str(), "CD");
    /// assert_eq!(s.as_str(), "AB🤔");
    /// assert_eq!(s.split_off(20), Err(Error::OutOfBounds));
    /// assert_eq!(s.split_off(4), Err(Error::Utf8));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn split_off(&mut self, at: u8) -> Result<Self, Error> where
        Self: Sized
    {
        is_inside_boundary(at, self.len())?;
        is_char_boundary(self, at)?;
        debug_assert!(at <= self.len() && self.as_str().is_char_boundary(at.into()));
        unsafe {
            let new = Self::from_utf8_unchecked(self.as_str().get_unchecked(at.into()..));
            self.set_len_unchecked(at);
            Ok(new)
        }
    }

    /// Empties `ArrayString`
    ///
    /// ```rust
    /// # use arraystring::{error::Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let mut s = SmallString::try_from_str("ABCD")?;
    /// assert!(!s.is_empty());
    /// s.clear();
    /// assert!(s.is_empty());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn clear(&mut self) {
        unsafe {self.set_len_unchecked(0)};
    }

    /// Removes the specified range of the `ArrayString`, and replaces it with the given string. The given string doesn't need to have the same length as the range.
    ///
    /// ```rust
    /// # use arraystring::{error::Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let mut s = SmallString::try_from_str("ABCD🤔")?;
    /// s.replace_range(2..4, "EFGHI")?;
    /// assert_eq!(s.as_str(), "ABEFGHI🤔");
    ///
    /// assert_eq!(s.replace_range(9.., "J"), Err(Error::Utf8));
    /// assert_eq!(s.replace_range(..90, "K"), Err(Error::OutOfBounds));
    /// assert_eq!(s.replace_range(0..1, "0".repeat(SmallString::capacity().into())),
    ///            Err(Error::OutOfBounds));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    fn replace_range<S, R>(&mut self, r: R, with: S) -> Result<(), Error>
        where
            S: AsRef<str>,
            R: RangeBounds<u8>,
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

        let len = replace_with.len().into_lossy();

        is_inside_boundary(start, end)?;
        is_inside_boundary(end, self.len())?;
        let replaced = (end as usize).saturating_sub(start.into());
        is_inside_boundary(replaced.saturating_add(len.into()), Self::capacity())?;
        is_char_boundary(self, start)?;
        is_char_boundary(self, end)?;

        debug_assert!(start <= end && end <= self.len());
        debug_assert!(len.saturating_sub(end).saturating_add(start) <= Self::capacity());
        debug_assert!(self.as_str().is_char_boundary(start.into()));
        debug_assert!(self.as_str().is_char_boundary(end.into()));

        if start.saturating_add(len) > end {
            unsafe { shift_right_unchecked(self, end, start.saturating_add(len)) };
        } else {
            unsafe { shift_left_unchecked(self, end, start.saturating_add(len)) };
        }

        let grow = len.saturating_sub(replaced.into_lossy());
        unsafe { self.set_len_unchecked(self.len().saturating_add(grow)); }
        let ptr = replace_with.as_ptr();
        let dest = unsafe { self.as_mut_bytes().as_mut_ptr().add(start.into()) };
        unsafe { copy_nonoverlapping(ptr, dest, len.into()) };
        Ok(())
    }
}

impl<T> ArrayStringExt for T where T: ArrayStringBase, T: ?Sized {}