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
//! impl_string(struct Username(20));
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
//! - `nightly` enables benchmarks (we will move to criterion eventually)
//!
//! ## Examples
//!
//! ```rust
//! extern crate arraystring;
//! #[macro_use]
//! use arraystring::{Error, prelude::*};
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
//!         username: Username::from_str("user")?,
//!         role: Role::from_str("admin")?
//!     };
//!     println!("{:?}", user);
//! }
//! ```
//!
//! ## Licenses
//!
//! MIT and Apache-2.0

#![doc(html_root_url = "https://docs.rs/arraystring/0.1.0/arraystring")]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "nightly", feature(test))]
#![deny(
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

#[cfg(feature = "logs")]
#[macro_use]
extern crate log;

#[cfg(feature = "diesel-traits")]
#[doc(hidden)]
pub extern crate diesel;

#[cfg(feature = "serde-traits")]
#[doc(hidden)]
pub extern crate serde;

//#[cfg(feature = "nightly")]
//pub mod ffi;

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
#[doc(hidden)]
#[macro_use]
pub extern crate std as core;

#[cfg(feature = "nightly")]
extern crate test;
#[cfg(feature = "nightly")]
pub use test::black_box;

#[macro_use]
mod macros;
pub mod error;
pub mod array;
mod utils;

/// Most used traits and data-strucutres
pub mod prelude {
    pub use error::{FromUtf16Error, FromUtf8Error, OutOfBoundsError};
    pub use array::ArrayString;
    pub use {CacheString, MaxString, Size, SmallString};
}

/// [`ArrayString`]'s buffer index
///
/// [`ArrayString`]: ./array/trait.ArrayString.html
pub type Size = u8;

pub use array::ArrayString;
impl_string!(pub struct SmallString(23));
impl_string!(pub struct CacheString(63));
impl_string!(pub struct MaxString(255));

#[cfg(test)]
mod tests {
    #[cfg(feature = "nightly")]
    use super::prelude::*;
    #[cfg(feature = "nightly")]
    use std;
    #[cfg(feature = "nightly")]
    extern crate test;
    #[cfg(feature = "nightly")]
    use self::std::sync::Arc;
    #[cfg(feature = "nightly")]
    use self::test::{black_box, Bencher};

    #[cfg(feature = "nightly")]
    static DATA: &str = "ajio";

    #[bench]
    #[cfg(feature = "nightly")]
    fn string_from_str(b: &mut Bencher) {
        b.iter(|| black_box(String::from(DATA)));
    }

    #[bench]
    #[cfg(feature = "nightly")]
    fn arrayfrom_str(b: &mut Bencher) {
        b.iter(|| unsafe { black_box(CacheString::from_str_unchecked(DATA)) });
    }

    #[bench]
    #[cfg(feature = "nightly")]
    fn string_clone(b: &mut Bencher) {
        let string = String::from(DATA);
        b.iter(|| black_box(string.clone()));
        //b.iter(|| black_box(clone_string(&string)));
    }

    #[bench]
    #[cfg(feature = "nightly")]
    fn arrayclone(b: &mut Bencher) {
        let string = unsafe { CacheString::from_str_unchecked(DATA) };
        b.iter(|| black_box(string.clone()));
        //b.iter(|| black_box(clone_limited(&string)));
    }

    #[bench]
    #[cfg(feature = "nightly")]
    fn arc_clone(b: &mut Bencher) {
        let string = Arc::new(String::from(DATA));
        b.iter(|| black_box(string.clone()));
        //b.iter(|| black_box(clone_limited(&string)));
    }
}
