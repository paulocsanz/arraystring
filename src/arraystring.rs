//! `ArrayString` definition and Api implementation

use crate::arraystring::sealed::ValidCapacity;
use crate::utils::{is_char_boundary, is_inside_boundary};
use crate::utils::{truncate_str, IntoLossy};
use crate::{prelude::*, Error};
use core::char::{decode_utf16, REPLACEMENT_CHARACTER};
use core::{cmp::min, ops::*};
#[cfg(feature = "logs")]
use log::{debug, trace};
#[cfg(all(feature = "no-panic", not(debug_assertions)))]
use no_panic::no_panic;

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
    Self: ValidCapacity,
{
    /// Creates new empty string.
    ///
    /// ```rust
    /// # use arraystring::prelude::*;
    /// # #[cfg(not(miri))] let _ = env_logger::try_init();
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
    /// # #[cfg(not(miri))] let _ = env_logger::try_init();
    /// let string = ArrayString::<23>::try_from_str("My String")?;
    /// assert_eq!(string.as_str(), "My String");
    ///
    /// assert_eq!(ArrayString::<23>::try_from_str("")?.as_str(), "");
    ///
    /// let out_of_bounds = "0".repeat(ArrayString::<23>::capacity() + 1);
    /// assert!(ArrayString::<23>::try_from_str(out_of_bounds).is_err());
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

    /// Creates new `ArrayString` from string slice truncating size if bigger than [`capacity`].
    ///
    /// [`capacity`]: ./struct.ArrayString.html#method.capacity
    ///
    /// ```rust
    /// # use arraystring::prelude::*;
    /// # #[cfg(not(miri))] let _ = env_logger::try_init();
    /// let string = ArrayString::<23>::from_str_truncate("My String");
    /// # assert_eq!(string.as_str(), "My String");
    /// println!("{}", string);
    ///
    /// let truncate = "0".repeat(ArrayString::<23>::capacity() + 1);
    /// let truncated = "0".repeat(ArrayString::<23>::capacity());
    /// let string = ArrayString::<23>::from_str_truncate(&truncate);
    /// assert_eq!(string.as_str(), truncated);
    /// ```
    #[inline]
    #[cfg_attr(all(feature = "no-panic", not(debug_assertions)), no_panic)]
    pub fn from_str_truncate(string: impl AsRef<str>) -> Self {
        trace!("FromStr truncate: {}", string.as_ref());
        let mut s = Self::new();
        s.push_str_truncate(string);
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
    #[cfg_attr(all(feature = "no-panic", not(debug_assertions)), no_panic)]
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
    /// # #[cfg(not(miri))] let _ = env_logger::try_init();
    /// let string = ArrayString::<255>::from_iterator_truncate(&["My String", " Other String"][..]);
    /// assert_eq!(string.as_str(), "My String Other String");
    ///
    /// let out_of_bounds = (0..400).map(|_| "000");
    /// let truncated = "0".repeat(ArrayString::<23>::capacity());
    ///
    /// let truncate = ArrayString::<23>::from_iterator_truncate(out_of_bounds);
    /// assert_eq!(truncate.as_str(), truncated.as_str());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[cfg_attr(all(feature = "no-panic", not(debug_assertions)), no_panic)]
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

    /// Creates new `ArrayString` from char iterator if total length is lower or equal to [`capacity`], otherwise returns an error.
    ///
    /// [`capacity`]: ./struct.ArrayString.html#method.capacity
    ///
    /// ```rust
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # #[cfg(not(miri))] let _ = env_logger::try_init();
    /// let string = ArrayString::<23>::try_from_chars("My String".chars())?;
    /// assert_eq!(string.as_str(), "My String");
    ///
    /// let out_of_bounds = "0".repeat(ArrayString::<23>::capacity() + 1);
    /// assert!(ArrayString::<23>::try_from_chars(out_of_bounds.chars()).is_err());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[cfg_attr(all(feature = "no-panic", not(debug_assertions)), no_panic)]
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
    /// # #[cfg(not(miri))] let _ = env_logger::try_init();
    /// let string = ArrayString::<23>::from_chars_truncate("My String".chars());
    /// assert_eq!(string.as_str(), "My String");
    ///
    /// let out_of_bounds = "0".repeat(ArrayString::<23>::capacity() + 1);
    /// let truncated = "0".repeat(ArrayString::<23>::capacity());
    ///
    /// let truncate = ArrayString::<23>::from_chars_truncate(out_of_bounds.chars());
    /// assert_eq!(truncate.as_str(), truncated.as_str());
    /// ```
    #[inline]
    #[cfg_attr(all(feature = "no-panic", not(debug_assertions)), no_panic)]
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

    /// Creates new `ArrayString` from `u16` slice, returning [`Utf16`] on invalid utf-16 data or [`OutOfBounds`] if bigger than [`capacity`]
    ///
    /// [`Utf16`]: ./error/enum.Error.html#variant.Utf16
    /// [`OutOfBounds`]: ./error/enum.Error.html#variant.OutOfBounds
    /// [`capacity`]: ./struct.ArrayString.html#method.capacity
    ///
    /// ```rust
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # #[cfg(not(miri))] let _ = env_logger::try_init();
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
    #[cfg_attr(all(feature = "no-panic", not(debug_assertions)), no_panic)]
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
    /// # #[cfg(not(miri))] let _ = env_logger::try_init();
    /// let music = [0xD834, 0xDD1E, 0x006d, 0x0075, 0x0073, 0x0069, 0x0063];
    /// let string = ArrayString::<23>::from_utf16_truncate(music)?;
    /// assert_eq!(string.as_str(), "𝄞music");
    ///
    /// let invalid_utf16 = [0xD834, 0xDD1E, 0x006d, 0x0075, 0xD800, 0x0069, 0x0063];
    /// assert_eq!(ArrayString::<23>::from_utf16_truncate(invalid_utf16), Err(Utf16));
    ///
    /// let out_of_bounds: Vec<u16> = (0..300).map(|_| 0).collect();
    /// assert_eq!(ArrayString::<23>::from_utf16_truncate(out_of_bounds)?.as_str(),
    ///            "\0".repeat(ArrayString::<23>::capacity()).as_str());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[cfg_attr(all(feature = "no-panic", not(debug_assertions)), no_panic)]
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
    /// # #[cfg(not(miri))] let _ = env_logger::try_init();
    /// let music = [0xD834, 0xDD1E, 0x006d, 0x0075, 0x0073, 0x0069, 0x0063];
    /// let string = ArrayString::<23>::from_utf16_lossy_truncate(music);
    /// assert_eq!(string.as_str(), "𝄞music");
    ///
    /// let invalid_utf16 = [0xD834, 0xDD1E, 0x006d, 0x0075, 0xD800, 0x0069, 0x0063];
    /// assert_eq!(ArrayString::<23>::from_utf16_lossy_truncate(invalid_utf16).as_str(), "𝄞mu\u{FFFD}ic");
    ///
    /// let out_of_bounds: Vec<u16> = (0..300).map(|_| 0).collect();
    /// assert_eq!(ArrayString::<23>::from_utf16_lossy_truncate(&out_of_bounds).as_str(),
    ///            "\0".repeat(ArrayString::<23>::capacity()).as_str());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[cfg_attr(all(feature = "no-panic", not(debug_assertions)), no_panic)]
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
    /// # #[cfg(not(miri))] let _ = env_logger::try_init();
    /// let s = ArrayString::<23>::try_from_str("My String")?;
    /// assert_eq!(s.as_str(), "My String");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[cfg_attr(all(feature = "no-panic", not(debug_assertions)), no_panic)]
    pub fn as_str(&self) -> &str {
        trace!("As str: {self}");
        self.as_ref()
    }

    /// Extracts a mutable string slice containing the entire `ArrayString`
    ///
    /// ```rust
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # #[cfg(not(miri))] let _ = env_logger::try_init();
    /// let mut s = ArrayString::<23>::try_from_str("My String")?;
    /// assert_eq!(s.as_mut_str(), "My String");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[cfg_attr(all(feature = "no-panic", not(debug_assertions)), no_panic)]
    pub fn as_mut_str(&mut self) -> &mut str {
        trace!("As mut str: {self}");
        self.as_mut()
    }

    /// Extracts a byte slice containing the entire `ArrayString`
    ///
    /// ```rust
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # #[cfg(not(miri))] let _ = env_logger::try_init();
    /// let s = ArrayString::<23>::try_from_str("My String")?;
    /// assert_eq!(s.as_bytes(), "My String".as_bytes());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[cfg_attr(all(feature = "no-panic", not(debug_assertions)), no_panic)]
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
    #[cfg_attr(all(feature = "no-panic", not(debug_assertions)), no_panic)]
    pub unsafe fn as_mut_bytes(&mut self) -> &mut [u8] {
        trace!("As mut str");
        let len = self.len();
        self.array.as_mut_slice().get_unchecked_mut(..len)
    }

    /// Returns maximum string capacity, defined at compile time, it will never change
    ///
    /// ```rust
    /// # use arraystring::prelude::*;
    /// # #[cfg(not(miri))] let _ = env_logger::try_init();
    /// assert_eq!(ArrayString::<32>::capacity(), 32);
    /// ```
    #[inline]
    pub const fn capacity() -> usize {
        N
    }

    /// Pushes string slice to the end of the `ArrayString` if total size is lower or equal to [`capacity`], otherwise returns an error.
    ///
    /// [`capacity`]: ./struct.ArrayString.html#method.capacity
    ///
    /// ```rust
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # #[cfg(not(miri))] let _ = env_logger::try_init();
    /// let mut s = ArrayString::<255>::try_from_str("My String")?;
    /// s.try_push_str(" My other String")?;
    /// assert_eq!(s.as_str(), "My String My other String");
    ///
    /// assert!(s.try_push_str("0".repeat(ArrayString::<255>::capacity())).is_err());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[cfg_attr(all(feature = "no-panic", not(debug_assertions)), no_panic)]
    pub fn try_push_str(&mut self, string: impl AsRef<str>) -> Result<(), OutOfBounds> {
        trace!("Push str: {}", string.as_ref());
        let str = string.as_ref();
        if str.is_empty() {
            return Ok(());
        }
        is_inside_boundary(str.len() + self.len(), Self::capacity())?;
        unsafe { self.push_str_unchecked(str.as_bytes()) };
        Ok(())
    }

    /// Pushes string slice to the end of the `ArrayString` truncating total size if bigger than [`capacity`].
    ///
    /// [`capacity`]: ./struct.ArrayString.html#method.capacity
    ///
    /// ```rust
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # #[cfg(not(miri))] let _ = env_logger::try_init();
    /// let mut s = ArrayString::<255>::try_from_str("My String")?;
    /// s.push_str_truncate(" My other String");
    /// assert_eq!(s.as_str(), "My String My other String");
    ///
    /// let mut s = ArrayString::<23>::default();
    /// s.push_str_truncate("0".repeat(ArrayString::<23>::capacity() + 1));
    /// assert_eq!(s.as_str(), "0".repeat(ArrayString::<23>::capacity()).as_str());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[cfg_attr(all(feature = "no-panic", not(debug_assertions)), no_panic)]
    pub fn push_str_truncate(&mut self, string: impl AsRef<str>) {
        trace!("Push str truncate: {}", string.as_ref());
        let str = string.as_ref().as_bytes();
        let size = Self::capacity().saturating_sub(self.len());
        if size == 0 || str.is_empty() {
            return;
        }
        let str = truncate_str(str, size);
        unsafe { self.push_str_unchecked(str) };
    }

    #[inline]
    unsafe fn push_str_unchecked(&mut self, bytes: &[u8]) {
        core::ptr::copy_nonoverlapping(
            bytes.as_ptr(),
            self.array.as_mut_ptr().add(self.len()),
            bytes.len(),
        );
        self.size += bytes.len().into_lossy();
    }

    /// Inserts character to the end of the `ArrayString` erroring if total size if bigger than [`capacity`].
    ///
    /// [`capacity`]: ./struct.ArrayString.html#method.capacity
    ///
    /// ```rust
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # #[cfg(not(miri))] let _ = env_logger::try_init();
    /// let mut s = ArrayString::<23>::try_from_str("My String")?;
    /// s.try_push('!')?;
    /// assert_eq!(s.as_str(), "My String!");
    ///
    /// let mut s = ArrayString::<23>::try_from_str(&"0".repeat(ArrayString::<23>::capacity()))?;
    /// assert!(s.try_push('!').is_err());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[cfg_attr(all(feature = "no-panic", not(debug_assertions)), no_panic)]
    pub fn try_push(&mut self, ch: char) -> Result<(), OutOfBounds> {
        trace!("Push: {}", ch);
        let mut buf = [0; 4];
        self.try_push_str(ch.encode_utf8(&mut buf))
    }

    /// Truncates `ArrayString` to specified size (if smaller than current size and a valid utf-8 char index).
    ///
    /// ```rust
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # #[cfg(not(miri))] let _ = env_logger::try_init();
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
    #[cfg_attr(all(feature = "no-panic", not(debug_assertions)), no_panic)]
    pub fn truncate(&mut self, size: usize) -> Result<(), Utf8> {
        debug!("Truncate: {}", size);
        let len = min(self.len(), size);
        is_char_boundary(self, len).map(|()| self.size = len.into_lossy())
    }

    /// Removes last character from `ArrayString`, if any.
    ///
    /// ```rust
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # #[cfg(not(miri))] let _ = env_logger::try_init();
    /// let mut s = ArrayString::<23>::try_from_str("A🤔")?;
    /// assert_eq!(s.pop(), Some('🤔'));
    /// assert_eq!(s.pop(), Some('A'));
    /// assert_eq!(s.pop(), None);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[cfg_attr(all(feature = "no-panic", not(debug_assertions)), no_panic)]
    pub fn pop(&mut self) -> Option<char> {
        debug!("Pop");
        self.as_str().chars().last().map(|ch| {
            self.size = self.size.saturating_sub(ch.len_utf8().into_lossy());
            ch
        })
    }

    /// Removes whitespaces from the beggining and end of the string
    ///
    /// ```rust
    /// # use arraystring::prelude::*;
    /// # fn main() -> Result<(), OutOfBounds> {
    /// # #[cfg(not(miri))] let _ = env_logger::try_init();
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
    #[cfg_attr(all(feature = "no-panic", not(debug_assertions)), no_panic)]
    pub fn trim(&mut self) {
        trace!("Trim");
        let mut start = self.len();
        for (pos, char) in self.as_str().char_indices() {
            if !char.is_whitespace() {
                start = pos;
                break;
            }
        }
        let mut end = self.len();
        for (pos, char) in self.as_str().char_indices().rev() {
            if pos < start {
                self.size = 0;
                return;
            }
            if !char.is_whitespace() {
                break;
            }
            end = pos;
        }

        self.size = end.saturating_sub(start).into_lossy();

        unsafe {
            let ptr = self.array.as_mut_ptr();
            core::ptr::copy(ptr.add(start), ptr, self.len());
        }
    }

    /// Removes specified char from `ArrayString`
    ///
    /// ```rust
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # #[cfg(not(miri))] let _ = env_logger::try_init();
    /// let mut s = ArrayString::<23>::try_from_str("ABCD🤔")?;
    /// assert_eq!(s.remove("ABCD🤔".len()), Err(Error::OutOfBounds));
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
    #[cfg_attr(all(feature = "no-panic", not(debug_assertions)), no_panic)]
    pub fn remove(&mut self, idx: usize) -> Result<char, Error> {
        debug!("Remove: {}", idx);
        is_inside_boundary(idx, self.len())?;
        is_char_boundary(self, idx)?;
        let char = unsafe {
            self.as_str()
                .get_unchecked(idx..)
                .chars()
                .next()
                .ok_or(OutOfBounds)?
        };
        let len = char.len_utf8();
        if idx + len < self.len() {
            self.array.copy_within(idx + len.., idx);
        }
        self.size -= len.into_lossy();
        Ok(char)
    }

    /// Retains only the characters specified by the predicate.
    ///
    /// ```rust
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # #[cfg(not(miri))] let _ = env_logger::try_init();
    /// let mut s = ArrayString::<23>::try_from_str("ABCD🤔")?;
    /// s.retain(|c| c != '🤔');
    /// assert_eq!(s.as_str(), "ABCD");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[cfg_attr(all(feature = "no-panic", not(debug_assertions)), no_panic)]
    pub fn retain(&mut self, mut f: impl FnMut(char) -> bool) {
        trace!("Retain");
        // Not the most efficient solution, we could shift left during batch mismatch
        *self = Self::from_chars_truncate(self.as_str().chars().filter(|c| f(*c)));
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
    /// # #[cfg(not(miri))] let _ = env_logger::try_init();
    /// let mut s = ArrayString::<23>::try_from_str("ABCD🤔")?;
    /// s.try_insert(1, 'A')?;
    /// s.try_insert(2, 'B')?;
    /// assert_eq!(s.as_str(), "AABBCD🤔");
    /// assert_eq!(s.try_insert(20, 'C'), Err(Error::OutOfBounds));
    /// assert_eq!(s.try_insert(8, 'D'), Err(Error::Utf8));
    ///
    /// let mut s = ArrayString::<23>::try_from_str(&"0".repeat(ArrayString::<23>::capacity()))?;
    /// assert_eq!(s.try_insert(0, 'C'), Err(Error::OutOfBounds));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[cfg_attr(all(feature = "no-panic", not(debug_assertions)), no_panic)]
    pub fn try_insert(&mut self, idx: usize, ch: char) -> Result<(), Error> {
        let mut buf = [0; 4];
        self.try_insert_str(idx, ch.encode_utf8(&mut buf))
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
    /// # #[cfg(not(miri))] let _ = env_logger::try_init();
    /// let mut s = ArrayString::<23>::try_from_str("ABCD🤔")?;
    /// s.try_insert_str(1, "AB")?;
    /// s.try_insert_str(1, "BC")?;
    /// assert_eq!(s.try_insert_str(1, "0".repeat(ArrayString::<23>::capacity())),
    ///            Err(Error::OutOfBounds));
    /// assert_eq!(s.as_str(), "ABCABBCD🤔");
    /// assert_eq!(s.try_insert_str(20, "C"), Err(Error::OutOfBounds));
    /// assert_eq!(s.try_insert_str(10, "D"), Err(Error::Utf8));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[cfg_attr(all(feature = "no-panic", not(debug_assertions)), no_panic)]
    pub fn try_insert_str(&mut self, idx: usize, string: impl AsRef<str>) -> Result<(), Error> {
        trace!("Try insert at {idx} str: {}", string.as_ref());
        let str = string.as_ref().as_bytes();
        is_inside_boundary(idx, self.len())?;
        is_inside_boundary(idx + str.len() + self.len(), Self::capacity())?;
        is_char_boundary(self, idx)?;
        if str.is_empty() {
            return Ok(());
        }

        unsafe {
            let ptr = self.array.as_mut_ptr().add(idx);
            core::ptr::copy(ptr, ptr.add(str.len()), self.len().saturating_sub(idx));
            core::ptr::copy(str.as_ptr(), ptr, str.len());
        }
        self.size = self.len().saturating_add(str.len()).into_lossy();

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
    /// # #[cfg(not(miri))] let _ = env_logger::try_init();
    /// let mut s = ArrayString::<23>::try_from_str("ABCD🤔")?;
    /// s.insert_str_truncate(1, "AB")?;
    /// s.insert_str_truncate(1, "BC")?;
    /// assert_eq!(s.as_str(), "ABCABBCD🤔");
    ///
    /// assert_eq!(s.insert_str_truncate(20, "C"), Err(Error::OutOfBounds));
    /// assert_eq!(s.insert_str_truncate(10, "D"), Err(Error::Utf8));
    ///
    /// s.clear();
    /// s.insert_str_truncate(0, "0".repeat(ArrayString::<23>::capacity() + 10))?;
    /// assert_eq!(s.as_str(), "0".repeat(ArrayString::<23>::capacity()).as_str());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[cfg_attr(all(feature = "no-panic", not(debug_assertions)), no_panic)]
    pub fn insert_str_truncate(
        &mut self,
        idx: usize,
        string: impl AsRef<str>,
    ) -> Result<(), Error> {
        trace!("Insert str at {idx}: {}", string.as_ref());
        is_inside_boundary(idx, self.len())?;
        is_char_boundary(self, idx)?;
        let size = Self::capacity().saturating_sub(idx);
        if size == 0 {
            return Ok(());
        }
        let str = truncate_str(string.as_ref().as_bytes(), size);
        if str.is_empty() {
            return Ok(());
        }

        unsafe {
            let ptr = self.array.as_mut_ptr().add(idx);
            core::ptr::copy(ptr, ptr.add(str.len()), self.len().saturating_sub(idx));
            core::ptr::copy(str.as_ptr(), ptr, str.len());
        }
        self.size = self.len().saturating_add(str.len()).into_lossy();
        Ok(())
    }

    /// Returns `ArrayString` length.
    ///
    /// ```rust
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # #[cfg(not(miri))] let _ = env_logger::try_init();
    /// let mut s = ArrayString::<23>::try_from_str("ABCD")?;
    /// assert_eq!(s.len(), 4);
    /// s.try_push('🤔')?;
    /// // Emojis use 4 bytes (this is the default rust behavior, length of u8)
    /// assert_eq!(s.len(), 8);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[cfg_attr(all(feature = "no-panic", not(debug_assertions)), no_panic)]
    pub fn len(&self) -> usize {
        trace!("Len");
        self.size as usize
    }

    /// Checks if `ArrayString` is empty.
    ///
    /// ```rust
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # #[cfg(not(miri))] let _ = env_logger::try_init();
    /// let mut s = ArrayString::<23>::try_from_str("ABCD")?;
    /// assert!(!s.is_empty());
    /// s.clear();
    /// assert!(s.is_empty());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[cfg_attr(all(feature = "no-panic", not(debug_assertions)), no_panic)]
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
    /// # #[cfg(not(miri))] let _ = env_logger::try_init();
    /// let mut s = ArrayString::<23>::try_from_str("AB🤔CD")?;
    /// assert_eq!(s.split_off(6)?.as_str(), "CD");
    /// assert_eq!(s.as_str(), "AB🤔");
    /// assert_eq!(s.split_off(20), Err(Error::OutOfBounds));
    /// assert_eq!(s.split_off(4), Err(Error::Utf8));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[cfg_attr(all(feature = "no-panic", not(debug_assertions)), no_panic)]
    pub fn split_off(&mut self, at: usize) -> Result<Self, Error> {
        debug!("Split off");
        is_inside_boundary(at, self.len())?;
        is_char_boundary(self, at)?;
        debug_assert!(at <= self.len() && self.as_str().is_char_boundary(at));
        let new =
            unsafe { Self::try_from_str(self.as_str().get_unchecked(at..)).unwrap_unchecked() };
        self.size = at.into_lossy();
        Ok(new)
    }

    /// Empties `ArrayString`
    ///
    /// ```rust
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # #[cfg(not(miri))] let _ = env_logger::try_init();
    /// let mut s = ArrayString::<23>::try_from_str("ABCD")?;
    /// assert!(!s.is_empty());
    /// s.clear();
    /// assert!(s.is_empty());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[cfg_attr(all(feature = "no-panic", not(debug_assertions)), no_panic)]
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
    /// # #[cfg(not(miri))] let _ = env_logger::try_init();
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
    #[cfg_attr(all(feature = "no-panic", not(debug_assertions)), no_panic)]
    pub fn drain(&mut self, range: impl RangeBounds<usize>) -> Result<Drain<N>, Error> {
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
        debug_assert!(self.as_str().is_char_boundary(start));
        debug_assert!(self.as_str().is_char_boundary(end));

        let drain = unsafe {
            let slice = self.as_str().get_unchecked(start..end);
            Self::try_from_str(slice).unwrap_unchecked()
        };
        self.array.copy_within(end.., start);
        self.size = self
            .size
            .saturating_sub(end.saturating_sub(start).into_lossy());
        Ok(Drain(drain))
    }

    /// Removes the specified range of the `ArrayString`, and replaces it with the given string. The given string doesn't need to have the same length as the range.
    ///
    /// ```rust
    /// # use arraystring::{Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # #[cfg(not(miri))] let _ = env_logger::try_init();
    /// let mut s = ArrayString::<23>::try_from_str("ABCD🤔")?;
    /// s.replace_range(2..4, "EFGHI")?;
    /// assert_eq!(s.as_bytes(), "ABEFGHI🤔".as_bytes());
    ///
    /// assert_eq!(s.replace_range(9.., "J"), Err(Error::Utf8));
    /// assert_eq!(s.replace_range(..90, "K"), Err(Error::OutOfBounds));
    /// assert_eq!(s.replace_range(0..1, "0".repeat(ArrayString::<23>::capacity())), Err(Error::OutOfBounds));
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
        let str = with.as_ref().as_bytes();
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
            with.as_ref().len(),
            with.as_ref()
        );
        if start == end && str.is_empty() {
            return Ok(());
        }
        is_inside_boundary(start, end)?;
        is_inside_boundary(end, self.len())?;
        is_char_boundary(self, start)?;
        is_char_boundary(self, end)?;
        is_inside_boundary(
            (start + str.len() + self.len()).saturating_sub(end),
            Self::capacity(),
        )?;

        let this_len = self.len();
        unsafe {
            let ptr = self.array.as_mut_ptr();
            core::ptr::copy(ptr.add(end), ptr.add(str.len().saturating_add(start)), this_len.saturating_sub(end));
            core::ptr::copy(str.as_ptr(), ptr.add(start), str.len());
        }
        self.size = self.len().saturating_add(str.len()).saturating_add(start).saturating_sub(end).into_lossy();
        Ok(())
    }
}

/// Temporary hack until const generics constraints are stable
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
