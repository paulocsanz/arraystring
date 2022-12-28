//! Fixed capacity stack based generic string
//!
//! Can't outgrow initial capacity (defined at compile time), always occupies [`capacity`] `+ 1` bytes of memory
//!
//! *Maximum Capacity is 255*
//!
//! *Doesn't allocate memory on the heap and should never panic in release (except in `Index`/`IndexMut` traits, since they are supposed to)*
//!
//! *The no panic garantee can be ensured at compilation time with the `no-panic` feature, just be aware that a compiler update might break this garantee, therefore making the crate uncompilable, open an issue if you notice.*
//!
//! ## Why
//!
//! Data is generally bounded, you don't want a phone number with 30 characters, nor a username with 100. You probably don't even support it in your database.
//!
//! Why pay the cost of heap allocations of strings with unlimited capacity if you have limited boundaries?
//!
//! Stack based strings are generally faster to create, clone and append to than heap based strings (custom allocators and thread-locals may help with heap based ones).
//!
//! But that becomes less true as you increase the array size, [`CacheString`] occupies a full cache line and 255 bytes is the maximum we accept ([`MaxString`] and it's probably already slower than heap based strings of that size - like in `std::string::String`)
//!
//! There are other stack based strings out there, they generally don't use stable const generics and a lot of them only support stack based strings in the context of small string optimizations.
//!
//! Be aware that array based strings always occupy the full space in memory, so they may use more memory (although in the stack) than dynamic strings.
//!
//! [`capacity`]: ./struct.ArrayString.html#method.capacity
//! [`MaxString`]: ./type.MaxString.html
//!
//! ## Features
//!
//! **default:** `std`
//!
//! - `std` enabled by default, enables `std` compatibility, implementing std only traits (disable it to be `#[no_std]` compatible)
//! - `serde-traits` enables serde traits integration (`Serialize`/`Deserialize`)
//!
//!     Opperates like `String`, but truncates it if it's bigger than capacity
//!
//! - `diesel-traits` enables diesel 2.0 traits integration
//!
//!      Opperates like `String`, but truncates it if it's bigger than capacity
//!
//! - `no-panic` checks at compile time that the panic function is not linked by the library
//!
//!      Be careful before using this, it won't change functions behaviors, it will just enforce that panic functions can't be linked by this library. This may break your compilation and won't improve the safety of this library. It's mostly for testing and environments where if the non panicking invariantcan't be garanteed compilation should fail. This should not apply to most projects.
//!
//!      Only works when all optimizations are enabled, and may break in future compiler updates. Please open an issue if you notice.
//!
//! - `logs` enables internal logging
//!
//!     You will probably only need this if you are debugging this library
//!
//! ## Examples
//!
//! ```rust
//! use arraystring::{Error, ArrayString};
//!
//! type Username = ArrayString<20>;
//! type Role = ArrayString<5>;
//!
//! #[derive(Debug)]
//! pub struct User {
//!     pub username: Username,
//!     pub role: Role,
//! }
//!
//! fn main() -> Result<(), Error> {
//!     let user = User {
//!         username: Username::try_from_str("user")?,
//!         role: Role::try_from_str("admin")?
//!     };
//!     println!("{:?}", user);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Licenses
//!
//! `MIT` and `Apache-2.0`

#![doc(html_root_url = "https://docs.rs/arraystring/0.3.0/arraystring")]
#![cfg_attr(docs_rs_workaround, feature(doc_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]
#![warn(
    missing_docs,
    missing_debug_implementations,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results,
    bad_style,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    unconditional_recursion,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true
)]
#![doc(test(attr(deny(warnings))))]

/// Remove logging macros when they are disabled (at compile time)
#[macro_use]
#[cfg(not(feature = "logs"))]
#[allow(unused)]
mod mock {
    macro_rules! trace(($($x:tt)*) => ());
    macro_rules! debug(($($x:tt)*) => ());
    macro_rules! info(($($x:tt)*) => ());
    macro_rules! warn(($($x:tt)*) => ());
    macro_rules! error(($($x:tt)*) => ());
}

mod arraystring;
mod drain;
pub mod error;
mod implementations;
#[cfg(any(feature = "serde-traits", feature = "diesel-traits"))]
mod integration;
#[doc(hidden)]
pub mod utils;

/// All structs defined by this crate
pub mod prelude {
    pub use crate::arraystring::ArrayString;
    pub use crate::drain::Drain;
    pub use crate::error::{OutOfBounds, Utf16, Utf8};
    pub use crate::{CacheString, MaxString, SmallString};
}

pub use crate::arraystring::ArrayString;
pub use crate::drain::Drain;
pub use crate::error::Error;

/// String with the same `core::mem::size_of` of a `String` (`core::mem::size_of::<usize> * 3`)
///
/// 24 bytes in 64 bits architecture
///
/// 12 bytes in 32 bits architecture
pub type SmallString = ArrayString<{ core::mem::size_of::<usize>() * 3 }>;

/// Biggest `ArrayString<N>` supported (255 bytes of text)
pub type MaxString = ArrayString<255>;

mod cache_string {
    use crate::{prelude::*, Error};
    use core::fmt::{self, Debug, Display, Formatter, Write};
    use core::{borrow::Borrow, borrow::BorrowMut, ops::*};
    use core::{cmp::Ordering, hash::Hash, hash::Hasher, str::FromStr};

    const CACHE_STRING_SIZE: usize = 63;
    /// Newtype string that occupies 64 bytes in memory and is 64 bytes aligned (full cache line)
    ///
    /// 63 bytes of text
    #[repr(align(64))]
    #[derive(Copy, Clone, Default)]
    #[cfg_attr(
        feature = "diesel-traits",
        derive(diesel::AsExpression, diesel::FromSqlRow)
    )]
    #[cfg_attr(feature = "diesel-traits", diesel(sql_type = diesel::sql_types::Text))]
    pub struct CacheString(pub(crate) ArrayString<CACHE_STRING_SIZE>);

    impl CacheString {
        /// Creates new empty `CacheString`.
        ///
        /// ```rust
        /// # use arraystring::prelude::*;
        /// # #[cfg(not(miri))] let _ = env_logger::try_init();
        /// let string = CacheString::new();
        /// assert!(string.is_empty());
        /// ```
        #[inline]
        pub const fn new() -> Self {
            Self(ArrayString::<CACHE_STRING_SIZE>::new())
        }

        /// Creates new `CacheString` from string slice if length is lower or equal to [`capacity`], otherwise returns an error.
        ///
        /// [`capacity`]: ./struct.CacheString.html#method.capacity
        /// ```rust
        /// # use arraystring::{Error, prelude::*};
        /// # fn main() -> Result<(), Error> {
        /// # #[cfg(not(miri))] let _ = env_logger::try_init();
        /// let string = CacheString::try_from_str("My String")?;
        /// assert_eq!(string.as_str(), "My String");
        ///
        /// assert_eq!(CacheString::try_from_str("")?.as_str(), "");
        ///
        /// let out_of_bounds = "0".repeat(CacheString::capacity() + 1);
        /// assert!(CacheString::try_from_str(out_of_bounds).is_err());
        /// # Ok(())
        /// # }
        /// ```
        #[inline]
        pub fn try_from_str(s: impl AsRef<str>) -> Result<Self, OutOfBounds> {
            Ok(Self(ArrayString::try_from_str(s)?))
        }

        /// Creates new `CacheString` from string slice truncating size if bigger than [`capacity`].
        ///
        /// [`capacity`]: ./struct.CacheString.html#method.capacity
        ///
        /// ```rust
        /// # use arraystring::prelude::*;
        /// # #[cfg(not(miri))] let _ = env_logger::try_init();
        /// let string = CacheString::from_str_truncate("My String");
        /// # assert_eq!(string.as_str(), "My String");
        /// println!("{}", string);
        ///
        /// let truncate = "0".repeat(CacheString::capacity() + 1);
        /// let truncated = "0".repeat(CacheString::capacity());
        /// let string = CacheString::from_str_truncate(&truncate);
        /// assert_eq!(string.as_str(), truncated);
        /// ```
        #[inline]
        pub fn from_str_truncate(string: impl AsRef<str>) -> Self {
            Self(ArrayString::from_str_truncate(string))
        }

        /// Creates new `CacheString` from string slice iterator if total length is lower or equal to [`capacity`], otherwise returns an error.
        ///
        /// [`capacity`]: ./struct.CacheString.html#method.capacity
        ///
        /// ```rust
        /// # use arraystring::prelude::*;
        /// # fn main() -> Result<(), OutOfBounds> {
        /// let string = CacheString::try_from_iterator(&["My String", " My Other String"][..])?;
        /// assert_eq!(string.as_str(), "My String My Other String");
        ///
        /// let out_of_bounds = (0..100).map(|_| "000");
        /// assert!(CacheString::try_from_iterator(out_of_bounds).is_err());
        /// # Ok(())
        /// # }
        /// ```
        #[inline]
        pub fn try_from_iterator(
            iter: impl IntoIterator<Item = impl AsRef<str>>,
        ) -> Result<Self, OutOfBounds> {
            Ok(Self(ArrayString::try_from_iterator(iter)?))
        }

        /// Creates new `CacheString` from string slice iterator truncating size if bigger than [`capacity`].
        ///
        /// [`capacity`]: ./struct.CacheString.html#method.capacity
        ///
        /// ```rust
        /// # use arraystring::prelude::*;
        /// # fn main() -> Result<(), OutOfBounds> {
        /// # #[cfg(not(miri))] let _ = env_logger::try_init();
        /// let string = CacheString::from_iterator_truncate(&["My String", " Other String"][..]);
        /// assert_eq!(string.as_str(), "My String Other String");
        ///
        /// let out_of_bounds = (0..400).map(|_| "000");
        /// let truncated = "0".repeat(CacheString::capacity());
        ///
        /// let truncate = CacheString::from_iterator_truncate(out_of_bounds);
        /// assert_eq!(truncate.as_str(), truncated.as_str());
        /// # Ok(())
        /// # }
        /// ```
        #[inline]
        pub fn from_iterator_truncate(iter: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
            Self(ArrayString::from_iterator_truncate(iter))
        }

        /// Creates new `CacheString` from char iterator if total length is lower or equal to [`capacity`], otherwise returns an error.
        ///
        /// [`capacity`]: ./struct.CacheString.html#method.capacity
        ///
        /// ```rust
        /// # use arraystring::{Error, prelude::*};
        /// # fn main() -> Result<(), Error> {
        /// # #[cfg(not(miri))] let _ = env_logger::try_init();
        /// let string = CacheString::try_from_chars("My String".chars())?;
        /// assert_eq!(string.as_str(), "My String");
        ///
        /// let out_of_bounds = "0".repeat(CacheString::capacity() + 1);
        /// assert!(CacheString::try_from_chars(out_of_bounds.chars()).is_err());
        /// # Ok(())
        /// # }
        /// ```
        #[inline]
        pub fn try_from_chars(iter: impl IntoIterator<Item = char>) -> Result<Self, OutOfBounds> {
            Ok(Self(ArrayString::try_from_chars(iter)?))
        }

        /// Creates new `CacheString` from char iterator truncating size if bigger than [`capacity`].
        ///
        /// [`capacity`]: ./struct.CacheString.html#method.capacity
        ///
        /// ```rust
        /// # use arraystring::prelude::*;
        /// # #[cfg(not(miri))] let _ = env_logger::try_init();
        /// let string = CacheString::from_chars_truncate("My String".chars());
        /// assert_eq!(string.as_str(), "My String");
        ///
        /// let out_of_bounds = "0".repeat(CacheString::capacity() + 1);
        /// let truncated = "0".repeat(CacheString::capacity());
        ///
        /// let truncate = CacheString::from_chars_truncate(out_of_bounds.chars());
        /// assert_eq!(truncate.as_str(), truncated.as_str());
        /// ```
        #[inline]
        pub fn from_chars_truncate(iter: impl IntoIterator<Item = char>) -> Self {
            Self(ArrayString::from_chars_truncate(iter))
        }

        /// Creates new `CacheString` from `u16` slice, returning [`Utf16`] on invalid utf-16 data or [`OutOfBounds`] if bigger than [`capacity`]
        ///
        /// [`Utf16`]: ./error/enum.Error.html#variant.Utf16
        /// [`OutOfBounds`]: ./error/enum.Error.html#variant.OutOfBounds
        /// [`capacity`]: ./struct.CacheString.html#method.capacity
        ///
        /// ```rust
        /// # use arraystring::{Error, prelude::*};
        /// # fn main() -> Result<(), Error> {
        /// # #[cfg(not(miri))] let _ = env_logger::try_init();
        /// let music = [0xD834, 0xDD1E, 0x006d, 0x0075, 0x0073, 0x0069, 0x0063];
        /// let string = CacheString::try_from_utf16(music)?;
        /// assert_eq!(string.as_str(), "𝄞music");
        ///
        /// let invalid_utf16 = [0xD834, 0xDD1E, 0x006d, 0x0075, 0xD800, 0x0069, 0x0063];
        /// assert_eq!(CacheString::try_from_utf16(invalid_utf16), Err(Error::Utf16));
        ///
        /// let out_of_bounds: Vec<_> = (0..300).map(|_| 0).collect();
        /// assert_eq!(CacheString::try_from_utf16(out_of_bounds), Err(Error::OutOfBounds));
        /// # Ok(())
        /// # }
        /// ```
        #[inline]
        pub fn try_from_utf16(slice: impl AsRef<[u16]>) -> Result<Self, Error> {
            Ok(Self(ArrayString::try_from_utf16(slice)?))
        }

        /// Creates new `CacheString` from `u16` slice, returning [`Utf16`] on invalid utf-16 data, truncating if bigger than [`capacity`].
        ///
        /// [`Utf16`]: ./error/struct.Utf16.html
        /// [`capacity`]: ./struct.CacheString.html#method.capacity
        ///
        /// ```rust
        /// # use arraystring::{Error, prelude::*};
        /// # fn main() -> Result<(), Error> {
        /// # #[cfg(not(miri))] let _ = env_logger::try_init();
        /// let music = [0xD834, 0xDD1E, 0x006d, 0x0075, 0x0073, 0x0069, 0x0063];
        /// let string = CacheString::from_utf16_truncate(music)?;
        /// assert_eq!(string.as_str(), "𝄞music");
        ///
        /// let invalid_utf16 = [0xD834, 0xDD1E, 0x006d, 0x0075, 0xD800, 0x0069, 0x0063];
        /// assert_eq!(CacheString::from_utf16_truncate(invalid_utf16), Err(Utf16));
        ///
        /// let out_of_bounds: Vec<u16> = (0..300).map(|_| 0).collect();
        /// assert_eq!(CacheString::from_utf16_truncate(out_of_bounds)?.as_str(),
        ///            "\0".repeat(CacheString::capacity()).as_str());
        /// # Ok(())
        /// # }
        /// ```
        #[inline]
        pub fn from_utf16_truncate(slice: impl AsRef<[u16]>) -> Result<Self, Utf16> {
            Ok(Self(ArrayString::from_utf16_truncate(slice)?))
        }

        /// Creates new `CacheString` from `u16` slice, replacing invalid utf-16 data with `REPLACEMENT_CHARACTER` (\u{FFFD}) and truncating size if bigger than [`capacity`]
        ///
        /// [`capacity`]: ./struct.CacheString.html#method.capacity
        ///
        /// ```rust
        /// # use arraystring::{Error, prelude::*};
        /// # fn main() -> Result<(), Error> {
        /// # #[cfg(not(miri))] let _ = env_logger::try_init();
        /// let music = [0xD834, 0xDD1E, 0x006d, 0x0075, 0x0073, 0x0069, 0x0063];
        /// let string = CacheString::from_utf16_lossy_truncate(music);
        /// assert_eq!(string.as_str(), "𝄞music");
        ///
        /// let invalid_utf16 = [0xD834, 0xDD1E, 0x006d, 0x0075, 0xD800, 0x0069, 0x0063];
        /// assert_eq!(CacheString::from_utf16_lossy_truncate(invalid_utf16).as_str(), "𝄞mu\u{FFFD}ic");
        ///
        /// let out_of_bounds: Vec<u16> = (0..300).map(|_| 0).collect();
        /// assert_eq!(CacheString::from_utf16_lossy_truncate(&out_of_bounds).as_str(),
        ///            "\0".repeat(CacheString::capacity()).as_str());
        /// # Ok(())
        /// # }
        /// ```
        #[inline]
        pub fn from_utf16_lossy_truncate(slice: impl AsRef<[u16]>) -> Self {
            Self(ArrayString::from_utf16_lossy_truncate(slice))
        }

        /// Returns maximum string capacity, defined at compile time, it will never change
        ///
        /// Should always return 63 bytes
        ///
        /// ```rust
        /// # use arraystring::prelude::*;
        /// # #[cfg(not(miri))] let _ = env_logger::try_init();
        /// assert_eq!(CacheString::capacity(), 63);
        /// ```
        #[inline]
        pub const fn capacity() -> usize {
            CACHE_STRING_SIZE
        }

        /// Splits `CacheString` in two if `at` is smaller than `self.len()`.
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
        /// let mut s = CacheString::try_from_str("AB🤔CD")?;
        /// assert_eq!(s.split_off(6)?.as_str(), "CD");
        /// assert_eq!(s.as_str(), "AB🤔");
        /// assert_eq!(s.split_off(20), Err(Error::OutOfBounds));
        /// assert_eq!(s.split_off(4), Err(Error::Utf8));
        /// # Ok(())
        /// # }
        /// ```
        #[inline]
        pub fn split_off(&mut self, at: usize) -> Result<Self, Error> {
            Ok(Self(self.0.split_off(at)?))
        }
    }

    impl Debug for CacheString {
        #[inline]
        fn fmt(&self, f: &mut Formatter) -> fmt::Result {
            f.debug_tuple("CacheString").field(&self.0).finish()
        }
    }

    impl Hash for CacheString {
        #[inline]
        fn hash<H: Hasher>(&self, hasher: &mut H) {
            self.0.hash(hasher);
        }
    }

    impl PartialEq for CacheString {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.0.eq(&other.0)
        }
    }
    impl Eq for CacheString {}

    impl Ord for CacheString {
        #[inline]
        fn cmp(&self, other: &Self) -> Ordering {
            self.0.cmp(&other.0)
        }
    }

    impl PartialOrd for CacheString {
        #[inline]
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Deref for CacheString {
        type Target = ArrayString<CACHE_STRING_SIZE>;

        #[inline]
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl DerefMut for CacheString {
        #[inline]
        fn deref_mut(&mut self) -> &mut ArrayString<CACHE_STRING_SIZE> {
            &mut self.0
        }
    }

    impl FromIterator<char> for CacheString {
        fn from_iter<I: IntoIterator<Item = char>>(iter: I) -> Self {
            Self(ArrayString::<CACHE_STRING_SIZE>::from_iter(iter))
        }
    }

    impl<'a> FromIterator<&'a str> for CacheString {
        fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> Self {
            Self(ArrayString::<CACHE_STRING_SIZE>::from_iter(iter))
        }
    }

    impl Extend<char> for CacheString {
        fn extend<I: IntoIterator<Item = char>>(&mut self, iterable: I) {
            self.0.extend(iterable);
        }
    }

    impl<'a> Extend<&'a char> for CacheString {
        fn extend<I: IntoIterator<Item = &'a char>>(&mut self, iter: I) {
            self.0.extend(iter)
        }
    }

    impl<'a> Extend<&'a str> for CacheString {
        fn extend<I: IntoIterator<Item = &'a str>>(&mut self, iterable: I) {
            self.0.extend(iterable);
        }
    }

    impl Display for CacheString {
        #[inline]
        fn fmt(&self, f: &mut Formatter) -> fmt::Result {
            Display::fmt(&self.0, f)
        }
    }

    impl AsRef<str> for CacheString {
        #[inline]
        fn as_ref(&self) -> &str {
            self.0.as_ref()
        }
    }

    impl AsMut<str> for CacheString {
        #[inline]
        fn as_mut(&mut self) -> &mut str {
            self.0.as_mut()
        }
    }

    impl AsRef<[u8]> for CacheString {
        #[inline]
        fn as_ref(&self) -> &[u8] {
            self.0.as_ref()
        }
    }

    impl FromStr for CacheString {
        type Err = OutOfBounds;

        #[inline]
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(Self(ArrayString::try_from_str(s)?))
        }
    }

    impl PartialEq<str> for CacheString {
        #[inline]
        fn eq(&self, other: &str) -> bool {
            self.0.eq(other)
        }
    }

    impl PartialEq<&str> for CacheString {
        #[inline]
        fn eq(&self, other: &&str) -> bool {
            self.0.eq(other)
        }
    }

    impl Borrow<str> for CacheString {
        #[inline]
        fn borrow(&self) -> &str {
            self.0.borrow()
        }
    }
    impl BorrowMut<str> for CacheString {
        #[inline]
        fn borrow_mut(&mut self) -> &mut str {
            self.0.borrow_mut()
        }
    }

    impl<'a> Add<&'a str> for CacheString {
        type Output = Self;

        #[inline]
        fn add(self, other: &str) -> Self::Output {
            Self(self.0.add(other))
        }
    }

    impl Write for CacheString {
        #[inline]
        fn write_str(&mut self, slice: &str) -> fmt::Result {
            self.0.write_str(slice)
        }
    }

    impl From<ArrayString<CACHE_STRING_SIZE>> for CacheString {
        fn from(array: ArrayString<CACHE_STRING_SIZE>) -> Self {
            Self(array)
        }
    }

    impl From<&str> for CacheString {
        fn from(s: &str) -> Self {
            Self(ArrayString::<CACHE_STRING_SIZE>::from(s))
        }
    }

    impl IndexMut<RangeFrom<u8>> for CacheString {
        #[inline]
        fn index_mut(&mut self, index: RangeFrom<u8>) -> &mut str {
            self.0.index_mut(index)
        }
    }

    impl IndexMut<RangeTo<u8>> for CacheString {
        #[inline]
        fn index_mut(&mut self, index: RangeTo<u8>) -> &mut str {
            self.0.index_mut(index)
        }
    }

    impl IndexMut<RangeFull> for CacheString {
        #[inline]
        fn index_mut(&mut self, index: RangeFull) -> &mut str {
            self.0.index_mut(index)
        }
    }

    impl IndexMut<Range<u8>> for CacheString {
        #[inline]
        fn index_mut(&mut self, index: Range<u8>) -> &mut str {
            self.0.index_mut(index)
        }
    }

    impl IndexMut<RangeToInclusive<u8>> for CacheString {
        #[inline]
        fn index_mut(&mut self, index: RangeToInclusive<u8>) -> &mut str {
            self.0.index_mut(index)
        }
    }

    impl IndexMut<RangeInclusive<u8>> for CacheString {
        #[inline]
        fn index_mut(&mut self, index: RangeInclusive<u8>) -> &mut str {
            self.0.index_mut(index)
        }
    }

    impl Index<RangeFrom<u8>> for CacheString {
        type Output = str;

        #[inline]
        fn index(&self, index: RangeFrom<u8>) -> &Self::Output {
            self.0.index(index)
        }
    }

    impl Index<RangeTo<u8>> for CacheString {
        type Output = str;

        #[inline]
        fn index(&self, index: RangeTo<u8>) -> &Self::Output {
            self.0.index(index)
        }
    }

    impl Index<RangeFull> for CacheString {
        type Output = str;

        #[inline]
        fn index(&self, index: RangeFull) -> &Self::Output {
            self.0.index(index)
        }
    }

    impl Index<Range<u8>> for CacheString {
        type Output = str;

        #[inline]
        fn index(&self, index: Range<u8>) -> &Self::Output {
            self.0.index(index)
        }
    }

    impl Index<RangeToInclusive<u8>> for CacheString {
        type Output = str;

        #[inline]
        fn index(&self, index: RangeToInclusive<u8>) -> &Self::Output {
            self.0.index(index)
        }
    }

    impl Index<RangeInclusive<u8>> for CacheString {
        type Output = str;

        #[inline]
        fn index(&self, index: RangeInclusive<u8>) -> &Self::Output {
            self.0.index(index)
        }
    }
}
pub use cache_string::*;

#[cfg(test)]
mod tests {
    #[test]
    fn size_of_cache() {
        assert_eq!(core::mem::size_of::<super::CacheString>(), 64);
    }
}
