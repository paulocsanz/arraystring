//! `ArrayString` definition and Api implementation

use crate::utils::{encode_char_utf8_unchecked, is_char_boundary, is_inside_boundary, never};
use crate::utils::{shift_left_unchecked, shift_right_unchecked, truncate_str, IntoLossy};
use crate::{prelude::*, Error};
use core::char::{decode_utf16, REPLACEMENT_CHARACTER};
use core::str::{from_utf8, from_utf8_unchecked};
use core::{cmp::min, ops::*, ptr::copy_nonoverlapping};
#[cfg(feature = "logs")]
use log::{debug, trace};

/// String based on a generic array (size defined at compile time through `const generics`)
///
/// Can't outgrow capacity (defined at compile time), always occupies [`capacity`] `+ 1` bytes of memory
///
/// *Doesn't allocate memory on the heap and never panics (all panic branches are stripped at compile time)*
///
/// [`capacity`]: ./struct.ArrayString.html#method.capacity
#[derive(Copy, Clone)]
#[cfg_attr(
    feature = "diesel-traits",
    derive(diesel::AsExpression, diesel::FromSqlRow)
)]
#[cfg_attr(feature = "diesel-traits", diesel(sql_type = diesel::sql_types::Text))]
pub struct ArrayString<const N: usize> {
    /// Array type corresponding to specified `SIZE`
    pub(crate) array: [u8; N],
    /// Current string size
    pub(crate) size: u8,
}

impl<const N: usize> ArrayString<N>
where
    Self: sealed::ValidCapacity,
{
    /// Creates new empty string.
    ///
    /// ```rust
    /// # use arraystring::prelude::*;
    /// # let _ = env_logger::try_init();
    /// let string = ArrayString::<23>::new();
    /// assert!(string.is_empty());
    /// ```
    #[inline]
    pub const fn new() -> Self {
        Self {
            array: [0; N],
            size: 0,
        }
    }

    /// Creates new `ArrayString` from string slice if length is lower or equal to [`capacity`], otherwise returns an error.
    ///
    /// [`capacity`]: ./struct.ArrayString.html#method.capacity
    /// ```rust
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let string = ArrayString::<23>::try_from_str("My String")?;
    /// assert_eq!(string.as_str(), "My String");
    ///
    /// assert_eq!(ArrayString::<23>::try_from_str("")?.as_str(), "");
    ///
    /// let out_of_bounds = "0".repeat(ArrayString::<23>::capacity() as usize + 1);
    /// assert!(ArrayString::<23>::try_from_str(out_of_bounds).is_err());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn try_from_str(string: impl AsRef<str>) -> Result<Self, OutOfBounds> {
        trace!("Try from str: {}", string.as_ref());
        let mut s = Self::new();
        s.try_push_str(string)?;
        Ok(s)
    }

    /// Creates new `ArrayString` from string slice truncating size if bigger than [`capacity`].
    ///
    /// [`capacity`]: ./struct.ArrayString.html#method.capacity
    ///
    /// ```rust
    /// # use arraystring::prelude::*;
    /// # let _ = env_logger::try_init();
    /// let string = ArrayString::<23>::from_str_truncate("My String");
    /// # assert_eq!(string.as_str(), "My String");
    /// println!("{}", string);
    ///
    /// let truncate = "0".repeat(ArrayString::<23>::capacity() as usize + 1);
    /// let truncated = "0".repeat(ArrayString::<23>::capacity().into());
    /// let string = ArrayString::<23>::from_str_truncate(&truncate);
    /// assert_eq!(string.as_str(), truncated);
    /// ```
    #[inline]
    pub fn from_str_truncate(string: impl AsRef<str>) -> Self {
        trace!("FromStr truncate: {}", string.as_ref());
        let mut s = Self::new();
        s.push_str_truncate(string);
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
    /// let filled = "0".repeat(ArrayString::<23>::capacity().into());
    /// let string = unsafe {
    ///     ArrayString::<23>::from_str_unchecked(&filled)
    /// };
    /// assert_eq!(string.as_str(), filled.as_str());
    ///
    /// // Undefined behavior, don't do it
    /// // let out_of_bounds = "0".repeat(ArrayString::<23>::capacity().into() + 1);
    /// // let ub = unsafe { ArrayString::<23>::from_str_unchecked(out_of_bounds) };
    /// ```
    #[inline]
    pub unsafe fn from_str_unchecked(string: impl AsRef<str>) -> Self {
        trace!("FromStr unchecked: {}", string.as_ref());
        let mut s = Self::new();
        s.push_str_unchecked(string);
        s
    }

    /// Creates new `ArrayString` from string slice iterator if total length is lower or equal to [`capacity`], otherwise returns an error.
    ///
    /// [`capacity`]: ./struct.ArrayString.html#method.capacity
    ///
    /// ```rust
    /// # use arraystring::prelude::*;
    /// # fn main() -> Result<(), OutOfBounds> {
    /// let string = ArrayString::<255>::try_from_iterator(&["My String", " My Other String"][..])?;
    /// assert_eq!(string.as_str(), "My String My Other String");
    ///
    /// let out_of_bounds = (0..100).map(|_| "000");
    /// assert!(ArrayString::<23>::try_from_iterator(out_of_bounds).is_err());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn try_from_iterator(
        iter: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Result<Self, OutOfBounds> {
        trace!("FromIterator");
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
    /// let string = ArrayString::<255>::from_iterator_truncate(&["My String", " Other String"][..]);
    /// assert_eq!(string.as_str(), "My String Other String");
    ///
    /// let out_of_bounds = (0..400).map(|_| "000");
    /// let truncated = "0".repeat(ArrayString::<23>::capacity().into());
    ///
    /// let truncate = ArrayString::<23>::from_iterator_truncate(out_of_bounds);
    /// assert_eq!(truncate.as_str(), truncated.as_str());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn from_iterator_truncate(iter: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        trace!("FromIterator truncate");
        let mut out = Self::new();
        for s in iter {
            if out.try_push_str(s.as_ref()).is_err() {
                out.push_str_truncate(s);
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
    ///     ArrayString::<255>::from_iterator_unchecked(&["My String", " My Other String"][..])
    /// };
    /// assert_eq!(string.as_str(), "My String My Other String");
    ///
    /// // Undefined behavior, don't do it
    /// // let out_of_bounds = (0..400).map(|_| "000");
    /// // let undefined_behavior = unsafe {
    /// //     ArrayString::<23>::from_iterator_unchecked(out_of_bounds)
    /// // };
    /// ```
    #[inline]
    pub unsafe fn from_iterator_unchecked(iter: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        trace!("FromIterator unchecked");
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
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let string = ArrayString::<23>::try_from_chars("My String".chars())?;
    /// assert_eq!(string.as_str(), "My String");
    ///
    /// let out_of_bounds = "0".repeat(ArrayString::<23>::capacity() as usize + 1);
    /// assert!(ArrayString::<23>::try_from_chars(out_of_bounds.chars()).is_err());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn try_from_chars(iter: impl IntoIterator<Item = char>) -> Result<Self, OutOfBounds> {
        trace!("TryFrom chars");
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
    /// let string = ArrayString::<23>::from_chars_truncate("My String".chars());
    /// assert_eq!(string.as_str(), "My String");
    ///
    /// let out_of_bounds = "0".repeat(ArrayString::<23>::capacity() as usize + 1);
    /// let truncated = "0".repeat(ArrayString::<23>::capacity().into());
    ///
    /// let truncate = ArrayString::<23>::from_chars_truncate(out_of_bounds.chars());
    /// assert_eq!(truncate.as_str(), truncated.as_str());
    /// ```
    #[inline]
    pub fn from_chars_truncate(iter: impl IntoIterator<Item = char>) -> Self {
        trace!("From chars truncate");
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
    /// let string = unsafe { ArrayString::<23>::from_chars_unchecked("My String".chars()) };
    /// assert_eq!(string.as_str(), "My String");
    ///
    /// // Undefined behavior, don't do it
    /// // let out_of_bounds = "000".repeat(400);
    /// // let undefined_behavior = unsafe { ArrayString::<23>::from_chars_unchecked(out_of_bounds.chars()) };
    /// ```
    #[inline]
    pub unsafe fn from_chars_unchecked(iter: impl IntoIterator<Item = char>) -> Self {
        trace!("From chars unchecked");
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
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let string = ArrayString::<23>::try_from_utf8("My String")?;
    /// assert_eq!(string.as_str(), "My String");
    ///
    /// let invalid_utf8 = [0, 159, 146, 150];
    /// assert_eq!(ArrayString::<23>::try_from_utf8(invalid_utf8), Err(Error::Utf8));
    ///
    /// let out_of_bounds = "0000".repeat(400);
    /// assert_eq!(ArrayString::<23>::try_from_utf8(out_of_bounds.as_bytes()), Err(Error::OutOfBounds));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn try_from_utf8(slice: impl AsRef<[u8]>) -> Result<Self, Error> {
        debug!("From utf8: {:?}", slice.as_ref());
        Ok(Self::try_from_str(from_utf8(slice.as_ref())?)?)
    }

    /// Creates new `ArrayString` from byte slice, returning [`Utf8`] on invalid utf-8 data, truncating if bigger than [`capacity`].
    ///
    /// [`Utf8`]: ./error/struct.Utf8.html
    /// [`capacity`]: ./struct.ArrayString.html#method.capacity
    ///
    /// ```rust
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let string = ArrayString::<23>::from_utf8_truncate("My String")?;
    /// assert_eq!(string.as_str(), "My String");
    ///
    /// let invalid_utf8 = [0, 159, 146, 150];
    /// assert_eq!(ArrayString::<23>::from_utf8_truncate(invalid_utf8), Err(Utf8));
    ///
    /// let out_of_bounds = "0".repeat(300);
    /// assert_eq!(ArrayString::<23>::from_utf8_truncate(out_of_bounds.as_bytes())?.as_str(),
    ///            "0".repeat(ArrayString::<23>::capacity().into()).as_str());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn from_utf8_truncate(slice: impl AsRef<[u8]>) -> Result<Self, Utf8> {
        debug!("From utf8: {:?}", slice.as_ref());
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
    /// let string = unsafe { ArrayString::<23>::from_utf8_unchecked("My String") };
    /// assert_eq!(string.as_str(), "My String");
    ///
    /// // Undefined behavior, don't do it
    /// // let out_of_bounds = "0".repeat(300);
    /// // let ub = unsafe { ArrayString::<23>::from_utf8_unchecked(out_of_bounds)) };
    /// ```
    #[inline]
    pub unsafe fn from_utf8_unchecked(slice: impl AsRef<[u8]>) -> Self {
        trace!("From utf8 unchecked: {:?}", slice.as_ref());
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
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let music = [0xD834, 0xDD1E, 0x006d, 0x0075, 0x0073, 0x0069, 0x0063];
    /// let string = ArrayString::<23>::try_from_utf16(music)?;
    /// assert_eq!(string.as_str(), "𝄞music");
    ///
    /// let invalid_utf16 = [0xD834, 0xDD1E, 0x006d, 0x0075, 0xD800, 0x0069, 0x0063];
    /// assert_eq!(ArrayString::<23>::try_from_utf16(invalid_utf16), Err(Error::Utf16));
    ///
    /// let out_of_bounds: Vec<_> = (0..300).map(|_| 0).collect();
    /// assert_eq!(ArrayString::<23>::try_from_utf16(out_of_bounds), Err(Error::OutOfBounds));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn try_from_utf16(slice: impl AsRef<[u16]>) -> Result<Self, Error> {
        debug!("From utf16: {:?}", slice.as_ref());
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
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let music = [0xD834, 0xDD1E, 0x006d, 0x0075, 0x0073, 0x0069, 0x0063];
    /// let string = ArrayString::<23>::from_utf16_truncate(music)?;
    /// assert_eq!(string.as_str(), "𝄞music");
    ///
    /// let invalid_utf16 = [0xD834, 0xDD1E, 0x006d, 0x0075, 0xD800, 0x0069, 0x0063];
    /// assert_eq!(ArrayString::<23>::from_utf16_truncate(invalid_utf16), Err(Utf16));
    ///
    /// let out_of_bounds: Vec<u16> = (0..300).map(|_| 0).collect();
    /// assert_eq!(ArrayString::<23>::from_utf16_truncate(out_of_bounds)?.as_str(),
    ///            "\0".repeat(ArrayString::<23>::capacity().into()).as_str());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn from_utf16_truncate(slice: impl AsRef<[u16]>) -> Result<Self, Utf16> {
        debug!("From utf16: {:?}", slice.as_ref());
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
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let music = [0xD834, 0xDD1E, 0x006d, 0x0075, 0x0073, 0x0069, 0x0063];
    /// let string = ArrayString::<23>::from_utf16_lossy_truncate(music);
    /// assert_eq!(string.as_str(), "𝄞music");
    ///
    /// let invalid_utf16 = [0xD834, 0xDD1E, 0x006d, 0x0075, 0xD800, 0x0069, 0x0063];
    /// assert_eq!(ArrayString::<23>::from_utf16_lossy_truncate(invalid_utf16).as_str(), "𝄞mu\u{FFFD}ic");
    ///
    /// let out_of_bounds: Vec<u16> = (0..300).map(|_| 0).collect();
    /// assert_eq!(ArrayString::<23>::from_utf16_lossy_truncate(&out_of_bounds).as_str(),
    ///            "\0".repeat(ArrayString::<23>::capacity().into()).as_str());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn from_utf16_lossy_truncate(slice: impl AsRef<[u16]>) -> Self {
        debug!("From utf16 lossy: {:?}", slice.as_ref());
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
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let s = ArrayString::<23>::try_from_str("My String")?;
    /// assert_eq!(s.as_str(), "My String");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn as_str(&self) -> &str {
        trace!("As str: {self}");
        self.as_ref()
    }

    /// Extracts a mutable string slice containing the entire `ArrayString`
    ///
    /// ```rust
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let mut s = ArrayString::<23>::try_from_str("My String")?;
    /// assert_eq!(s.as_mut_str(), "My String");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn as_mut_str(&mut self) -> &mut str {
        trace!("As mut str: {self}");
        self.as_mut()
    }

    /// Extracts a byte slice containing the entire `ArrayString`
    ///
    /// ```rust
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let s = ArrayString::<23>::try_from_str("My String")?;
    /// assert_eq!(s.as_bytes(), "My String".as_bytes());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        trace!("As bytes");
        self.as_ref()
    }

    /// Extracts a mutable string slice containing the entire `ArrayString`
    ///
    /// # Safety
    ///
    /// It's UB to store invalid UTF-8 data in the returned byte array
    ///
    /// ```rust
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// let mut s = ArrayString::<23>::try_from_str("My String")?;
    /// assert_eq!(unsafe { s.as_mut_bytes() }, "My String".as_bytes());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub unsafe fn as_mut_bytes(&mut self) -> &mut [u8] {
        trace!("As mut str");
        let len = self.len();
        self.array.as_mut_slice().get_unchecked_mut(..len.into())
    }

    /// Returns maximum string capacity, defined at compile time, it will never change
    ///
    /// ```rust
    /// # use arraystring::prelude::*;
    /// # let _ = env_logger::try_init();
    /// assert_eq!(ArrayString::<32>::capacity(), 32);
    /// ```
    #[inline]
    pub const fn capacity() -> u8 {
        N as u8
    }

    /// Pushes string slice to the end of the `ArrayString` if total size is lower or equal to [`capacity`], otherwise returns an error.
    ///
    /// [`capacity`]: ./struct.ArrayString.html#method.capacity
    ///
    /// ```rust
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let mut s = ArrayString::<255>::try_from_str("My String")?;
    /// s.try_push_str(" My other String")?;
    /// assert_eq!(s.as_str(), "My String My other String");
    ///
    /// assert!(s.try_push_str("0".repeat(ArrayString::<255>::capacity().into())).is_err());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn try_push_str(&mut self, string: impl AsRef<str>) -> Result<(), OutOfBounds> {
        trace!("Push str: {}", string.as_ref());
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
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let mut s = ArrayString::<255>::try_from_str("My String")?;
    /// s.push_str_truncate(" My other String");
    /// assert_eq!(s.as_str(), "My String My other String");
    ///
    /// let mut s = ArrayString::<23>::default();
    /// s.push_str_truncate("0".repeat(ArrayString::<23>::capacity() as usize + 1));
    /// assert_eq!(s.as_str(), "0".repeat(ArrayString::<23>::capacity().into()).as_str());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn push_str_truncate(&mut self, string: impl AsRef<str>) {
        trace!("Push str truncate: {}", string.as_ref());
        let size = Self::capacity().saturating_sub(self.len());
        unsafe { self.push_str_unchecked(truncate_str(string.as_ref(), size.into())) }
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
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// let mut s = ArrayString::<255>::try_from_str("My String")?;
    /// unsafe { s.push_str_unchecked(" My other String") };
    /// assert_eq!(s.as_str(), "My String My other String");
    ///
    /// // Undefined behavior, don't do it
    /// // let mut undefined_behavior = ArrayString::<23>::default();
    /// // undefined_behavior.push_str_unchecked("0".repeat(ArrayString::<23>::capacity().into() + 1));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub unsafe fn push_str_unchecked(&mut self, string: impl AsRef<str>) {
        let (s, len) = (string.as_ref(), string.as_ref().len());
        debug!("Push str unchecked: {} ({} + {})", s, self.len(), len);
        debug_assert!(len.saturating_add(self.len().into()) <= Self::capacity() as usize);

        let dest = self.as_mut_bytes().as_mut_ptr().add(self.len().into());
        copy_nonoverlapping(s.as_ptr(), dest, len);
        self.size = self.size.saturating_add(len.into_lossy());
    }

    /// Inserts character to the end of the `ArrayString` erroring if total size if bigger than [`capacity`].
    ///
    /// [`capacity`]: ./struct.ArrayString.html#method.capacity
    ///
    /// ```rust
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let mut s = ArrayString::<23>::try_from_str("My String")?;
    /// s.try_push('!')?;
    /// assert_eq!(s.as_str(), "My String!");
    ///
    /// let mut s = ArrayString::<23>::try_from_str(&"0".repeat(ArrayString::<23>::capacity().into()))?;
    /// assert!(s.try_push('!').is_err());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn try_push(&mut self, character: char) -> Result<(), OutOfBounds> {
        trace!("Push: {}", character);
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
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// let mut s = ArrayString::<23>::try_from_str("My String")?;
    /// unsafe { s.push_unchecked('!') };
    /// assert_eq!(s.as_str(), "My String!");
    ///
    /// // s = ArrayString::<23>::try_from_str(&"0".repeat(ArrayString::<23>::capacity().into()))?;
    /// // Undefined behavior, don't do it
    /// // s.push_unchecked('!');
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub unsafe fn push_unchecked(&mut self, ch: char) {
        let (len, chlen) = (self.len(), ch.len_utf8().into_lossy());
        debug!("Push unchecked (len: {}): {} (len: {})", len, ch, chlen);
        encode_char_utf8_unchecked(self, ch, len);
        self.size = self.size.saturating_add(chlen);
    }

    /// Truncates `ArrayString` to specified size (if smaller than current size and a valid utf-8 char index).
    ///
    /// ```rust
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let mut s = ArrayString::<23>::try_from_str("My String")?;
    /// s.truncate(5)?;
    /// assert_eq!(s.as_str(), "My St");
    ///
    /// // Does nothing
    /// s.truncate(6)?;
    /// assert_eq!(s.as_str(), "My St");
    ///
    /// // Index is not at a valid char
    /// let mut s = ArrayString::<23>::try_from_str("🤔")?;
    /// assert!(s.truncate(1).is_err());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn truncate(&mut self, size: u8) -> Result<(), Utf8> {
        debug!("Truncate: {}", size);
        let len = min(self.len(), size);
        is_char_boundary(self, len).map(|()| self.size = len)
    }

    /// Removes last character from `ArrayString`, if any.
    ///
    /// ```rust
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let mut s = ArrayString::<23>::try_from_str("A🤔")?;
    /// assert_eq!(s.pop(), Some('🤔'));
    /// assert_eq!(s.pop(), Some('A'));
    /// assert_eq!(s.pop(), None);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn pop(&mut self) -> Option<char> {
        debug!("Pop");
        self.as_str().chars().last().map(|ch| {
            self.size = self.size.saturating_sub(ch.len_utf8().into_lossy());
            ch
        })
    }

    /// Removes spaces from the beggining and end of the string
    ///
    /// ```rust
    /// # use arraystring::prelude::*;
    /// # fn main() -> Result<(), OutOfBounds> {
    /// # let _ = env_logger::try_init();
    /// let mut string = ArrayString::<255>::try_from_str("   to be trimmed     ")?;
    /// string.trim();
    /// assert_eq!(string.as_str(), "to be trimmed");
    ///
    /// let mut string = ArrayString::<23>::try_from_str("   🤔")?;
    /// string.trim();
    /// assert_eq!(string.as_str(), "🤔");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn trim(&mut self) {
        trace!("Trim");
        let is_whitespace = |s: &[u8], index: u8| {
            debug_assert!((index as usize) < s.len());
            unsafe { s.get_unchecked(index as usize) == &b' ' }
        };
        let (mut start, mut end, mut leave) = (0_u8, self.len(), 0_u8);
        while start < end && leave < 2 {
            leave = 0;

            if is_whitespace(self.as_bytes(), start) {
                start += 1;
            } else {
                leave += 1;
            }

            if start < end && is_whitespace(self.as_bytes(), end - 1) {
                end -= 1;
            } else {
                leave += 1;
            }
        }

        unsafe { shift_left_unchecked(self, start, 0u8) };
        self.size = end.saturating_sub(start);
    }

    /// Removes specified char from `ArrayString`
    ///
    /// ```rust
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let mut s = ArrayString::<23>::try_from_str("ABCD🤔")?;
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
    pub fn remove(&mut self, idx: u8) -> Result<char, Error> {
        debug!("Remove: {}", idx);
        is_inside_boundary(idx.saturating_add(1), self.len())?;
        is_char_boundary(self, idx)?;
        debug_assert!(idx < self.len() && self.as_str().is_char_boundary(idx.into()));
        let ch = unsafe { self.as_str().get_unchecked(idx.into()..).chars().next() };
        let ch = ch.unwrap_or_else(|| unsafe { never("Missing char") });
        unsafe { shift_left_unchecked(self, idx.saturating_add(ch.len_utf8().into_lossy()), idx) };
        self.size = self.size.saturating_sub(ch.len_utf8().into_lossy());
        Ok(ch)
    }

    /// Retains only the characters specified by the predicate.
    ///
    /// ```rust
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let mut s = ArrayString::<23>::try_from_str("ABCD🤔")?;
    /// s.retain(|c| c != '🤔');
    /// assert_eq!(s.as_str(), "ABCD");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn retain(&mut self, mut f: impl FnMut(char) -> bool) {
        trace!("Retain");
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
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let mut s = ArrayString::<23>::try_from_str("ABCD🤔")?;
    /// s.try_insert(1, 'A')?;
    /// s.try_insert(2, 'B')?;
    /// assert_eq!(s.as_str(), "AABBCD🤔");
    /// assert_eq!(s.try_insert(20, 'C'), Err(Error::OutOfBounds));
    /// assert_eq!(s.try_insert(8, 'D'), Err(Error::Utf8));
    ///
    /// let mut s = ArrayString::<23>::try_from_str(&"0".repeat(ArrayString::<23>::capacity().into()))?;
    /// assert_eq!(s.try_insert(0, 'C'), Err(Error::OutOfBounds));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn try_insert(&mut self, idx: u8, ch: char) -> Result<(), Error> {
        trace!("Insert {} to {}", ch, idx);
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
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// let mut s = ArrayString::<23>::try_from_str("ABCD🤔")?;
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
    pub unsafe fn insert_unchecked(&mut self, idx: u8, ch: char) {
        let clen = ch.len_utf8().into_lossy();
        debug!("Insert uncheck ({}+{clen}) {ch} at {idx}", self.len());
        shift_right_unchecked(self, idx, idx.saturating_add(clen));
        encode_char_utf8_unchecked(self, ch, idx);
        self.size = self.size.saturating_add(clen);
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
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let mut s = ArrayString::<23>::try_from_str("ABCD🤔")?;
    /// s.try_insert_str(1, "AB")?;
    /// s.try_insert_str(1, "BC")?;
    /// assert_eq!(s.try_insert_str(1, "0".repeat(ArrayString::<23>::capacity().into())),
    ///            Err(Error::OutOfBounds));
    /// assert_eq!(s.as_str(), "ABCABBCD🤔");
    /// assert_eq!(s.try_insert_str(20, "C"), Err(Error::OutOfBounds));
    /// assert_eq!(s.try_insert_str(10, "D"), Err(Error::Utf8));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn try_insert_str(&mut self, idx: u8, s: impl AsRef<str>) -> Result<(), Error> {
        trace!("Try insert at {idx} str: {}", s.as_ref());
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
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let mut s = ArrayString::<23>::try_from_str("ABCD🤔")?;
    /// s.insert_str_truncate(1, "AB")?;
    /// s.insert_str_truncate(1, "BC")?;
    /// assert_eq!(s.as_str(), "ABCABBCD🤔");
    ///
    /// assert_eq!(s.insert_str_truncate(20, "C"), Err(Error::OutOfBounds));
    /// assert_eq!(s.insert_str_truncate(10, "D"), Err(Error::Utf8));
    ///
    /// s.clear();
    /// s.insert_str_truncate(0, "0".repeat(ArrayString::<23>::capacity() as usize + 10))?;
    /// assert_eq!(s.as_str(), "0".repeat(ArrayString::<23>::capacity().into()).as_str());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn insert_str_truncate(&mut self, idx: u8, string: impl AsRef<str>) -> Result<(), Error> {
        trace!("Insert str at {idx}: {}", string.as_ref());
        is_inside_boundary(idx, self.len())?;
        is_char_boundary(self, idx)?;
        let size = Self::capacity().saturating_sub(self.len());
        unsafe { self.insert_str_unchecked(idx, truncate_str(string.as_ref(), size.into())) };
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
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// let mut s = ArrayString::<23>::try_from_str("ABCD🤔")?;
    /// unsafe { s.insert_str_unchecked(1, "AB") };
    /// unsafe { s.insert_str_unchecked(1, "BC") };
    /// assert_eq!(s.as_str(), "ABCABBCD🤔");
    ///
    /// // Undefined behavior, don't do it
    /// // unsafe { s.insert_str_unchecked(20, "C") };
    /// // unsafe { s.insert_str_unchecked(10, "D") };
    /// // unsafe { s.insert_str_unchecked(1, "0".repeat(ArrayString::<23>::capacity().into())) };
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub unsafe fn insert_str_unchecked(&mut self, idx: u8, string: impl AsRef<str>) {
        let (s, slen) = (string.as_ref(), string.as_ref().len().into_lossy());
        let ptr = s.as_ptr();
        trace!("InsertStr uncheck {}+{slen} {s} at {idx}", self.len());
        debug_assert!(self.len().saturating_add(slen) <= Self::capacity());
        debug_assert!(idx <= self.len());
        debug_assert!(self.as_str().is_char_boundary(idx.into()));

        shift_right_unchecked(self, idx, idx.saturating_add(slen));
        let dest = self.as_mut_bytes().as_mut_ptr().add(idx.into());
        copy_nonoverlapping(ptr, dest, slen.into());
        self.size = self.size.saturating_add(slen);
    }

    /// Returns `ArrayString` length.
    ///
    /// ```rust
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let mut s = ArrayString::<23>::try_from_str("ABCD")?;
    /// assert_eq!(s.len(), 4);
    /// s.try_push('🤔')?;
    /// // Emojis use 4 bytes (this is the default rust behavior, length of u8)
    /// assert_eq!(s.len(), 8);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn len(&self) -> u8 {
        trace!("Len");
        self.size
    }

    /// Checks if `ArrayString` is empty.
    ///
    /// ```rust
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let mut s = ArrayString::<23>::try_from_str("ABCD")?;
    /// assert!(!s.is_empty());
    /// s.clear();
    /// assert!(s.is_empty());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        trace!("Is empty");
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
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let mut s = ArrayString::<23>::try_from_str("AB🤔CD")?;
    /// assert_eq!(s.split_off(6)?.as_str(), "CD");
    /// assert_eq!(s.as_str(), "AB🤔");
    /// assert_eq!(s.split_off(20), Err(Error::OutOfBounds));
    /// assert_eq!(s.split_off(4), Err(Error::Utf8));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn split_off(&mut self, at: u8) -> Result<Self, Error> {
        debug!("Split off");
        is_inside_boundary(at, self.len())?;
        is_char_boundary(self, at)?;
        debug_assert!(at <= self.len() && self.as_str().is_char_boundary(at.into()));
        let new = unsafe { Self::from_utf8_unchecked(self.as_str().get_unchecked(at.into()..)) };
        self.size = at;
        Ok(new)
    }

    /// Empties `ArrayString`
    ///
    /// ```rust
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let mut s = ArrayString::<23>::try_from_str("ABCD")?;
    /// assert!(!s.is_empty());
    /// s.clear();
    /// assert!(s.is_empty());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        trace!("Clear");
        self.size = 0;
    }

    /// Creates a draining iterator that removes the specified range in the `ArrayString` and yields the removed chars.
    ///
    /// Note: The element range is removed even if the iterator is not consumed until the end.
    ///
    /// ```rust
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let mut s = ArrayString::<23>::try_from_str("ABCD🤔")?;
    /// assert_eq!(s.drain(..3)?.collect::<Vec<_>>(), vec!['A', 'B', 'C']);
    /// assert_eq!(s.as_str(), "D🤔");
    ///
    /// assert_eq!(s.drain(3..), Err(Error::Utf8));
    /// assert_eq!(s.drain(10..), Err(Error::OutOfBounds));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn drain(&mut self, range: impl RangeBounds<u8>) -> Result<Drain<N>, Error> {
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

        debug!("Drain iterator (len: {}): {start}..{end}", self.len());
        is_inside_boundary(start, end)?;
        is_inside_boundary(end, self.len())?;
        is_char_boundary(self, start)?;
        is_char_boundary(self, end)?;
        debug_assert!(start <= end && end <= self.len());
        debug_assert!(self.as_str().is_char_boundary(start.into()));
        debug_assert!(self.as_str().is_char_boundary(end.into()));

        let drain = unsafe {
            let slice = self.as_str().get_unchecked(start.into()..end.into());
            Self::from_str_unchecked(slice)
        };
        unsafe { shift_left_unchecked(self, end, start) };
        self.size = self.size.saturating_sub(end.saturating_sub(start));
        Ok(Drain(drain))
    }

    /// Removes the specified range of the `ArrayString`, and replaces it with the given string. The given string doesn't need to have the same length as the range.
    ///
    /// ```rust
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let mut s = ArrayString::<23>::try_from_str("ABCD🤔")?;
    /// s.replace_range(2..4, "EFGHI")?;
    /// assert_eq!(s.as_str(), "ABEFGHI🤔");
    ///
    /// assert_eq!(s.replace_range(9.., "J"), Err(Error::Utf8));
    /// assert_eq!(s.replace_range(..90, "K"), Err(Error::OutOfBounds));
    /// assert_eq!(s.replace_range(0..1, "0".repeat(ArrayString::<23>::capacity().into())),
    ///            Err(Error::OutOfBounds));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn replace_range(
        &mut self,
        r: impl RangeBounds<u8>,
        with: impl AsRef<str>,
    ) -> Result<(), Error> {
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
        debug!(
            "Replace range (len: {}) ({}..{}) with (len: {}) {}",
            self.len(),
            start,
            end,
            len,
            replace_with
        );

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
        self.size = self.size.saturating_add(grow);
        let ptr = replace_with.as_ptr();
        let dest = unsafe { self.as_mut_bytes().as_mut_ptr().add(start.into()) };
        unsafe { copy_nonoverlapping(ptr, dest, len.into()) };
        Ok(())
    }
}

pub(crate) mod sealed {
    use super::*;
    pub trait ValidCapacity {}
    impl ValidCapacity for ArrayString<1> {}
    impl ValidCapacity for ArrayString<2> {}
    impl ValidCapacity for ArrayString<3> {}
    impl ValidCapacity for ArrayString<4> {}
    impl ValidCapacity for ArrayString<5> {}
    impl ValidCapacity for ArrayString<6> {}
    impl ValidCapacity for ArrayString<7> {}
    impl ValidCapacity for ArrayString<8> {}
    impl ValidCapacity for ArrayString<9> {}
    impl ValidCapacity for ArrayString<10> {}
    impl ValidCapacity for ArrayString<11> {}
    impl ValidCapacity for ArrayString<12> {}
    impl ValidCapacity for ArrayString<13> {}
    impl ValidCapacity for ArrayString<14> {}
    impl ValidCapacity for ArrayString<15> {}
    impl ValidCapacity for ArrayString<16> {}
    impl ValidCapacity for ArrayString<17> {}
    impl ValidCapacity for ArrayString<18> {}
    impl ValidCapacity for ArrayString<19> {}
    impl ValidCapacity for ArrayString<20> {}
    impl ValidCapacity for ArrayString<21> {}
    impl ValidCapacity for ArrayString<22> {}
    impl ValidCapacity for ArrayString<23> {}
    impl ValidCapacity for ArrayString<24> {}
    impl ValidCapacity for ArrayString<25> {}
    impl ValidCapacity for ArrayString<26> {}
    impl ValidCapacity for ArrayString<27> {}
    impl ValidCapacity for ArrayString<28> {}
    impl ValidCapacity for ArrayString<29> {}
    impl ValidCapacity for ArrayString<30> {}
    impl ValidCapacity for ArrayString<31> {}
    impl ValidCapacity for ArrayString<32> {}
    impl ValidCapacity for ArrayString<33> {}
    impl ValidCapacity for ArrayString<34> {}
    impl ValidCapacity for ArrayString<35> {}
    impl ValidCapacity for ArrayString<36> {}
    impl ValidCapacity for ArrayString<37> {}
    impl ValidCapacity for ArrayString<38> {}
    impl ValidCapacity for ArrayString<39> {}
    impl ValidCapacity for ArrayString<40> {}
    impl ValidCapacity for ArrayString<41> {}
    impl ValidCapacity for ArrayString<42> {}
    impl ValidCapacity for ArrayString<43> {}
    impl ValidCapacity for ArrayString<44> {}
    impl ValidCapacity for ArrayString<45> {}
    impl ValidCapacity for ArrayString<46> {}
    impl ValidCapacity for ArrayString<47> {}
    impl ValidCapacity for ArrayString<48> {}
    impl ValidCapacity for ArrayString<49> {}
    impl ValidCapacity for ArrayString<50> {}
    impl ValidCapacity for ArrayString<51> {}
    impl ValidCapacity for ArrayString<52> {}
    impl ValidCapacity for ArrayString<53> {}
    impl ValidCapacity for ArrayString<54> {}
    impl ValidCapacity for ArrayString<55> {}
    impl ValidCapacity for ArrayString<56> {}
    impl ValidCapacity for ArrayString<57> {}
    impl ValidCapacity for ArrayString<58> {}
    impl ValidCapacity for ArrayString<59> {}
    impl ValidCapacity for ArrayString<60> {}
    impl ValidCapacity for ArrayString<61> {}
    impl ValidCapacity for ArrayString<62> {}
    impl ValidCapacity for ArrayString<63> {}
    impl ValidCapacity for ArrayString<64> {}
    impl ValidCapacity for ArrayString<65> {}
    impl ValidCapacity for ArrayString<66> {}
    impl ValidCapacity for ArrayString<67> {}
    impl ValidCapacity for ArrayString<68> {}
    impl ValidCapacity for ArrayString<69> {}
    impl ValidCapacity for ArrayString<70> {}
    impl ValidCapacity for ArrayString<71> {}
    impl ValidCapacity for ArrayString<72> {}
    impl ValidCapacity for ArrayString<73> {}
    impl ValidCapacity for ArrayString<74> {}
    impl ValidCapacity for ArrayString<75> {}
    impl ValidCapacity for ArrayString<76> {}
    impl ValidCapacity for ArrayString<77> {}
    impl ValidCapacity for ArrayString<78> {}
    impl ValidCapacity for ArrayString<79> {}
    impl ValidCapacity for ArrayString<80> {}
    impl ValidCapacity for ArrayString<81> {}
    impl ValidCapacity for ArrayString<82> {}
    impl ValidCapacity for ArrayString<83> {}
    impl ValidCapacity for ArrayString<84> {}
    impl ValidCapacity for ArrayString<85> {}
    impl ValidCapacity for ArrayString<86> {}
    impl ValidCapacity for ArrayString<87> {}
    impl ValidCapacity for ArrayString<88> {}
    impl ValidCapacity for ArrayString<89> {}
    impl ValidCapacity for ArrayString<90> {}
    impl ValidCapacity for ArrayString<91> {}
    impl ValidCapacity for ArrayString<92> {}
    impl ValidCapacity for ArrayString<93> {}
    impl ValidCapacity for ArrayString<94> {}
    impl ValidCapacity for ArrayString<95> {}
    impl ValidCapacity for ArrayString<96> {}
    impl ValidCapacity for ArrayString<97> {}
    impl ValidCapacity for ArrayString<98> {}
    impl ValidCapacity for ArrayString<99> {}
    impl ValidCapacity for ArrayString<100> {}
    impl ValidCapacity for ArrayString<101> {}
    impl ValidCapacity for ArrayString<102> {}
    impl ValidCapacity for ArrayString<103> {}
    impl ValidCapacity for ArrayString<104> {}
    impl ValidCapacity for ArrayString<105> {}
    impl ValidCapacity for ArrayString<106> {}
    impl ValidCapacity for ArrayString<107> {}
    impl ValidCapacity for ArrayString<108> {}
    impl ValidCapacity for ArrayString<109> {}
    impl ValidCapacity for ArrayString<110> {}
    impl ValidCapacity for ArrayString<111> {}
    impl ValidCapacity for ArrayString<112> {}
    impl ValidCapacity for ArrayString<113> {}
    impl ValidCapacity for ArrayString<114> {}
    impl ValidCapacity for ArrayString<115> {}
    impl ValidCapacity for ArrayString<116> {}
    impl ValidCapacity for ArrayString<117> {}
    impl ValidCapacity for ArrayString<118> {}
    impl ValidCapacity for ArrayString<119> {}
    impl ValidCapacity for ArrayString<120> {}
    impl ValidCapacity for ArrayString<121> {}
    impl ValidCapacity for ArrayString<122> {}
    impl ValidCapacity for ArrayString<123> {}
    impl ValidCapacity for ArrayString<124> {}
    impl ValidCapacity for ArrayString<125> {}
    impl ValidCapacity for ArrayString<126> {}
    impl ValidCapacity for ArrayString<127> {}
    impl ValidCapacity for ArrayString<128> {}
    impl ValidCapacity for ArrayString<129> {}
    impl ValidCapacity for ArrayString<130> {}
    impl ValidCapacity for ArrayString<131> {}
    impl ValidCapacity for ArrayString<132> {}
    impl ValidCapacity for ArrayString<133> {}
    impl ValidCapacity for ArrayString<134> {}
    impl ValidCapacity for ArrayString<135> {}
    impl ValidCapacity for ArrayString<136> {}
    impl ValidCapacity for ArrayString<137> {}
    impl ValidCapacity for ArrayString<138> {}
    impl ValidCapacity for ArrayString<139> {}
    impl ValidCapacity for ArrayString<140> {}
    impl ValidCapacity for ArrayString<141> {}
    impl ValidCapacity for ArrayString<142> {}
    impl ValidCapacity for ArrayString<143> {}
    impl ValidCapacity for ArrayString<144> {}
    impl ValidCapacity for ArrayString<145> {}
    impl ValidCapacity for ArrayString<146> {}
    impl ValidCapacity for ArrayString<147> {}
    impl ValidCapacity for ArrayString<148> {}
    impl ValidCapacity for ArrayString<149> {}
    impl ValidCapacity for ArrayString<150> {}
    impl ValidCapacity for ArrayString<151> {}
    impl ValidCapacity for ArrayString<152> {}
    impl ValidCapacity for ArrayString<153> {}
    impl ValidCapacity for ArrayString<154> {}
    impl ValidCapacity for ArrayString<155> {}
    impl ValidCapacity for ArrayString<156> {}
    impl ValidCapacity for ArrayString<157> {}
    impl ValidCapacity for ArrayString<158> {}
    impl ValidCapacity for ArrayString<159> {}
    impl ValidCapacity for ArrayString<160> {}
    impl ValidCapacity for ArrayString<161> {}
    impl ValidCapacity for ArrayString<162> {}
    impl ValidCapacity for ArrayString<163> {}
    impl ValidCapacity for ArrayString<164> {}
    impl ValidCapacity for ArrayString<165> {}
    impl ValidCapacity for ArrayString<166> {}
    impl ValidCapacity for ArrayString<167> {}
    impl ValidCapacity for ArrayString<168> {}
    impl ValidCapacity for ArrayString<169> {}
    impl ValidCapacity for ArrayString<170> {}
    impl ValidCapacity for ArrayString<171> {}
    impl ValidCapacity for ArrayString<172> {}
    impl ValidCapacity for ArrayString<173> {}
    impl ValidCapacity for ArrayString<174> {}
    impl ValidCapacity for ArrayString<175> {}
    impl ValidCapacity for ArrayString<176> {}
    impl ValidCapacity for ArrayString<177> {}
    impl ValidCapacity for ArrayString<178> {}
    impl ValidCapacity for ArrayString<179> {}
    impl ValidCapacity for ArrayString<180> {}
    impl ValidCapacity for ArrayString<181> {}
    impl ValidCapacity for ArrayString<182> {}
    impl ValidCapacity for ArrayString<183> {}
    impl ValidCapacity for ArrayString<184> {}
    impl ValidCapacity for ArrayString<185> {}
    impl ValidCapacity for ArrayString<186> {}
    impl ValidCapacity for ArrayString<187> {}
    impl ValidCapacity for ArrayString<188> {}
    impl ValidCapacity for ArrayString<189> {}
    impl ValidCapacity for ArrayString<190> {}
    impl ValidCapacity for ArrayString<191> {}
    impl ValidCapacity for ArrayString<192> {}
    impl ValidCapacity for ArrayString<193> {}
    impl ValidCapacity for ArrayString<194> {}
    impl ValidCapacity for ArrayString<195> {}
    impl ValidCapacity for ArrayString<196> {}
    impl ValidCapacity for ArrayString<197> {}
    impl ValidCapacity for ArrayString<198> {}
    impl ValidCapacity for ArrayString<199> {}
    impl ValidCapacity for ArrayString<200> {}
    impl ValidCapacity for ArrayString<201> {}
    impl ValidCapacity for ArrayString<202> {}
    impl ValidCapacity for ArrayString<203> {}
    impl ValidCapacity for ArrayString<204> {}
    impl ValidCapacity for ArrayString<205> {}
    impl ValidCapacity for ArrayString<206> {}
    impl ValidCapacity for ArrayString<207> {}
    impl ValidCapacity for ArrayString<208> {}
    impl ValidCapacity for ArrayString<209> {}
    impl ValidCapacity for ArrayString<210> {}
    impl ValidCapacity for ArrayString<211> {}
    impl ValidCapacity for ArrayString<212> {}
    impl ValidCapacity for ArrayString<213> {}
    impl ValidCapacity for ArrayString<214> {}
    impl ValidCapacity for ArrayString<215> {}
    impl ValidCapacity for ArrayString<216> {}
    impl ValidCapacity for ArrayString<217> {}
    impl ValidCapacity for ArrayString<218> {}
    impl ValidCapacity for ArrayString<219> {}
    impl ValidCapacity for ArrayString<220> {}
    impl ValidCapacity for ArrayString<221> {}
    impl ValidCapacity for ArrayString<222> {}
    impl ValidCapacity for ArrayString<223> {}
    impl ValidCapacity for ArrayString<224> {}
    impl ValidCapacity for ArrayString<225> {}
    impl ValidCapacity for ArrayString<226> {}
    impl ValidCapacity for ArrayString<227> {}
    impl ValidCapacity for ArrayString<228> {}
    impl ValidCapacity for ArrayString<229> {}
    impl ValidCapacity for ArrayString<230> {}
    impl ValidCapacity for ArrayString<231> {}
    impl ValidCapacity for ArrayString<232> {}
    impl ValidCapacity for ArrayString<233> {}
    impl ValidCapacity for ArrayString<234> {}
    impl ValidCapacity for ArrayString<235> {}
    impl ValidCapacity for ArrayString<236> {}
    impl ValidCapacity for ArrayString<237> {}
    impl ValidCapacity for ArrayString<238> {}
    impl ValidCapacity for ArrayString<239> {}
    impl ValidCapacity for ArrayString<240> {}
    impl ValidCapacity for ArrayString<241> {}
    impl ValidCapacity for ArrayString<242> {}
    impl ValidCapacity for ArrayString<243> {}
    impl ValidCapacity for ArrayString<244> {}
    impl ValidCapacity for ArrayString<245> {}
    impl ValidCapacity for ArrayString<246> {}
    impl ValidCapacity for ArrayString<247> {}
    impl ValidCapacity for ArrayString<248> {}
    impl ValidCapacity for ArrayString<249> {}
    impl ValidCapacity for ArrayString<250> {}
    impl ValidCapacity for ArrayString<251> {}
    impl ValidCapacity for ArrayString<252> {}
    impl ValidCapacity for ArrayString<253> {}
    impl ValidCapacity for ArrayString<254> {}
    impl ValidCapacity for ArrayString<255> {}
}
