//! Misc functions to improve readability

use core::ptr::copy;
use prelude::*;

/// Marks branch as impossible, UB if taken in prod, panics in debug
///
/// This function should never be used lightly, it will cause UB if used wrong
pub(crate) unsafe fn never(s: &str) -> ! {
    #[cfg(debug_assertions)]
    panic!("{}", s);

    #[cfg(not(debug_assertions))]
    ::std::hint::unreachable_unchecked()
}

/// Uninitialized safe version of `https://github.com/rust-lang/rust/blob/master/src/libcore/char/methods.rs#L447`
///
/// # Safety
///
/// It's UB if index is out of bounds
#[inline]
pub(crate) unsafe fn encode_char_utf8_unchecked<S: ArrayString>(
    s: &mut S,
    ch: char,
    index: Size,
) {
    trace!("Encode char: {} to {}", ch, index);
    debug_assert!(index + ch.len_utf8() as Size <= S::SIZE);
    let buf = s.buffer().get_unchecked_mut(index as usize..);
    if buf.len() >= ch.len_utf8() {
        let _ = ch.encode_utf8(buf);
    } else {
        never("Buffer size is too small");
    }
}

#[inline]
pub(crate) unsafe fn shift_right_unchecked<S: ArrayString>(s: &mut S, from: Size, to: Size) {
    debug_assert!(from <= to);
    debug_assert!(to <= S::SIZE);
    let (from, to, len) = (from as usize, to as usize, (s.len() - from) as usize);
    debug!(
        "Shift right unchecked {:?} (from: {}) to {}",
        s.as_str().get_unchecked(from..from + len),
        from,
        to
    );
    copy(
        s.as_bytes().as_ptr().add(from),
        s.as_bytes_mut().as_mut_ptr().add(to),
        len,
    );
}

#[inline]
pub(crate) unsafe fn shift_left_unchecked<S: ArrayString>(s: &mut S, from: Size, to: Size) {
    debug_assert!(to <= from && from <= s.len());
    let len = (s.len() - to) as usize;
    debug!(
        "Shift left unchecked {:?} from: {} to {}",
        &s.as_str()[from as usize..from as usize + len],
        from,
        to
    );
    let from = s.as_bytes().as_ptr().add(from as usize);
    let to = s.as_bytes_mut().as_mut_ptr().add(to as usize);
    copy(from, to, len);
}

#[inline]
pub(crate) fn out_of_bounds(size: Size, limit: Size) -> Result<(), OutOfBoundsError> {
    trace!("Out of bounds: ensures {} < {}", size, limit);
    Some(()).filter(|_| size <= limit).ok_or(OutOfBoundsError)
}

#[inline]
pub(crate) fn is_char_boundary<S: ArrayString>(s: &S, idx: Size) -> Result<(), FromUtf8Error> {
    Some(())
        .filter(|_| s.as_str().is_char_boundary(idx as usize))
        .ok_or(FromUtf8Error)
}

#[inline]
pub(crate) fn truncate_str(slice: &str, size: Size) -> &str {
    let mut ch = slice.chars();
    while ch.as_str().len() > size as usize {
        let _ = ch.next_back();
    }
    ch.as_str()
}

/// Creates new [`ArrayString`] from string slice if length is lower or equal to [`SIZE`], otherwise returns an error.
///
/// [`ArrayString`]: ../array/trait.ArrayString.html
/// [`SIZE`]: ../array/trait.ArrayString.html#SIZE
/// ```rust
/// # use arraystring::{error::Error, prelude::*, utils::from_str};
/// # fn main() -> Result<(), Error> {
/// let string: LimitedString = from_str("My String")?;
/// assert_eq!(string.as_str(), "My String");
///
/// assert_eq!(from_str::<LimitedString, _>("")?.as_str(), "");
///
/// let out_of_bounds = "0".repeat(LimitedString::SIZE as usize + 1);
/// assert!(from_str::<LimitedString, _>(out_of_bounds).is_err());
/// # Ok(())
/// # }
/// ```
pub fn from_str<H, S>(s: S) -> Result<H, OutOfBoundsError>
where
    H: ArrayString,
    S: AsRef<str>,
{
    trace!("FromStr: {:?}", s.as_ref());
    out_of_bounds(s.as_ref().len() as Size, H::SIZE)?;
    unsafe { Ok(H::from_str_unchecked(s.as_ref())) }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "logs")]
    extern crate env_logger;

    use super::*;
    use std::str::FromStr;
    use LimitedString;

    #[cfg(feature = "logs")]
    fn setup_logger() {
        use self::std::sync::Once;
        static INITIALIZE: Once = Once::new();
        INITIALIZE.call_once(env_logger::init);
    }

    #[cfg(not(feature = "logs"))]
    fn setup_logger() {}

    #[test]
    fn shift() {
        setup_logger();
        let mut ls = LimitedString::from_str("abcdefg").unwrap();
        unsafe { shift_right_unchecked(&mut ls, 0, 1) };
        ls.0[0] = 'a' as u8;
        ls.1 += 1;
        assert_eq!(ls.as_str(), "aabcdefg");

        let mut ls = LimitedString::from_str("abcdefg").unwrap();
        unsafe { shift_left_unchecked(&mut ls, 1, 0) };
        ls.1 -= 1;
        assert_eq!(ls.as_str(), "bcdefg");

        let mut ls = LimitedString::from_str("abcdefg").unwrap();
        unsafe { shift_right_unchecked(&mut ls, 0, 0) };
        assert_eq!(ls.as_str(), "abcdefg");
        unsafe { shift_left_unchecked(&mut ls, 0, 0) };
        assert_eq!(ls.as_str(), "abcdefg");
    }
}
