//! Misc functions to improve readability

use crate::{arraystring::sealed::ValidCapacity, prelude::*};
#[cfg(feature = "logs")]
use log::trace;
#[cfg(not(debug_assertions))]
use no_panic::no_panic;

pub(crate) trait IntoLossy<T>: Sized {
    fn into_lossy(self) -> T;
}

/// Returns error if size is outside of specified boundary
#[cfg_attr(not(debug_assertions), no_panic)]
#[inline]
pub(crate) fn is_inside_boundary(size: usize, limit: usize) -> Result<(), OutOfBounds> {
    trace!("Out of bounds: ensures {} <= {}", size, limit);
    (size <= limit).then_some(()).ok_or(OutOfBounds)
}

/// Returns error if index is not at a valid utf-8 char boundary
#[inline]
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

#[inline]
pub(crate) unsafe fn is_char_boundary_at(arr: &[u8], index: usize) -> bool {
    if index == 0 {
        return true;
    }
    (*arr.get_unchecked(index) as i8) >= -0x40
}

/// Truncates string to specified size (ignoring last bytes if they form a partial `char`)
#[inline]
#[cfg_attr(not(debug_assertions), no_panic)]
pub(crate) fn truncate_str(slice: &[u8], mut size: usize) -> &[u8] {
    trace!("Truncate str: {:?} at {size}", core::str::from_utf8(slice),);
    if size >= slice.len() {
        return slice;
    }
    // Safety: size will always be between 0 and capacity, so get_unchecked will never fail
    // We unroll the loop here as a utf-8 character can have at most 4 bytes, so decreasing 4
    // times ensures we will find the char boundary. `is_char_boundary_at` returns true for 0 index
    unsafe {
        if is_char_boundary_at(slice, size) {
            return slice.get_unchecked(..size);
        }
        size -= 1;
        if is_char_boundary_at(slice, size) {
            return slice.get_unchecked(..size);
        }
        size -= 1;
        if is_char_boundary_at(slice, size) {
            return slice.get_unchecked(..size);
        }
        size -= 1;
        slice.get_unchecked(..size)
    }
}

impl IntoLossy<u8> for usize {
    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    fn into_lossy(self) -> u8 {
        self as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate() {
        assert_eq!(truncate_str(b"i", 10), b"i");
        assert_eq!(truncate_str(b"iiiiii", 3), b"iii");
        assert_eq!(truncate_str("🤔🤔🤔".as_bytes(), 5), "🤔".as_bytes());
    }
}
