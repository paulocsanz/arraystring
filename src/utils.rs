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

/// Encodes `char` into `ArrayString` at specified position, heavily unsafe
///
/// We reimplement the `core` function to avoid panicking (UB instead, be careful)
///
/// Reimplemented from:
///
/// `https://github.com/rust-lang/rust/blob/7843e2792dce0f20d23b3c1cca51652013bef0ea/src/libcore/char/methods.rs#L447`
/// # Safety
///
/// - It's UB if index is outside of buffer's boundaries (buffer needs at most 4 bytes)
/// - It's UB if index is inside a character (like a index 3 for "a🤔")
#[inline]
pub(crate) unsafe fn encode_char_utf8_unchecked<S: ArrayString>(s: &mut S, ch: char, index: Size) {
    trace!("Encode char: {} to {}", ch, index);

    // UTF-8 ranges and tags for encoding characters
    const TAG_CONT: u8 = 0b1000_0000;
    const TAG_TWO_B: u8 = 0b1100_0000;
    const TAG_THREE_B: u8 = 0b1110_0000;
    const TAG_FOUR_B: u8 = 0b1111_0000;
    const MAX_ONE_B: u32 = 0x80;
    const MAX_TWO_B: u32 = 0x800;
    const MAX_THREE_B: u32 = 0x10000;

    debug_assert!(index + ch.len_utf8() as Size <= S::CAPACITY);
    debug_assert!(s.len() + ch.len_utf8() as Size <= S::CAPACITY);
    let (dst, code) = (s.buffer().get_unchecked_mut(index as usize..), ch as u32);

    if code < MAX_ONE_B {
        debug_assert!(dst.len() >= 1);
        *dst.get_unchecked_mut(0) = code as u8;
    } else if code < MAX_TWO_B {
        debug_assert!(dst.len() >= 2);
        *dst.get_unchecked_mut(0) = (code >> 6 & 0x1F) as u8 | TAG_TWO_B;
        *dst.get_unchecked_mut(1) = (code & 0x3F) as u8 | TAG_CONT;
    } else if code < MAX_THREE_B {
        debug_assert!(dst.len() >= 3);
        *dst.get_unchecked_mut(0) = (code >> 12 & 0x0F) as u8 | TAG_THREE_B;
        *dst.get_unchecked_mut(1) = (code >> 6 & 0x3F) as u8 | TAG_CONT;
        *dst.get_unchecked_mut(2) = (code & 0x3F) as u8 | TAG_CONT;
    } else {
        debug_assert!(dst.len() >= 4);
        *dst.get_unchecked_mut(0) = (code >> 18 & 0x07) as u8 | TAG_FOUR_B;
        *dst.get_unchecked_mut(1) = (code >> 12 & 0x3F) as u8 | TAG_CONT;
        *dst.get_unchecked_mut(2) = (code >> 6 & 0x3F) as u8 | TAG_CONT;
        *dst.get_unchecked_mut(3) = (code & 0x3F) as u8 | TAG_CONT;
    }
}

#[inline]
unsafe fn shift_unchecked(s: &mut [u8], from: usize, to: usize, len: usize) {
    debug_assert!(to + len <= s.len() && from + len <= s.len());
    debug!("Shift {}, {} to {}", &s[from..from + len], from, to);
    let (from, to) = (s.as_ptr().add(from), s.as_mut_ptr().add(to));
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
    shift_unchecked(s.buffer(), from, to, len as usize);
}

#[inline]
pub(crate) unsafe fn shift_left_unchecked<S: ArrayString>(s: &mut S, from: Size, to: Size) {
    debug_assert!(to <= from && from <= s.len());
    let (from, to, len) = (from as usize, to as usize, s.len() - to);
    shift_unchecked(s.buffer(), from, to, len as usize);
}

/// Returns error if size is outside of specified boundary
#[inline]
pub fn is_inside_boundary(size: Size, limit: Size) -> Result<(), OutOfBounds> {
    trace!("Out of bounds: ensures {} < {}", size, limit);
    Some(()).filter(|_| size <= limit).ok_or(OutOfBounds)
}

/// Returns error if index is not at a valid utf-8 char boundary
#[inline]
pub fn is_char_boundary<S: ArrayString>(s: &S, idx: Size) -> Result<(), Utf8> {
    trace!("Is char boundary: {} at {}", s.as_str(), idx);
    if s.as_str().is_char_boundary(idx as usize) {
        return Ok(());
    }
    Err(Utf8)
}

#[inline]
pub(crate) fn truncate_str(slice: &str, size: Size) -> &str {
    trace!("Truncate str: {} at {}", slice, size);
    if slice.is_char_boundary(size as usize) {
        unsafe { &slice.get_unchecked(..size as usize) }
    } else if (size as usize) < slice.len() {
        let mut index = size.saturating_sub(1) as usize;
        while !slice.is_char_boundary(index) {
            index = index.saturating_sub(1);
        }
        unsafe { &slice.get_unchecked(..index) }
    } else {
        slice
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "logs")]
    extern crate env_logger;

    use core::str::from_utf8;
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
    fn truncate() {
        assert_eq!(truncate_str("i", 10), "i");
        assert_eq!(truncate_str("iiiiii", 3), "iii");
        assert_eq!(truncate_str("🤔🤔🤔", 5), "🤔");
    }

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


    #[test]
    fn encode_char_utf8() {
        use array::Buffer;
        setup_logger();
        let mut string = CacheString::default();
        unsafe { encode_char_utf8_unchecked(&mut string, 'a', 0) };
        assert_eq!(from_utf8(unsafe { &string.buffer()[..1] }).unwrap(), "a");

        let mut string = CacheString::try_from_str("a").unwrap();
        unsafe { encode_char_utf8_unchecked(&mut string, '🤔', 1) };
        assert_eq!(from_utf8(unsafe { &string.buffer()[..5] }).unwrap(), "a🤔");
    }
}
