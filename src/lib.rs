//! Generic-array based string
//!
//! Since rust doesn't have constant generics yet [`typenum`] is used to allow for generic arrays (through `generic-array` crate)
//!
//! Can't outgrow capacity (defined at compile time), always occupies [`capacity`] `+ 1` bytes of memory
//!
//! *Doesn't allocate memory on the heap*
//!
//! ## Why
//!
//! Data is generally bounded, you don't want a phone number with 30 characters, nor a username with 100. You probably don't even support it in your database.
//!
//! Why pay the cost of heap allocations of strings with unlimited capacity if you have limited boundaries?
//!
//! Array based strings always occupy the full space in memory, so they may use more memory (in the stack) than dynamic strings.
//!
//! Stack based strings are generally faster to create, clone and append to than heap based strings (custom allocators and thread-locals may help with heap based ones).
//!
//! But that becomes less true as you increase the array size, 255 bytes is the maximum we accept (bigger will just wrap) and it's probably already slower than heap based strings of that size (like in `std::string::String`)
//!
//! There are other stack based strings out there, they generally can have "unlimited" capacity (heap allocate), but the stack based size is defined by the library implementor, we go through a different route by implementing a string based in a generic array.
//!
//! [`typenum`]: ../typenum/index.html
//! [`capacity`]: ./struct.ArrayString.html#method.capacity
//!
//! ## Features
//!
//! **default:** `std`
//!
//! - `std` enabled by default, enables `std` compatibility (remove it to be `#[no_std]` compatible)
//! - `serde-traits` enables serde traits integration (`Serialize`/`Deserialize`)
//!
//!     Opperates like `String`, but truncates it if it's bigger than capacity
//!
//! - `diesel-traits` enables diesel traits integration (`Insertable`/`Queryable`)
//!
//!     Opperates like `String`, but truncates it if it's bigger than capacity
//!
//! - `logs` enables internal logging
//!
//!     You will probably only need this if you are debugging this library
//!
//! ## Examples
//!
//! ```rust
//! use arraystring::{Error, ArrayString, typenum::U5, typenum::U20};
//!
//! type Username = ArrayString<U20>;
//! type Role = ArrayString<U5>;
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
//! MIT and Apache-2.0

#![doc(html_root_url = "https://docs.rs/arraystring/0.1.0/arraystring")]
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
    const_err,
    dead_code,
    improper_ctypes,
    legacy_directory_ownership,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    plugin_as_library,
    private_in_public,
    safe_extern_statics,
    unconditional_recursion,
    unions_with_drop_fields,
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

#[cfg(feature = "std")]
extern crate std as core;

#[cfg(all(feature = "diesel-traits", test))]
#[macro_use]
extern crate diesel;

mod array;
pub mod drain;
pub mod error;
mod implementations;
#[cfg(any(feature = "serde-traits", feature = "diesel-traits"))]
mod integration;
#[doc(hidden)]
pub mod utils;

/// Most used traits and data-strucutres
pub mod prelude {
    pub use crate::array::ArrayString;
    pub use crate::drain::Drain;
    pub use crate::error::{OutOfBounds, Utf16, Utf8};
    #[doc(hidden)]
    pub use crate::utils::setup_logger;
    #[doc(hidden)]
    pub use crate::InlinableString;
    pub use crate::{CacheString, MaxString, SmallString};

    pub(crate) use generic_array::ArrayLength;
}

pub use crate::array::ArrayString;
pub use crate::error::Error;

use core::ops::Deref;
#[cfg(feature = "serde-traits")]
use serde::{Deserialize, Serialize};
use typenum::{U127, U21, U255, U63};

/// String with the same `mem::size_of` of a `String` in a 64 bits architecture
pub type SmallString = ArrayString<U21>;
/// Newtype string that occupies 64 bytes in memory and is 64 bytes aligned (full cache line)
#[repr(align(64))]
#[derive(Copy, Clone, Debug, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde-traits", derive(Deserialize, Serialize))]
pub struct CacheString(pub ArrayString<U63>);

impl Deref for CacheString {
    type Target = ArrayString<U63>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Biggest string that is inlined by the compiler (you should not depend on this, since the size can change, this is not stable)
#[doc(hidden)]
pub type InlinableString = ArrayString<U127>;
/// Biggest array based string (255 bytes of string)
pub type MaxString = ArrayString<U255>;
