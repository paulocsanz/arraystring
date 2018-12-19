//! Misc functions to improve readability

use core::ptr::copy;
use prelude::*;

/// Setup `env_logger`
#[cfg(all(feature = "logs", feature = "std"))]
#[cfg_attr(
    docs_rs_workaround,
    doc(cfg(all(feature = "logs", feature = "std")))
)]
#[inline]
pub fn setup_logger() {
    use std::sync::Once;
    /// Ensures logger is initialized only once
    static INITIALIZE: Once = Once::new();
    INITIALIZE.call_once(::env_logger::init);
}

/// Mocks `setup_logger`
#[cfg(any(not(feature = "logs"), not(feature = "std")))]
#[inline]
#[doc(hidden)]
pub fn setup_logger() {}

/// Marks branch as impossible, UB if taken in prod, panics in debug
///
/// This function should never be used lightly, it will cause UB if used wrong
#[inline]
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

    debug_assert!(ch.len_utf8().saturating_add(index.into()) <= S::CAPACITY as usize);
    debug_assert!(ch.len_utf8().saturating_add(s.len().into()) <= S::CAPACITY as usize);
    let (dst, code) = (s.buffer().get_unchecked_mut(index.into()..), ch as u32);

    if code < MAX_ONE_B {
        debug_assert!(!dst.is_empty());
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

/// Copies part of slice to another part (`mem::copy`, basically `memmove`)
#[inline]
unsafe fn shift_unchecked(s: &mut [u8], from: usize, to: usize, len: usize) {
    debug_assert!(to.saturating_add(len) <= s.len() && from.saturating_add(len) <= s.len());
    debug!(
        "Shift {:?}, {} to {}",
        &s.get_unchecked(from..from.saturating_add(len)),
        from,
        to
    );
    let (f, t) = (s.as_ptr().add(from), s.as_mut_ptr().add(to));
    copy(f, t, len);
}

/// Shifts string right
/// # Safety
///
/// It's UB if `to + (s.len() - from)` is out of [`S::CAPACITY`]
///
/// [`S::CAPACITY`]: ../traits/trait.ArrayString.html#CAPACITY
#[inline]
pub(crate) unsafe fn shift_right_unchecked<S: ArrayString>(s: &mut S, from: Size, to: Size) {
    let (f, t, l) = (from as usize, to as usize, s.len().saturating_sub(from));
    debug_assert!(f <= t && t.saturating_add(l.into()) <= S::CAPACITY as usize);
    debug_assert!(s.as_str().is_char_boundary(f));
    shift_unchecked(s.buffer(), f, t, l.into());
}

/// Shifts string left
#[inline]
pub(crate) unsafe fn shift_left_unchecked<S: ArrayString>(s: &mut S, from: Size, to: Size) {
    debug_assert!(to <= from && from <= s.len());
    let (f, t, l) = (from as usize, to as usize, s.len().saturating_sub(to));
    debug_assert!(s.as_str().is_char_boundary(f));
    shift_unchecked(s.buffer(), f, t, l.into());
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
pub fn is_char_boundary<S: ArrayString>(s: &S, idx: Size) -> Result<(), Utf8> {
    trace!("Is char boundary: {} at {}", s.as_str(), idx);
    if s.as_str().is_char_boundary(idx.into()) {
        return Ok(());
    }
    Err(Utf8)
}

/// Truncates string to specified size (ignoring last bytes if they form a partial `char`)
#[inline]
pub(crate) fn truncate_str(slice: &str, size: Size) -> &str {
    trace!("Truncate str: {} at {}", slice, size);
    if slice.is_char_boundary(size.into()) {
        unsafe { slice.get_unchecked(..size.into()) }
    } else if (size as usize) < slice.len() {
        let mut index = size.saturating_sub(1) as usize;
        while !slice.is_char_boundary(index) {
            index = index.saturating_sub(1);
        }
        unsafe { slice.get_unchecked(..index) }
    } else {
        slice
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "logs")]
    extern crate env_logger;

    use super::*;
    use core::str::from_utf8;

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
        use traits::Buffer;
        setup_logger();
        let mut string = CacheString::default();
        unsafe { encode_char_utf8_unchecked(&mut string, 'a', 0) };
        assert_eq!(from_utf8(unsafe { &string.buffer()[..1] }).unwrap(), "a");

        let mut string = CacheString::try_from_str("a").unwrap();
        unsafe { encode_char_utf8_unchecked(&mut string, '🤔', 1) };
        assert_eq!(
            from_utf8(unsafe { &string.buffer()[..5] }).unwrap(),
            "a🤔"
        );
    }
}
