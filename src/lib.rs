//! Fixed capacity stack based generic string
//!
//! Can't outgrow initial capacity (defined at compile time), always occupies [`capacity`] `+ 1` bytes of memory
//!
//! *Doesn't allocate memory on the heap and never panics in release (all panic branches are stripped at compile time - except `Index`/`IndexMut` traits, since they are supposed to)*
//!
//! ## Why
//!
//! Data is generally bounded, you don't want a phone number with 30 characters, nor a username with 100. You probably don't even support it in your database.
//!
//! Why pay the cost of heap allocations of strings with unlimited capacity if you have limited boundaries?
//!
//! Stack based strings are generally faster to create, clone and append to than heap based strings (custom allocators and thread-locals may help with heap based ones).
//!
//! But that becomes less true as you increase the array size, 255 bytes is the maximum we accept (bigger will just wrap) and it's probably already slower than heap based strings of that size (like in `std::string::String`)
//!
//! There are other stack based strings out there, they generally can have "unlimited" capacity (heap allocate), but the stack based size is defined by the library implementor, we go through a different route by implementing a string based in a generic array.
//!
//! Array based strings always occupies the full space in memory, so they may use more memory (in the stack) than dynamic strings.
//!
//! [`capacity`]: ./struct.ArrayString.html#method.capacity
//!
//! ## Features
//!
//! **default:** `std`, `impl-all`
//!
//! - `std` enabled by default, enables `std` compatibility - `impl Error` trait for errors (remove it to be `#[no_std]` compatible)
//! - `impl-all` enabled by default, automatically implements `ArrayString` for every single possible size (1 to 255), it make make compile times longer, so you may disable it and manually call `impl_generic_array!` for the sizes wanted
//! - `serde-traits` enables serde traits integration (`Serialize`/`Deserialize`)
//!
//!     Opperates like `String`, but truncates it if it's bigger than capacity
//!
//!  - `diesel-traits` enables diesel traits integration (`Insertable`/`Queryable`)
//!
//!      Opperates like `String`, but truncates it if it's bigger than capacity
//!
//! - `logs` enables internal logging
//!
//!     You will probably only need this if you are debugging this library
//!
//! ## Examples
//!
//! ```rust
//! use arraystring::{Error, prelude::*};
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
//!  ## Comparisons
//!
//! *These benchmarks ran while I streamed video and used my computer (with* **non-disclosed specs**) *as usual, so don't take the actual times too seriously, just focus on the comparison*
//!
//! ```my_custom_benchmark
//! small-string  (23 bytes)      clone                  4.837 ns
//! small-string  (23 bytes)      try_from_str          14.777 ns
//! small-string  (23 bytes)      from_str_truncate     11.360 ns
//! small-string  (23 bytes)      from_str_unchecked    11.291 ns
//! small-string  (23 bytes)      try_push_str           1.162 ns
//! small-string  (23 bytes)      push_str               3.490 ns
//! small-string  (23 bytes)      push_str_unchecked     1.098 ns
//! -------------------------------------------------------------
//! cache-string  (63 bytes)      clone                 10.170 ns
//! cache-string  (63 bytes)      try_from_str          25.579 ns
//! cache-string  (63 bytes)      from_str_truncate     16.977 ns
//! cache-string  (63 bytes)      from_str_unchecked    17.201 ns
//! cache-string  (63 bytes)      try_push_str           1.160 ns
//! cache-string  (63 bytes)      push_str               3.486 ns
//! cache-string  (63 bytes)      push_str_unchecked     1.115 ns
//! -------------------------------------------------------------
//! max-string   (255 bytes)      clone                147.410 ns
//! max-string   (255 bytes)      try_from_str         157.340 ns
//! max-string   (255 bytes)      from_str_truncate    158.000 ns
//! max-string   (255 bytes)      from_str_unchecked   158.420 ns
//! max-string   (255 bytes)      try_push_str           1.167 ns
//! max-string   (255 bytes)      push_str               4.337 ns
//! max-string   (255 bytes)      push_str_unchecked     1.103 ns
//! -------------------------------------------------------------
//! string (19 bytes)             clone                 33.295 ns
//! string (19 bytes)             from                  32.512 ns
//! string (19 bytes)             push str              28.128 ns
//! -------------------------------------------------------------
//! arrayvec string (23 bytes)    clone                  7.725 ns
//! arrayvec string (23 bytes)    from                  14.794 ns
//! arrayvec string (23 bytes)    push str               1.363 ns
//! -------------------------------------------------------------
//! inlinable-string (30 bytes)   clone                 16.751 ns
//! inlinable-string (30 bytes)   from_str              29.310 ns
//! inlinable-string (30 bytes)   push_str               2.865 ns
//! -------------------------------------------------------------
//! smallstring crate (20 bytes)  clone                 60.988 ns
//! smallstring crate (20 bytes)  from_str              50.233 ns
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

pub use typenum;

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
pub mod drain;
pub mod error;
mod implementations;
#[cfg(any(feature = "serde-traits", feature = "diesel-traits"))]
mod integration;
#[doc(hidden)]
pub mod utils;
pub mod ext;

/// Most used traits and data-strucutres
pub mod prelude {
    pub use crate::arraystring::ArrayString;
    pub use crate::ext::ArrayStringExt;
    pub use crate::ext::ArrayStringBase;
    pub use crate::drain::Drain;
    pub use crate::error::{OutOfBounds, Utf16, Utf8};
    pub use crate::{CacheString, MaxString, SmallString};
}

pub use crate::arraystring::ArrayString;
pub use crate::error::Error;

/// String with the same `mem::size_of` of a `String`
///
/// 24 bytes in 64 bits architecture
///
/// 12 bytes in 32 bits architecture (or others)
#[cfg(target_pointer_width = "64")]
#[cfg_attr(docs_rs_workaround, doc(cfg(feature = "impl-all")))]
#[cfg(feature = "impl-all")]
pub type SmallString = ArrayString<23>;

/// String with the same `mem::size_of` of a `String`
///
/// 24 bytes in 64 bits architecture
///
/// 12 bytes in 32 bits architecture (or others)
#[cfg(not(target_pointer_width = "64"))]
#[cfg_attr(docs_rs_workaround, doc(cfg(feature = "impl-all")))]
#[cfg(feature = "impl-all")]
pub type SmallString = ArrayString<11>;

/// Biggest array based string (255 bytes of string)
#[cfg_attr(docs_rs_workaround, doc(cfg(feature = "impl-all")))]
#[cfg(feature = "impl-all")]
pub type MaxString = ArrayString<255>;

#[cfg_attr(docs_rs_workaround, doc(cfg(feature = "impl-all")))]
#[cfg(feature = "impl-all")]
mod cache_string {
    use crate::prelude::*;
    use core::fmt::{self, Debug, Display, Formatter, Write};
    use core::{borrow::Borrow, borrow::BorrowMut, ops::*};
    use core::{cmp::Ordering, hash::Hash, hash::Hasher, str::FromStr};
    #[cfg(feature = "diesel-traits")]
    use diesel::{AsExpression, FromSqlRow};
    use crate::Error;

    const CACHE_STRING_SIZE: usize = 63;
    /// Newtype string that occupies 64 bytes in memory and is 64 bytes aligned (full cache line)
    ///
    /// 63 bytes of string
    #[repr(align(64))]
    #[derive(Copy, Clone, Default)]
    #[cfg_attr(feature = "diesel-traits", derive(AsExpression, FromSqlRow))]
    #[cfg_attr(feature = "diesel-traits", diesel(sql_type = diesel::sql_types::Text))]
    pub struct CacheString(pub ArrayString<CACHE_STRING_SIZE>);

    impl CacheString {
        /// Creates new empty `CacheString`.
        ///
        /// ```rust
        /// # use arraystring::prelude::*;
        /// # let _ = env_logger::try_init();
        /// let string = CacheString::new();
        /// assert!(string.is_empty());
        /// ```
        #[inline]
        pub fn new() -> Self {
            Self::default()
        }

        /// Returns maximum string capacity, defined at compile time, it will never change
        ///
        /// Should always return 63 bytes
        ///
        /// ```rust
        /// # use arraystring::prelude::*;
        /// # let _ = env_logger::try_init();
        /// assert_eq!(CacheString::capacity(), 63);
        /// ```
        #[inline]
        pub const fn capacity() -> u8 {
            ArrayString::<CACHE_STRING_SIZE>::capacity()
        }


        /// Creates a draining iterator that removes the specified range in the `CacheString` and yields the removed chars.
        ///
        /// Note: The element range is removed even if the iterator is not consumed until the end.
        ///
        /// ```rust
        /// # use arraystring::{error::Error, prelude::*};
        /// # fn main() -> Result<(), Error> {
        /// # let _ = env_logger::try_init();
        /// let mut s = CacheString::try_from_str("ABCD🤔")?;
        /// assert_eq!(s.drain(..3)?.collect::<Vec<_>>(), vec!['A', 'B', 'C']);
        /// assert_eq!(s.as_str(), "D🤔");
        ///
        /// assert_eq!(s.drain(3..), Err(Error::Utf8));
        /// assert_eq!(s.drain(10..), Err(Error::OutOfBounds));
        /// # Ok(())
        /// # }
        /// ```
        #[inline]
        pub fn drain<R>(&mut self, range: R) -> Result<Drain<CACHE_STRING_SIZE>, Error>
            where
                R: RangeBounds<u8>,
        {
            self.0.drain(range)
        }
    }

    impl ArrayStringBase for CacheString {

        #[inline]
        fn len(&self) -> u8 {
            self.size
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
            Ok(CacheString(ArrayString::try_from_str(s)?))
        }
    }

    impl<'a, 'b> PartialEq<str> for CacheString {
        #[inline]
        fn eq(&self, other: &str) -> bool {
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
            CacheString(self.0.add(other))
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
            CacheString(array)
        }
    }
}
pub use cache_string::*;

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::size_of;

    #[test]
    #[cfg(feature = "impl-all")]
    fn size_of_cache() {
        assert_eq!(size_of::<CacheString>(), 64);
    }
}
