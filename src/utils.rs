//! Misc functions to improve readability

use crate::{arraystring::sealed::ValidCapacity, prelude::*};
#[cfg(feature = "logs")]
use log::{debug, trace};

pub(crate) trait IntoLossy<T>: Sized {
    fn into_lossy(self) -> T;
}

/// Returns error if size is outside of specified boundary
#[inline]
pub fn is_inside_boundary<S, L>(size: S, limit: L) -> Result<(), OutOfBounds>
where
    S: Into<usize>,
    L: Into<usize>,
{
    let (s, l) = (size.into(), limit.into());
    trace!("Out of bounds: ensures {} < {}", s, l);
    Some(()).filter(|_| s <= l).ok_or(OutOfBounds)
}

/// Returns error if index is not at a valid utf-8 char boundary
#[inline]
pub fn is_char_boundary<const N: usize>(
    s: &ArrayString<N>,
    idx: impl Into<usize>,
) -> Result<(), Utf8>
where
    ArrayString<N>: ValidCapacity,
{
    let idx = idx.into();
    trace!("Is char boundary: {} at {}", s.as_str(), idx);
    if s.as_str().is_char_boundary(idx) {
        return Ok(());
    }
    Err(Utf8)
}

#[inline]
pub unsafe fn is_char_boundary_at(arr: &[u8], index: usize) -> bool {
    if index == 0 {
        return true;
    }
    (*arr.get_unchecked(index) as i8) >= -0x40
}

/// Truncates string to specified size (ignoring last bytes if they form a partial `char`)
#[inline]
pub(crate) fn truncate_str(slice: &[u8], mut size: usize) -> &[u8] {
    trace!(
        "Truncate str: {} at {}",
        unsafe { std::str::from_utf8_unchecked(slice) },
        size
    );
    if size >= slice.len() {
        return slice;
    }
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

impl IntoLossy<u8> for u32 {
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
