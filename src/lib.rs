//! Stack based string with customized max-size

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
