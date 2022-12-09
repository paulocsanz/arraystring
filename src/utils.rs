//! Misc functions to improve readability

use crate::prelude::*;
use core::ptr::copy;
#[cfg(feature = "logs")]
use log::{debug, trace};

pub(crate) trait IntoLossy<T>: Sized {
    fn into_lossy(self) -> T;
}

/// Marks branch as impossible, UB if taken in prod, panics in debug
///
/// This function should never be used lightly, it will cause UB if used wrong
#[inline]
#[allow(unused_variables)]
pub(crate) unsafe fn never(s: &str) -> ! {
    #[cfg(debug_assertions)]
    panic!("{}", s);

    #[cfg(not(debug_assertions))]
    core::hint::unreachable_unchecked()
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
pub(crate) unsafe fn encode_char_utf8_unchecked<const N: usize>(
    s: &mut ArrayString<N>,
    ch: char,
    index: u8,
) -> usize {
    append_char_truncate(s.array.get_unchecked_mut(index as usize..), ch)
}

/// Encodes `char` into `&mut [u8]` and returns written length
#[inline]
pub(crate) fn append_char_truncate(
    dst: &mut [u8],
    code: char,
) -> usize {

    #[allow(clippy::missing_docs_in_private_items)]
    const TAG_CONT: u8 = 0b1000_0000;
    #[allow(clippy::missing_docs_in_private_items)]
    const TAG_TWO_B: u8 = 0b1100_0000;
    #[allow(clippy::missing_docs_in_private_items)]
    const TAG_THREE_B: u8 = 0b1110_0000;
    #[allow(clippy::missing_docs_in_private_items)]
    const TAG_FOUR_B: u8 = 0b1111_0000;

    let len = code.len_utf8();
    let code = code as u32;
    match (len, &mut dst[..]) {
        (1, [a, ..]) => {
            *a = code as u8;
        }
        (2, [a, b, ..]) => {
            *a = (code >> 6 & 0x1F) as u8 | TAG_TWO_B;
            *b = (code & 0x3F) as u8 | TAG_CONT;
        }
        (3, [a, b, c, ..]) => {
            *a = (code >> 12 & 0x0F) as u8 | TAG_THREE_B;
            *b = (code >> 6 & 0x3F) as u8 | TAG_CONT;
            *c = (code & 0x3F) as u8 | TAG_CONT;
        }
        (4, [a, b, c, d, ..]) => {
            *a = (code >> 18 & 0x07) as u8 | TAG_FOUR_B;
            *b = (code >> 12 & 0x3F) as u8 | TAG_CONT;
            *c = (code >> 6 & 0x3F) as u8 | TAG_CONT;
            *d = (code & 0x3F) as u8 | TAG_CONT;
        }
        _ => return 0,
    };
    len
}

/// Copies part of slice to another part (`mem::copy`, basically `memmove`)
#[inline]
unsafe fn shift_unchecked(s: &mut [u8], from: usize, to: usize, len: usize) {
    debug!(
        "Shift {:?} {}-{}",
        &s.get(from..).map(|s| s.get(..len)),
        from,
        to
    );
    debug_assert!(to.saturating_add(len) <= s.len() && from.saturating_add(len) <= s.len());
    let (f, t) = (s.as_ptr().add(from), s.as_mut_ptr().add(to));
    copy(f, t, len);
}

/// Shifts string right
///
/// # Safety
///
/// It's UB if `to + (s.len() - from)` is bigger than [`S::to_u8()`]
///
/// [`<S as Unsigned>::to_u8()`]: ../struct.ArrayString.html#CAPACITY
#[inline]
pub(crate) unsafe fn shift_right_unchecked<const N: usize, F, T>(s: &mut ArrayString<N>, from: F, to: T)
where
    F: Into<usize> + Copy,
    T: Into<usize> + Copy,
{
    let len = (s.len() as usize).saturating_sub(from.into());
    debug_assert!(from.into() <= to.into() && to.into().saturating_add(len) <= N);
    debug_assert!(s.as_str().is_char_boundary(from.into()));
    shift_unchecked(s.array.as_mut_slice(), from.into(), to.into(), len);
}

/// Shifts string left
#[inline]
pub(crate) unsafe fn shift_left_unchecked<const N: usize, F, T>(s: &mut ArrayString<N>, from: F, to: T)
where
    F: Into<usize> + Copy,
    T: Into<usize> + Copy,
{
    debug_assert!(to.into() <= from.into() && from.into() <= s.len().into());
    debug_assert!(s.as_str().is_char_boundary(from.into()));

    let len = (s.len() as usize).saturating_sub(to.into());
    shift_unchecked(s.array.as_mut_slice(), from.into(), to.into(), len);
}

/// Returns error if size is outside of specified boundary
#[inline]
pub fn is_inside_boundary<S, L>(size: S, limit: L) -> Result<(), OutOfBounds>
where
    S: Into<usize>,
    L: Into<usize>,
{
    let (s, l) = (size.into(), limit.into());
    (s <= l).then_some(()).ok_or(OutOfBounds)
}

/// Returns error if index is not at a valid utf-8 char boundary
#[inline]
pub fn is_char_boundary<const N: usize>(s: &ArrayString<N>, idx: u8) -> Result<(), Utf8> {
    trace!("Is char boundary: {} at {}", s.as_str(), idx);
    if s.as_str().is_char_boundary(idx.into()) {
        return Ok(());
    }
    Err(Utf8)
}

/// Truncates string to specified size (ignoring last bytes if they form a partial `char`)
#[inline]
pub(crate) fn truncate_str(slice: &str, size: u8) -> &str {
    trace!("Truncate str: {} at {}", slice, size);
    let mut size = size as usize;
    if slice.is_char_boundary(size) {
        return unsafe { slice.get_unchecked(..size) }
    } else if size >= slice.len() {
        return slice;
    }
    size -= 1;
    if slice.is_char_boundary(size) {
        return unsafe { slice.get_unchecked(..size) }
    }
    size -= 1;
    if slice.is_char_boundary(size) {
        return unsafe { slice.get_unchecked(..size) }
    }
    size -= 1;
    unsafe { slice.get_unchecked(..size) }
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
    use core::str::from_utf8;

    #[test]
    fn truncate() {
        assert_eq!(truncate_str("i", 10), "i");
        assert_eq!(truncate_str("iiiiii", 3), "iii");
        assert_eq!(truncate_str("🤔🤔🤔", 5), "🤔");
    }

    #[test]
    fn shift_right() {
        let _ = env_logger::try_init();
        let mut ls = SmallString::try_from_str("abcdefg").unwrap();
        unsafe { shift_right_unchecked(&mut ls, 0u8, 4u8) };
        ls.size += 4;
        assert_eq!(ls.as_str(), "abcdabcdefg");
    }

    #[test]
    fn shift_left() {
        let _ = env_logger::try_init();
        let mut ls = SmallString::try_from_str("abcdefg").unwrap();
        unsafe { shift_left_unchecked(&mut ls, 1u8, 0u8) };
        ls.size -= 1;
        assert_eq!(ls.as_str(), "bcdefg");
    }

    #[test]
    fn shift_nop() {
        let _ = env_logger::try_init();
        let mut ls = SmallString::try_from_str("abcdefg").unwrap();
        unsafe { shift_right_unchecked(&mut ls, 0u8, 0u8) };
        assert_eq!(ls.as_str(), "abcdefg");
        unsafe { shift_left_unchecked(&mut ls, 0u8, 0u8) };
        assert_eq!(ls.as_str(), "abcdefg");
    }

    #[test]
    fn encode_char_utf8() {
        let _ = env_logger::try_init();
        let mut string = SmallString::default();
        unsafe {
            let _ = encode_char_utf8_unchecked(&mut string, 'a', 0);
            assert_eq!(from_utf8(&string.array.as_mut_slice()[..1]).unwrap(), "a");
            let mut string = SmallString::try_from_str("a").unwrap();

            let _ = encode_char_utf8_unchecked(&mut string, '🤔', 1);
            assert_eq!(from_utf8(&string.array.as_mut_slice()[..5]).unwrap(), "a🤔");
        }
    }
}
