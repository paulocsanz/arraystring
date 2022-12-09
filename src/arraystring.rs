//! `ArrayString` definition and Api implementation

use core::ops::*;

#[cfg(feature = "diesel-traits")]
use diesel::{AsExpression, FromSqlRow};

use crate::{error::Error, prelude::*};
use crate::utils::{is_char_boundary, is_inside_boundary};
use crate::utils::shift_left_unchecked;

/// String based on a generic array (size defined at compile time through `typenum`)
///
/// Can't outgrow capacity (defined at compile time), always occupies [`capacity`] `+ 1` bytes of memory
///
/// *Doesn't allocate memory on the heap and never panics (all panic branches are stripped at compile time)*
///
/// [`capacity`]: ./struct.ArrayString.html#method.capacity
#[derive(Copy, Clone)]
#[cfg_attr(feature = "diesel-traits", derive(AsExpression, FromSqlRow))]
#[cfg_attr(feature = "diesel-traits", diesel(sql_type = diesel::sql_types::Text))]
pub struct ArrayString<const N: usize> {
    /// Array type corresponding to specified `SIZE`
    pub(crate) array: [u8; N],
    /// Current string size
    pub(crate) size: u8,
}

impl<const N: usize> ArrayString<N> {
    /// Creates new empty string.
    ///
    /// ```rust
    /// # use arraystring::prelude::*;
    /// # let _ = env_logger::try_init();
    /// let string = SmallString::new();
    /// assert!(string.is_empty());
    /// ```
    #[inline]
    pub const fn new() -> Self {
        // trace!("New empty ArrayString"); <-- why sacrifice consts for traces...
        Self {
            array: [0; N],
            size: 0,
        }
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
        if N > u8::MAX as usize {
            #[cfg(debug_assertions)] panic!("ArrayString<N> cannot be larger than ArrayString<255>");
            #[cfg(not(debug_assertions))] u8::MAX
        } else {
            N as u8
        }
    }

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
    #[inline]
    pub const fn len(&self) -> u8 {
        self.size
    }

    /// Creates a draining iterator that removes the specified range in the `ArrayString` and yields the removed chars.
    ///
    /// Note: The element range is removed even if the iterator is not consumed until the end.
    ///
    /// ```rust
    /// # use arraystring::{error::Error, prelude::*};
    /// # fn main() -> Result<(), Error> {
    /// # let _ = env_logger::try_init();
    /// let mut s = SmallString::try_from_str("ABCD🤔")?;
    /// assert_eq!(s.drain(..3)?.collect::<Vec<_>>(), vec!['A', 'B', 'C']);
    /// assert_eq!(s.as_str(), "D🤔");
    ///
    /// assert_eq!(s.drain(3..), Err(Error::Utf8));
    /// assert_eq!(s.drain(10..), Err(Error::OutOfBounds));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn drain<R>(&mut self, range: R) -> Result<Drain<N>, Error>
    where
        R: RangeBounds<u8>,
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
        Ok(Drain(drain, 0))
    }
}

impl<const N: usize> ArrayStringBase for ArrayString<N> {

    #[inline]
    fn len(&self) -> u8 {
        self.len()
    }

    #[inline]
    fn capacity() -> u8 {
        Self::capacity()
    }

    #[inline]
    fn new() -> Self {
        Self::new()
    }

    #[inline]
    unsafe fn raw_bytes(&self) -> &[u8] {
        &self.array
    }

    #[inline]
    unsafe fn raw_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.array
    }

    #[inline]
    unsafe fn set_len_unchecked(&mut self, len: u8) {
        self.size = len;
    }
}
