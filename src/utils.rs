//! Misc functions to improve readability

use crate::{arraystring::sealed::ValidCapacity, prelude::*};
#[cfg(feature = "logs")]
use log::trace;
#[cfg(all(feature = "no-panic", not(debug_assertions)))]
use no_panic::no_panic;

pub(crate) trait IntoLossy<T>: Sized {
    fn into_lossy(self) -> T;
}

/// Returns error if size is outside of specified boundary
#[inline]
#[cfg_attr(all(feature = "no-panic", not(debug_assertions)), no_panic)]
pub(crate) fn is_inside_boundary(size: usize, limit: usize) -> Result<(), OutOfBounds> {
    trace!("Out of bounds: ensures {} <= {}", size, limit);
    (size <= limit).then_some(()).ok_or(OutOfBounds)
}

/// Returns error if index is not at a valid utf-8 char boundary
#[inline]
#[cfg_attr(all(feature = "no-panic", not(debug_assertions)), no_panic)]
pub(crate) fn is_char_boundary<const N: usize>(s: &ArrayString<N>, idx: usize) -> Result<(), Utf8>
where
    ArrayString<N>: ValidCapacity,
{
    trace!("Is char boundary: {} at {}", s.as_str(), idx);
    if s.as_str().is_char_boundary(idx) {
        return Ok(());
    }
    Err(Utf8)
}

/// Truncates string to specified size (ignoring last bytes if they form a partial `char`)
#[inline]
#[cfg_attr(all(feature = "no-panic", not(debug_assertions)), no_panic)]
pub(crate) fn truncate_str(slice: &str, mut size: usize) -> &str {
    trace!("Truncate str: {slice} at {size}");
    if size >= slice.len() {
        return slice;
    }
    // Safety: size will always be between 0 and capacity, so get_unchecked will never fail
    // We unroll the loop here as a utf-8 character can have at most 4 bytes, so decreasing 4
    // times ensures we will always find the char boundary. `str::is_char_boundary` returns true for 0 index
    unsafe {
        if slice.is_char_boundary(size) {
            return slice.get_unchecked(..size);
        }
        size -= 1;
        if slice.is_char_boundary(size) {
            return slice.get_unchecked(..size);
        }
        size -= 1;
        if slice.is_char_boundary(size) {
            return slice.get_unchecked(..size);
        }
        size -= 1;
        slice.get_unchecked(..size)
    }
}

impl IntoLossy<u8> for usize {
    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    #[cfg_attr(all(feature = "no-panic", not(debug_assertions)), no_panic)]
    fn into_lossy(self) -> u8 {
        self as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate() {
        assert_eq!(truncate_str("i", 10), "i");
        assert_eq!(truncate_str("iiiiii", 3), "iii");
        assert_eq!(truncate_str("🤔🤔🤔", 5), "🤔");
    }
}
