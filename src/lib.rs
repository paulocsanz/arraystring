//! Fixed capacity stack based generic string
//!
//! Can't outgrow initial capacity (defined at compile time), always occupies [`capacity`] `+ 1` bytes of memory
//!
//! *Maximum Capacity is 255*
//!
//! *Doesn't allocate memory on the heap and never panics in release (all panic branches are stripped at compile time - except for `Index`/`IndexMut` traits, since they are supposed to)*
//!
//! ## Why
//!
//! Data is generally bounded, you don't want a phone number with 30 characters, nor a username with 100. You probably don't even support it in your database.
//!
//! Why pay the cost of heap allocations of strings with unlimited capacity if you have limited boundaries?
//!
//! Stack based strings are generally faster to create, clone and append to than heap based strings (custom allocators and thread-locals may help with heap based ones).
//!
//! But that becomes less true as you increase the array size, 255 bytes is the maximum we accept - [`MaxString`] and it's probably already slower than heap based strings of that size (like in `std::string::String`)
//!
//! There are other stack based strings out there, they generally can have "unlimited" capacity (heap allocate), but the stack based size is defined by the library implementor, we go through a different route by implementing a string based in a generic array.
//!
//! Array based strings always occupies the full space in memory, so they may use more memory (although in the stack) than dynamic strings.
//!
//! [`capacity`]: ./struct.ArrayString.html#method.capacity
//! [`MaxString`]: ./type.MaxString.html
//!
//! ## Features
//!
//! **default:** `std`
//!
//! - `std` enabled by default, enables `std` compatibility, implementing std trait (disable it to be `#[no_std]` compatible)
//! - `serde-traits` enables serde traits integration (`Serialize`/`Deserialize`)
//!
//!     Opperates like `String`, but truncates it if it's bigger than capacity
//!
//! - `diesel-traits` enables diesel traits integration
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

/// All structs defined by this crate
pub mod prelude {
    pub use crate::arraystring::ArrayString;
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
pub type SmallString = ArrayString<23>;

/// String with the same `mem::size_of` of a `String`
///
/// 24 bytes in 64 bits architecture
///
/// 12 bytes in 32 bits architecture (or others)
#[cfg(not(target_pointer_width = "64"))]
pub type SmallString = ArrayString<11>;

/// Biggest array based string (255 bytes of string)
pub type MaxString = ArrayString<255>;

mod cache_string {
    use crate::{prelude::*, Error};
    use core::fmt::{self, Debug, Display, Formatter, Write};
    use core::{borrow::Borrow, borrow::BorrowMut, ops::*};
    use core::{cmp::Ordering, hash::Hash, hash::Hasher, str::FromStr};

    const CACHE_STRING_SIZE: usize = 63;
    /// Newtype string that occupies 64 bytes in memory and is 64 bytes aligned (full cache line)
    ///
    /// 63 bytes of string
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
        /// # let _ = env_logger::try_init();
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
        /// # let _ = env_logger::try_init();
        /// let string = CacheString::try_from_str("My String")?;
        /// assert_eq!(string.as_str(), "My String");
        ///
        /// assert_eq!(CacheString::try_from_str("")?.as_str(), "");
        ///
        /// let out_of_bounds = "0".repeat(CacheString::capacity() as usize + 1);
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
        /// # let _ = env_logger::try_init();
        /// let string = CacheString::from_str_truncate("My String");
        /// # assert_eq!(string.as_str(), "My String");
        /// println!("{}", string);
        ///
        /// let truncate = "0".repeat(CacheString::capacity() as usize + 1);
        /// let truncated = "0".repeat(CacheString::capacity().into());
        /// let string = CacheString::from_str_truncate(&truncate);
        /// assert_eq!(string.as_str(), truncated);
        /// ```
        #[inline]
        pub fn from_str_truncate(string: impl AsRef<str>) -> Self {
            Self(ArrayString::from_str_truncate(string))
        }

        /// Creates new `CacheString` from string slice assuming length is appropriate.
        ///
        /// # Safety
        ///
        /// It's UB if `string.len()` > [`capacity`].
        ///
        /// [`capacity`]: ./struct.CacheString.html#method.capacity
        ///
        /// ```rust
        /// # use arraystring::prelude::*;
        /// let filled = "0".repeat(CacheString::capacity().into());
        /// let string = unsafe {
        ///     CacheString::from_str_unchecked(&filled)
        /// };
        /// assert_eq!(string.as_str(), filled.as_str());
        ///
        /// // Undefined behavior, don't do it
        /// // let out_of_bounds = "0".repeat(CacheString::capacity().into() + 1);
        /// // let ub = unsafe { CacheString::from_str_unchecked(out_of_bounds) };
        /// ```
        #[inline]
        pub unsafe fn from_str_unchecked(string: impl AsRef<str>) -> Self {
            Self(ArrayString::from_str_unchecked(string))
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
        /// # let _ = env_logger::try_init();
        /// let string = CacheString::from_iterator_truncate(&["My String", " Other String"][..]);
        /// assert_eq!(string.as_str(), "My String Other String");
        ///
        /// let out_of_bounds = (0..400).map(|_| "000");
        /// let truncated = "0".repeat(CacheString::capacity().into());
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

        /// Creates new `CacheString` from string slice iterator assuming length is appropriate.
        ///
        /// # Safety
        ///
        /// It's UB if `iter.map(|c| c.len()).sum()` > [`capacity`].
        ///
        /// [`capacity`]: ./struct.CacheString.html#method.capacity
        ///
        /// ```rust
        /// # use arraystring::prelude::*;
        /// let string = unsafe {
        ///     CacheString::from_iterator_unchecked(&["My String", " My Other String"][..])
        /// };
        /// assert_eq!(string.as_str(), "My String My Other String");
        ///
        /// // Undefined behavior, don't do it
        /// // let out_of_bounds = (0..400).map(|_| "000");
        /// // let undefined_behavior = unsafe {
        /// //     CacheString::from_iterator_unchecked(out_of_bounds)
        /// // };
        /// ```
        #[inline]
        pub unsafe fn from_iterator_unchecked(
            iter: impl IntoIterator<Item = impl AsRef<str>>,
        ) -> Self {
            Self(ArrayString::from_iterator_unchecked(iter))
        }

        /// Creates new `CacheString` from char iterator if total length is lower or equal to [`capacity`], otherwise returns an error.
        ///
        /// [`capacity`]: ./struct.CacheString.html#method.capacity
        ///
        /// ```rust
        /// # use arraystring::{Error, prelude::*};
        /// # fn main() -> Result<(), Error> {
        /// # let _ = env_logger::try_init();
        /// let string = CacheString::try_from_chars("My String".chars())?;
        /// assert_eq!(string.as_str(), "My String");
        ///
        /// let out_of_bounds = "0".repeat(CacheString::capacity() as usize + 1);
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
        /// # let _ = env_logger::try_init();
        /// let string = CacheString::from_chars_truncate("My String".chars());
        /// assert_eq!(string.as_str(), "My String");
        ///
        /// let out_of_bounds = "0".repeat(CacheString::capacity() as usize + 1);
        /// let truncated = "0".repeat(CacheString::capacity().into());
        ///
        /// let truncate = CacheString::from_chars_truncate(out_of_bounds.chars());
        /// assert_eq!(truncate.as_str(), truncated.as_str());
        /// ```
        #[inline]
        pub fn from_chars_truncate(iter: impl IntoIterator<Item = char>) -> Self {
            Self(ArrayString::from_chars_truncate(iter))
        }

        /// Creates new `CacheString` from char iterator assuming length is appropriate.
        ///
        /// # Safety
        ///
        /// It's UB if `iter.map(|c| c.len_utf8()).sum()` > [`capacity`].
        ///
        /// [`capacity`]: ./struct.CacheString.html#method.capacity
        ///
        /// ```rust
        /// # use arraystring::prelude::*;
        /// let string = unsafe { CacheString::from_chars_unchecked("My String".chars()) };
        /// assert_eq!(string.as_str(), "My String");
        ///
        /// // Undefined behavior, don't do it
        /// // let out_of_bounds = "000".repeat(400);
        /// // let undefined_behavior = unsafe { CacheString::from_chars_unchecked(out_of_bounds.chars()) };
        /// ```
        #[inline]
        pub unsafe fn from_chars_unchecked(iter: impl IntoIterator<Item = char>) -> Self {
            Self(ArrayString::from_chars_unchecked(iter))
        }

        /// Creates new `CacheString` from byte slice, returning [`Utf8`] on invalid utf-8 data or [`OutOfBounds`] if bigger than [`capacity`]
        ///
        /// [`Utf8`]: ./error/enum.Error.html#variant.Utf8
        /// [`OutOfBounds`]: ./error/enum.Error.html#variant.OutOfBounds
        /// [`capacity`]: ./struct.CacheString.html#method.capacity
        ///
        /// ```rust
        /// # use arraystring::{Error, prelude::*};
        /// # fn main() -> Result<(), Error> {
        /// # let _ = env_logger::try_init();
        /// let string = CacheString::try_from_utf8("My String")?;
        /// assert_eq!(string.as_str(), "My String");
        ///
        /// let invalid_utf8 = [0, 159, 146, 150];
        /// assert_eq!(CacheString::try_from_utf8(invalid_utf8), Err(Error::Utf8));
        ///
        /// let out_of_bounds = "0000".repeat(400);
        /// assert_eq!(CacheString::try_from_utf8(out_of_bounds.as_bytes()), Err(Error::OutOfBounds));
        /// # Ok(())
        /// # }
        /// ```
        #[inline]
        pub fn try_from_utf8(slice: impl AsRef<[u8]>) -> Result<Self, Error> {
            Ok(Self(ArrayString::try_from_utf8(slice)?))
        }

        /// Creates new `CacheString` from byte slice, returning [`Utf8`] on invalid utf-8 data, truncating if bigger than [`capacity`].
        ///
        /// [`Utf8`]: ./error/struct.Utf8.html
        /// [`capacity`]: ./struct.CacheString.html#method.capacity
        ///
        /// ```rust
        /// # use arraystring::{Error, prelude::*};
        /// # fn main() -> Result<(), Error> {
        /// # let _ = env_logger::try_init();
        /// let string = CacheString::from_utf8_truncate("My String")?;
        /// assert_eq!(string.as_str(), "My String");
        ///
        /// let invalid_utf8 = [0, 159, 146, 150];
        /// assert_eq!(CacheString::from_utf8_truncate(invalid_utf8), Err(Utf8));
        ///
        /// let out_of_bounds = "0".repeat(300);
        /// assert_eq!(CacheString::from_utf8_truncate(out_of_bounds.as_bytes())?.as_str(),
        ///            "0".repeat(CacheString::capacity().into()).as_str());
        /// # Ok(())
        /// # }
        /// ```
        #[inline]
        pub fn from_utf8_truncate(slice: impl AsRef<[u8]>) -> Result<Self, Utf8> {
            Ok(Self(ArrayString::from_utf8_truncate(slice)?))
        }

        /// Creates new `CacheString` from byte slice assuming it's utf-8 and of a appropriate size.
        ///
        /// # Safety
        ///
        /// It's UB if `slice` is not a valid utf-8 string or `slice.len()` > [`capacity`].
        ///
        /// [`capacity`]: ./struct.CacheString.html#method.capacity
        ///
        /// ```rust
        /// # use arraystring::prelude::*;
        /// let string = unsafe { CacheString::from_utf8_unchecked("My String") };
        /// assert_eq!(string.as_str(), "My String");
        ///
        /// // Undefined behavior, don't do it
        /// // let out_of_bounds = "0".repeat(300);
        /// // let ub = unsafe { CacheString::from_utf8_unchecked(out_of_bounds)) };
        /// ```
        #[inline]
        pub unsafe fn from_utf8_unchecked(slice: impl AsRef<[u8]>) -> Self {
            Self(ArrayString::from_utf8_unchecked(slice))
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
        /// # let _ = env_logger::try_init();
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
        /// # let _ = env_logger::try_init();
        /// let music = [0xD834, 0xDD1E, 0x006d, 0x0075, 0x0073, 0x0069, 0x0063];
        /// let string = CacheString::from_utf16_truncate(music)?;
        /// assert_eq!(string.as_str(), "𝄞music");
        ///
        /// let invalid_utf16 = [0xD834, 0xDD1E, 0x006d, 0x0075, 0xD800, 0x0069, 0x0063];
        /// assert_eq!(CacheString::from_utf16_truncate(invalid_utf16), Err(Utf16));
        ///
        /// let out_of_bounds: Vec<u16> = (0..300).map(|_| 0).collect();
        /// assert_eq!(CacheString::from_utf16_truncate(out_of_bounds)?.as_str(),
        ///            "\0".repeat(CacheString::capacity().into()).as_str());
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
        /// # let _ = env_logger::try_init();
        /// let music = [0xD834, 0xDD1E, 0x006d, 0x0075, 0x0073, 0x0069, 0x0063];
        /// let string = CacheString::from_utf16_lossy_truncate(music);
        /// assert_eq!(string.as_str(), "𝄞music");
        ///
        /// let invalid_utf16 = [0xD834, 0xDD1E, 0x006d, 0x0075, 0xD800, 0x0069, 0x0063];
        /// assert_eq!(CacheString::from_utf16_lossy_truncate(invalid_utf16).as_str(), "𝄞mu\u{FFFD}ic");
        ///
        /// let out_of_bounds: Vec<u16> = (0..300).map(|_| 0).collect();
        /// assert_eq!(CacheString::from_utf16_lossy_truncate(&out_of_bounds).as_str(),
        ///            "\0".repeat(CacheString::capacity().into()).as_str());
        /// # Ok(())
        /// # }
        /// ```
        #[inline]
        pub fn from_utf16_lossy_truncate(slice: impl AsRef<[u16]>) -> Self {
            Self(ArrayString::from_utf16_lossy_truncate(slice))
        }

        /// Extracts a string slice containing the entire `CacheString`
        ///
        /// ```rust
        /// # use arraystring::{Error, prelude::*};
        /// # fn main() -> Result<(), Error> {
        /// # let _ = env_logger::try_init();
        /// let s = CacheString::try_from_str("My String")?;
        /// assert_eq!(s.as_str(), "My String");
        /// # Ok(())
        /// # }
        /// ```
        #[inline]
        pub fn as_str(&self) -> &str {
            self.0.as_str()
        }

        /// Extracts a mutable string slice containing the entire `CacheString`
        ///
        /// ```rust
        /// # use arraystring::{Error, prelude::*};
        /// # fn main() -> Result<(), Error> {
        /// # let _ = env_logger::try_init();
        /// let mut s = CacheString::try_from_str("My String")?;
        /// assert_eq!(s.as_mut_str(), "My String");
        /// # Ok(())
        /// # }
        /// ```
        #[inline]
        pub fn as_mut_str(&mut self) -> &mut str {
            self.0.as_mut_str()
        }

        /// Extracts a byte slice containing the entire `CacheString`
        ///
        /// ```rust
        /// # use arraystring::{Error, prelude::*};
        /// # fn main() -> Result<(), Error> {
        /// # let _ = env_logger::try_init();
        /// let s = CacheString::try_from_str("My String")?;
        /// assert_eq!(s.as_bytes(), "My String".as_bytes());
        /// # Ok(())
        /// # }
        /// ```
        #[inline]
        pub fn as_bytes(&self) -> &[u8] {
            self.0.as_bytes()
        }

        /// Extracts a mutable string slice containing the entire `CacheString`
        ///
        /// # Safety
        ///
        /// It's UB to store invalid UTF-8 data in the returned byte array
        ///
        /// ```rust
        /// # use arraystring::{Error, prelude::*};
        /// # fn main() -> Result<(), Error> {
        /// let mut s = CacheString::try_from_str("My String")?;
        /// assert_eq!(unsafe { s.as_mut_bytes() }, "My String".as_bytes());
        /// # Ok(())
        /// # }
        /// ```
        #[inline]
        pub unsafe fn as_mut_bytes(&mut self) -> &mut [u8] {
            self.0.as_mut_bytes()
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
            CACHE_STRING_SIZE as u8
        }

        /// Pushes string slice to the end of the `CacheString` if total size is lower or equal to [`capacity`], otherwise returns an error.
        ///
        /// [`capacity`]: ./struct.CacheString.html#method.capacity
        ///
        /// ```rust
        /// # use arraystring::{Error, prelude::*};
        /// # fn main() -> Result<(), Error> {
        /// # let _ = env_logger::try_init();
        /// let mut s = CacheString::try_from_str("My String")?;
        /// s.try_push_str(" My other String")?;
        /// assert_eq!(s.as_str(), "My String My other String");
        ///
        /// assert!(s.try_push_str("0".repeat(CacheString::capacity().into())).is_err());
        /// # Ok(())
        /// # }
        /// ```
        #[inline]
        pub fn try_push_str(&mut self, string: impl AsRef<str>) -> Result<(), OutOfBounds> {
            self.0.try_push_str(string)
        }

        /// Pushes string slice to the end of the `CacheString` truncating total size if bigger than [`capacity`].
        ///
        /// [`capacity`]: ./struct.CacheString.html#method.capacity
        ///
        /// ```rust
        /// # use arraystring::{Error, prelude::*};
        /// # fn main() -> Result<(), Error> {
        /// # let _ = env_logger::try_init();
        /// let mut s = CacheString::try_from_str("My String")?;
        /// s.push_str_truncate(" My other String");
        /// assert_eq!(s.as_str(), "My String My other String");
        ///
        /// let mut s = CacheString::default();
        /// s.push_str_truncate("0".repeat(CacheString::capacity() as usize + 1));
        /// assert_eq!(s.as_str(), "0".repeat(CacheString::capacity().into()).as_str());
        /// # Ok(())
        /// # }
        /// ```
        #[inline]
        pub fn push_str_truncate(&mut self, string: impl AsRef<str>) {
            self.0.push_str_truncate(string);
        }

        /// Pushes string slice to the end of the `CacheString` assuming total size is appropriate.
        ///
        /// # Safety
        ///
        /// It's UB if `self.len() + string.len()` > [`capacity`].
        ///
        /// [`capacity`]: ./struct.CacheString.html#method.capacity
        ///
        /// ```rust
        /// # use arraystring::{Error, prelude::*};
        /// # fn main() -> Result<(), Error> {
        /// let mut s = CacheString::try_from_str("My String")?;
        /// unsafe { s.push_str_unchecked(" My other String") };
        /// assert_eq!(s.as_str(), "My String My other String");
        ///
        /// // Undefined behavior, don't do it
        /// // let mut undefined_behavior = CacheString::default();
        /// // undefined_behavior.push_str_unchecked("0".repeat(CacheString::capacity().into() + 1));
        /// # Ok(())
        /// # }
        /// ```
        #[inline]
        pub unsafe fn push_str_unchecked(&mut self, string: impl AsRef<str>) {
            self.0.push_str_unchecked(string);
        }

        /// Inserts character to the end of the `CacheString` erroring if total size if bigger than [`capacity`].
        ///
        /// [`capacity`]: ./struct.CacheString.html#method.capacity
        ///
        /// ```rust
        /// # use arraystring::{Error, prelude::*};
        /// # fn main() -> Result<(), Error> {
        /// # let _ = env_logger::try_init();
        /// let mut s = CacheString::try_from_str("My String")?;
        /// s.try_push('!')?;
        /// assert_eq!(s.as_str(), "My String!");
        ///
        /// let mut s = CacheString::try_from_str(&"0".repeat(CacheString::capacity().into()))?;
        /// assert!(s.try_push('!').is_err());
        /// # Ok(())
        /// # }
        /// ```
        #[inline]
        pub fn try_push(&mut self, character: char) -> Result<(), OutOfBounds> {
            self.0.try_push(character)
        }

        /// Inserts character to the end of the `CacheString` assuming length is appropriate
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
        /// let mut s = CacheString::try_from_str("My String")?;
        /// unsafe { s.push_unchecked('!') };
        /// assert_eq!(s.as_str(), "My String!");
        ///
        /// // s = CacheString::try_from_str(&"0".repeat(CacheString::capacity().into()))?;
        /// // Undefined behavior, don't do it
        /// // s.push_unchecked('!');
        /// # Ok(())
        /// # }
        /// ```
        #[inline]
        pub unsafe fn push_unchecked(&mut self, ch: char) {
            self.0.push_unchecked(ch);
        }

        /// Truncates `CacheString` to specified size (if smaller than current size and a valid utf-8 char index).
        ///
        /// ```rust
        /// # use arraystring::{Error, prelude::*};
        /// # fn main() -> Result<(), Error> {
        /// # let _ = env_logger::try_init();
        /// let mut s = CacheString::try_from_str("My String")?;
        /// s.truncate(5)?;
        /// assert_eq!(s.as_str(), "My St");
        ///
        /// // Does nothing
        /// s.truncate(6)?;
        /// assert_eq!(s.as_str(), "My St");
        ///
        /// // Index is not at a valid char
        /// let mut s = CacheString::try_from_str("🤔")?;
        /// assert!(s.truncate(1).is_err());
        /// # Ok(())
        /// # }
        /// ```
        #[inline]
        pub fn truncate(&mut self, size: u8) -> Result<(), Utf8> {
            self.0.truncate(size)
        }

        /// Removes last character from `CacheString`, if any.
        ///
        /// ```rust
        /// # use arraystring::{Error, prelude::*};
        /// # fn main() -> Result<(), Error> {
        /// # let _ = env_logger::try_init();
        /// let mut s = CacheString::try_from_str("A🤔")?;
        /// assert_eq!(s.pop(), Some('🤔'));
        /// assert_eq!(s.pop(), Some('A'));
        /// assert_eq!(s.pop(), None);
        /// # Ok(())
        /// # }
        /// ```
        #[inline]
        pub fn pop(&mut self) -> Option<char> {
            self.0.pop()
        }

        /// Removes spaces from the beggining and end of the string
        ///
        /// ```rust
        /// # use arraystring::prelude::*;
        /// # fn main() -> Result<(), OutOfBounds> {
        /// # let _ = env_logger::try_init();
        /// let mut string = CacheString::try_from_str("   to be trimmed     ")?;
        /// string.trim();
        /// assert_eq!(string.as_str(), "to be trimmed");
        ///
        /// let mut string = CacheString::try_from_str("   🤔")?;
        /// string.trim();
        /// assert_eq!(string.as_str(), "🤔");
        /// # Ok(())
        /// # }
        /// ```
        #[inline]
        pub fn trim(&mut self) {
            self.0.trim()
        }

        /// Removes specified char from `CacheString`
        ///
        /// ```rust
        /// # use arraystring::{Error, prelude::*};
        /// # fn main() -> Result<(), Error> {
        /// # let _ = env_logger::try_init();
        /// let mut s = CacheString::try_from_str("ABCD🤔")?;
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
            self.0.remove(idx)
        }

        /// Retains only the characters specified by the predicate.
        ///
        /// ```rust
        /// # use arraystring::{Error, prelude::*};
        /// # fn main() -> Result<(), Error> {
        /// # let _ = env_logger::try_init();
        /// let mut s = CacheString::try_from_str("ABCD🤔")?;
        /// s.retain(|c| c != '🤔');
        /// assert_eq!(s.as_str(), "ABCD");
        /// # Ok(())
        /// # }
        /// ```
        #[inline]
        pub fn retain(&mut self, f: impl FnMut(char) -> bool) {
            self.0.retain(f)
        }

        /// Inserts character at specified index, returning error if total length is bigger than [`capacity`].
        ///
        /// Returns [`OutOfBounds`] if `idx` is out of bounds and [`Utf8`] if `idx` is not a char position
        ///
        /// [`capacity`]: ./struct.CacheString.html#method.capacity
        /// [`OutOfBounds`]: ./error/enum.Error.html#variant.OutOfBounds
        /// [`Utf8`]: ./error/enum.Error.html#variant.Utf8
        ///
        /// ```rust
        /// # use arraystring::{Error, prelude::*};
        /// # fn main() -> Result<(), Error> {
        /// # let _ = env_logger::try_init();
        /// let mut s = CacheString::try_from_str("ABCD🤔")?;
        /// s.try_insert(1, 'A')?;
        /// s.try_insert(2, 'B')?;
        /// assert_eq!(s.as_str(), "AABBCD🤔");
        /// assert_eq!(s.try_insert(20, 'C'), Err(Error::OutOfBounds));
        /// assert_eq!(s.try_insert(8, 'D'), Err(Error::Utf8));
        ///
        /// let mut s = CacheString::try_from_str(&"0".repeat(CacheString::capacity().into()))?;
        /// assert_eq!(s.try_insert(0, 'C'), Err(Error::OutOfBounds));
        /// # Ok(())
        /// # }
        /// ```
        #[inline]
        pub fn try_insert(&mut self, idx: u8, ch: char) -> Result<(), Error> {
            self.0.try_insert(idx, ch)
        }

        /// Inserts character at specified index assuming length is appropriate
        ///
        /// # Safety
        ///
        /// It's UB if `idx` does not lie on a utf-8 `char` boundary
        ///
        /// It's UB if `self.len() + character.len_utf8()` > [`capacity`]
        ///
        /// [`capacity`]: ./struct.CacheString.html#method.capacity
        ///
        /// ```rust
        /// # use arraystring::{Error, prelude::*};
        /// # fn main() -> Result<(), Error> {
        /// let mut s = CacheString::try_from_str("ABCD🤔")?;
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
            self.0.insert_unchecked(idx, ch)
        }

        /// Inserts string slice at specified index, returning error if total length is bigger than [`capacity`].
        ///
        /// Returns [`OutOfBounds`] if `idx` is out of bounds
        /// Returns [`Utf8`] if `idx` is not a char position
        ///
        /// [`capacity`]: ./struct.CacheString.html#method.capacity
        /// [`OutOfBounds`]: ./error/enum.Error.html#variant.OutOfBounds
        /// [`Utf8`]: ./error/enum.Error.html#variant.Utf8
        ///
        /// ```rust
        /// # use arraystring::{Error, prelude::*};
        /// # fn main() -> Result<(), Error> {
        /// # let _ = env_logger::try_init();
        /// let mut s = CacheString::try_from_str("ABCD🤔")?;
        /// s.try_insert_str(1, "AB")?;
        /// s.try_insert_str(1, "BC")?;
        /// assert_eq!(s.try_insert_str(1, "0".repeat(CacheString::capacity().into())),
        ///            Err(Error::OutOfBounds));
        /// assert_eq!(s.as_str(), "ABCABBCD🤔");
        /// assert_eq!(s.try_insert_str(20, "C"), Err(Error::OutOfBounds));
        /// assert_eq!(s.try_insert_str(10, "D"), Err(Error::Utf8));
        /// # Ok(())
        /// # }
        /// ```
        #[inline]
        pub fn try_insert_str(&mut self, idx: u8, s: impl AsRef<str>) -> Result<(), Error> {
            self.0.try_insert_str(idx, s)
        }

        /// Inserts string slice at specified index, truncating size if bigger than [`capacity`].
        ///
        /// Returns [`OutOfBounds`] if `idx` is out of bounds and [`Utf8`] if `idx` is not a char position
        ///
        /// [`capacity`]: ./struct.CacheString.html#method.capacity
        /// [`OutOfBounds`]: ./error/enum.Error.html#variant.OutOfBounds
        /// [`Utf8`]: ./error/enum.Error.html#variant.Utf8
        ///
        /// ```rust
        /// # use arraystring::{Error, prelude::*};
        /// # fn main() -> Result<(), Error> {
        /// # let _ = env_logger::try_init();
        /// let mut s = CacheString::try_from_str("ABCD🤔")?;
        /// s.insert_str_truncate(1, "AB")?;
        /// s.insert_str_truncate(1, "BC")?;
        /// assert_eq!(s.as_str(), "ABCABBCD🤔");
        ///
        /// assert_eq!(s.insert_str_truncate(20, "C"), Err(Error::OutOfBounds));
        /// assert_eq!(s.insert_str_truncate(10, "D"), Err(Error::Utf8));
        ///
        /// s.clear();
        /// s.insert_str_truncate(0, "0".repeat(CacheString::capacity() as usize + 10))?;
        /// assert_eq!(s.as_str(), "0".repeat(CacheString::capacity().into()).as_str());
        /// # Ok(())
        /// # }
        /// ```
        #[inline]
        pub fn insert_str_truncate(
            &mut self,
            idx: u8,
            string: impl AsRef<str>,
        ) -> Result<(), Error> {
            self.0.insert_str_truncate(idx, string)
        }

        /// Inserts string slice at specified index, assuming total length is appropriate.
        ///
        /// # Safety
        ///
        /// It's UB if `idx` does not lie on a utf-8 `char` boundary
        ///
        /// It's UB if `self.len() + string.len()` > [`capacity`]
        ///
        /// [`capacity`]: ./struct.CacheString.html#method.capacity
        ///
        /// ```rust
        /// # use arraystring::{Error, prelude::*};
        /// # fn main() -> Result<(), Error> {
        /// let mut s = CacheString::try_from_str("ABCD🤔")?;
        /// unsafe { s.insert_str_unchecked(1, "AB") };
        /// unsafe { s.insert_str_unchecked(1, "BC") };
        /// assert_eq!(s.as_str(), "ABCABBCD🤔");
        ///
        /// // Undefined behavior, don't do it
        /// // unsafe { s.insert_str_unchecked(20, "C") };
        /// // unsafe { s.insert_str_unchecked(10, "D") };
        /// // unsafe { s.insert_str_unchecked(1, "0".repeat(CacheString::capacity().into())) };
        /// # Ok(())
        /// # }
        /// ```
        #[inline]
        pub unsafe fn insert_str_unchecked(&mut self, idx: u8, string: impl AsRef<str>) {
            self.0.insert_str_unchecked(idx, string)
        }

        /// Returns `CacheString` length.
        ///
        /// ```rust
        /// # use arraystring::{Error, prelude::*};
        /// # fn main() -> Result<(), Error> {
        /// # let _ = env_logger::try_init();
        /// let mut s = CacheString::try_from_str("ABCD")?;
        /// assert_eq!(s.len(), 4);
        /// s.try_push('🤔')?;
        /// // Emojis use 4 bytes (this is the default rust behavior, length of u8)
        /// assert_eq!(s.len(), 8);
        /// # Ok(())
        /// # }
        /// ```
        #[inline]
        pub fn len(&self) -> u8 {
            self.0.len()
        }

        /// Checks if `CacheString` is empty.
        ///
        /// ```rust
        /// # use arraystring::{Error, prelude::*};
        /// # fn main() -> Result<(), Error> {
        /// # let _ = env_logger::try_init();
        /// let mut s = CacheString::try_from_str("ABCD")?;
        /// assert!(!s.is_empty());
        /// s.clear();
        /// assert!(s.is_empty());
        /// # Ok(())
        /// # }
        /// ```
        #[inline]
        pub fn is_empty(&self) -> bool {
            self.0.is_empty()
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
        /// # let _ = env_logger::try_init();
        /// let mut s = CacheString::try_from_str("AB🤔CD")?;
        /// assert_eq!(s.split_off(6)?.as_str(), "CD");
        /// assert_eq!(s.as_str(), "AB🤔");
        /// assert_eq!(s.split_off(20), Err(Error::OutOfBounds));
        /// assert_eq!(s.split_off(4), Err(Error::Utf8));
        /// # Ok(())
        /// # }
        /// ```
        #[inline]
        pub fn split_off(&mut self, at: u8) -> Result<Self, Error> {
            Ok(Self(self.0.split_off(at)?))
        }

        /// Empties `CacheString`
        ///
        /// ```rust
        /// # use arraystring::{Error, prelude::*};
        /// # fn main() -> Result<(), Error> {
        /// # let _ = env_logger::try_init();
        /// let mut s = CacheString::try_from_str("ABCD")?;
        /// assert!(!s.is_empty());
        /// s.clear();
        /// assert!(s.is_empty());
        /// # Ok(())
        /// # }
        /// ```
        #[inline]
        pub fn clear(&mut self) {
            self.0.clear()
        }

        /// Creates a draining iterator that removes the specified range in the `CacheString` and yields the removed chars.
        ///
        /// Note: The element range is removed even if the iterator is not consumed until the end.
        ///
        /// ```rust
        /// # use arraystring::{Error, prelude::*};
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
        pub fn drain(
            &mut self,
            range: impl RangeBounds<u8>,
        ) -> Result<Drain<CACHE_STRING_SIZE>, Error> {
            self.0.drain(range)
        }

        /// Removes the specified range of the `CacheString`, and replaces it with the given string. The given string doesn't need to have the same length as the range.
        ///
        /// ```rust
        /// # use arraystring::{Error, prelude::*};
        /// # fn main() -> Result<(), Error> {
        /// # let _ = env_logger::try_init();
        /// let mut s = CacheString::try_from_str("ABCD🤔")?;
        /// s.replace_range(2..4, "EFGHI")?;
        /// assert_eq!(s.as_str(), "ABEFGHI🤔");
        ///
        /// assert_eq!(s.replace_range(9.., "J"), Err(Error::Utf8));
        /// assert_eq!(s.replace_range(..90, "K"), Err(Error::OutOfBounds));
        /// assert_eq!(s.replace_range(0..1, "0".repeat(CacheString::capacity().into())),
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
            self.0.replace_range(r, with)
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
            Ok(Self(ArrayString::try_from_str(s)?))
        }
    }

    impl PartialEq<str> for CacheString {
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
}
pub use cache_string::*;

#[cfg(test)]
mod tests {
    #[test]
    fn size_of_cache() {
        assert_eq!(core::mem::size_of::<super::CacheString>(), 64);
    }
}
