//! Stack based string with customized max-size
//!
//! A stack based strings with a maximum (customizable) size.
//!
//! **Never panics (all panic branches are impossible and therefore removed at compile time)**
//!
//! ## Why
//!
//! Data is generally bounded, you don't want a phone number with 30 characters, nor a username with 100. You probably don't even support it in your database.
//!
//! Why pay the cost of heap allocations of strings with unlimited capacity if you have limited boundaries?
//!
//! Array based strings always occupy the full space in memory, so they may use more size than dynamic strings.
//!
//! Array based strings are generally faster to create, clone and append than heap based strings (custom allocators and thread-locals may help with heap based ones).
//!
//! There are other stack based strings out there, they generally can grow (heap allocate), but the stack based size is defined by the library implementor, we go through a different route (fully stack based with customizable maximum size - per type)
//!
//! ArrayStrings types are created through a macro with customizable maximum size (implementing the appropriate traits)
//!
//! ```rust
//! // Always occupies 21 bytes of memory (in the stack)
//! //
//! // String's current (2018) implementation always uses 24 bytes + up to 20 bytes (actual username)
//! //   - Assuming 64 bit usize
//! //
//! // Remember that UTF-8 characters can use up to 4 bytes
//! # #[macro_use]
//! # extern crate arraystring;
//! # fn main() {
//! impl_string!(struct Username(20));
//! # }
//! ```
//!
//! ## Features
//!
//! **default:** `std`
//!
//! - `std` enabled by default, enables `std` compatibility (remove it to be `#[no_std]` compatible)
//! - `serde-traits` enables serde traits integration (`Serialize`/`Deserialize`)
//! - `diesel-traits` enables diesel traits integration (opperates like `String`)
//! - `logs` enables internal logging (you probably don't need it)
//!
//! ## Examples
//!
//! ```rust
//! #[macro_use]
//! extern crate arraystring;
//! use arraystring::{error::Error, prelude::*};
//!
//! impl_string!(pub struct Username(20));
//! impl_string!(pub struct Role(5));
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

pub mod array;
pub mod error;
pub mod utils;

/// Most used traits and data-strucutres
pub mod prelude {
    pub use crate::array::ArrayString;
    pub use crate::error::{OutOfBounds, Utf16, Utf8};
    #[doc(hidden)]
    pub use crate::utils::setup_logger;

    pub use crate::{SmallString, CacheString, MaxString, InlinableString};
}

//#[cfg(feature = "ffi")]
//pub mod ffi;

use crate::array::ArrayString;
use typenum::{U21, U63, U255, U127};

/// String with the same `mem::size_of` of `String`
pub type SmallString = ArrayString<U21>;
/// String that occupies 64 bytes in memory (full cache line)
pub type CacheString = ArrayString<U63>;
/// Biggest string that is inlined by the compiler (you should not depend on this, since the size can change, this is not stable)
#[doc(hidden)]
pub type InlinableString = ArrayString<U127>;
/// Maximum array string (255 bytes of string), bigger than that endsup slower
pub type MaxString = ArrayString<U255>;
