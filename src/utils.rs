//! Misc functions to improve readability

use core::ptr::copy;
use prelude::*;

/// Setup `env_logger`
#[cfg(all(feature = "logs", feature = "std"))]
pub fn setup_logger() {
    use std::sync::Once;
    static INITIALIZE: Once = Once::new();
    INITIALIZE.call_once(env_logger::init);
}

/// Mocks `setup_logger`
#[cfg(any(not(feature = "logs"), not(feature = "std")))]
#[doc(hidden)]
pub fn setup_logger() {}

/// Marks branch as impossible, UB if taken in prod, panics in debug
///
/// This function should never be used lightly, it will cause UB if used wrong
#[allow(unused_variables)]
pub(crate) unsafe fn never(s: &str) -> ! {
    #[cfg(debug_assertions)]
    panic!("{}", s);

    #[cfg(not(debug_assertions))]
    ::core::hint::unreachable_unchecked()
}

/// # Safety
///
/// It's UB if index is out of bounds (4 bytes needed at most)
#[inline]
pub(crate) unsafe fn encode_char_utf8_unchecked<S: ArrayString>(s: &mut S, ch: char, index: Size) {
    trace!("Encode char: {} to {}", ch, index);
    debug_assert!(index + ch.len_utf8() as Size <= S::CAPACITY);
    let buf = s.buffer().get_unchecked_mut(index as usize..);
    if buf.len() >= 4 {
        let _ = ch.encode_utf8(buf);
    } else {
        ::core::hint::unreachable_unchecked();
    }
}

#[inline]
pub unsafe fn shift_unchecked(s: &mut str, from: usize, to: usize, len: usize) {
    debug!("Shift {}, {} to {}", &s[from..from + len], from, to);
    let from = s.as_bytes().as_ptr().add(from);
    let to = s.as_bytes_mut().as_mut_ptr().add(to);
    copy(from, to, len);
}

/// # Safety
///
/// It's UB if `to + (s.len() - from)` is out of [`S::CAPACITY`]
///
/// [`S::CAPACITY`]: ../array/trait.ArrayString.html#CAPACITY
#[inline]
pub(crate) unsafe fn shift_right_unchecked<S: ArrayString>(s: &mut S, from: Size, to: Size) {
    let (from, to, len) = (from as usize, to as usize, s.len() - from);
    debug_assert!(from <= to && to + len as usize <= S::CAPACITY as usize);
    shift_unchecked(s.as_mut_str(), from, to, len as usize);
}

#[inline]
pub(crate) unsafe fn shift_left_unchecked<S: ArrayString>(s: &mut S, from: Size, to: Size) {
    debug_assert!(to <= from && from <= s.len());
    let (from, to, len) = (from as usize, to as usize, s.len() - to);
    shift_unchecked(s.as_mut_str(), from, to, len as usize);
}

#[inline]
pub(crate) fn is_inside_boundary(size: Size, limit: Size) -> Result<(), OutOfBounds> {
    trace!("Out of bounds: ensures {} < {}", size, limit);
    Some(()).filter(|_| size <= limit).ok_or(OutOfBounds)
}

#[inline]
pub(crate) fn is_char_boundary<S: ArrayString>(s: &S, idx: Size) -> Result<(), FromUtf8> {
    if s.as_str().is_char_boundary(idx as usize) {
        return Ok(());
    }
    Err(FromUtf8)
}

#[inline]
pub(crate) fn truncate_str(slice: &str, size: Size) -> &str {
    let mut ch = slice.chars();
    while ch.as_str().len() > size as usize {
        let _ = ch.next_back();
    }
    ch.as_str()
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "logs")]
    extern crate env_logger;

    use super::*;

    #[cfg(all(feature = "logs", feature = "std"))]
    fn setup_logger() {
        use std::sync::Once;
        static INITIALIZE: Once = Once::new();
        INITIALIZE.call_once(env_logger::init);
    }

    #[cfg(not(feature = "logs"))]
    fn setup_logger() {}

    #[test]
    fn shift_right() {
        setup_logger();
        let mut ls = CacheString::try_from_str("abcdefg").unwrap();
        unsafe { shift_right_unchecked(&mut ls, 0, 4) };
        ls.1 += 4;
        assert_eq!(ls.as_str(), "abcdabcdefg");
    }

    #[test]
    fn shift_left() {
        setup_logger();
        let mut ls = CacheString::try_from_str("abcdefg").unwrap();
        unsafe { shift_left_unchecked(&mut ls, 1, 0) };
        ls.1 -= 1;
        assert_eq!(ls.as_str(), "bcdefg");
    }

    #[test]
    fn shift_nop() {
        setup_logger();
        let mut ls = CacheString::try_from_str("abcdefg").unwrap();
        unsafe { shift_right_unchecked(&mut ls, 0, 0) };
        assert_eq!(ls.as_str(), "abcdefg");
        unsafe { shift_left_unchecked(&mut ls, 0, 0) };
        assert_eq!(ls.as_str(), "abcdefg");
    }
}
