#![allow(missing_docs)]

pub use {ByteArray, Drain, FromUtf8Error, CacheString, OutOfBoundsError, Size, StringError};

/*
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        for _ in 0..1_000_000 {}
    }
}
*/

#[no_mangle]
pub unsafe extern "C" fn new() -> CacheString {
    CacheString::new()
}

#[no_mangle]
pub unsafe extern "C" fn from_str(s: &str) -> Result<CacheString, OutOfBoundsError> {
    CacheString::from_str(s)
}

#[no_mangle]
pub unsafe extern "C" fn from_str_truncate(s: &str) -> CacheString {
    CacheString::from_str_truncate(s)
}

#[no_mangle]
pub unsafe extern "C" fn from_str_unchecked(s: &str) -> CacheString {
    CacheString::from_str_unchecked(s)
}

#[no_mangle]
pub unsafe extern "C" fn from_iterator(v: Vec<&str>) -> Result<CacheString, OutOfBoundsError> {
    CacheString::from_iterator(v)
}

#[no_mangle]
pub unsafe extern "C" fn from_iterator_truncate(v: Vec<&str>) -> CacheString {
    CacheString::from_iterator_truncate(v)
}

#[no_mangle]
pub unsafe extern "C" fn from_iterator_unchecked(v: Vec<&str>) -> CacheString {
    CacheString::from_iterator_unchecked(v)
}

#[no_mangle]
pub unsafe extern "C" fn from_chars(v: Vec<char>) -> Result<CacheString, OutOfBoundsError> {
    CacheString::from_chars(v)
}

#[no_mangle]
pub unsafe extern "C" fn from_chars_truncate(v: Vec<char>) -> CacheString {
    CacheString::from_chars_truncate(v)
}

#[no_mangle]
pub unsafe extern "C" fn from_chars_unchecked(v: Vec<char>) -> CacheString {
    CacheString::from_chars_unchecked(v)
}

#[no_mangle]
pub unsafe extern "C" fn from_utf8(slice: &[u8]) -> Result<CacheString, StringError> {
    CacheString::from_utf8(slice)
}

#[no_mangle]
pub unsafe extern "C" fn from_utf16(slice: &[u16]) -> Result<CacheString, StringError> {
    CacheString::from_utf16(slice)
}

#[no_mangle]
pub unsafe extern "C" fn from_utf8_unchecked(slice: &[u8]) -> CacheString {
    CacheString::from_utf8_unchecked(slice)
}

#[no_mangle]
pub unsafe extern "C" fn into_bytes(s: CacheString) -> (ByteArray, Size) {
    s.into_bytes()
}

#[no_mangle]
pub unsafe extern "C" fn push_str(ls: &mut CacheString, s: &str) -> Result<(), OutOfBoundsError> {
    ls.push_str(s)
}

#[no_mangle]
pub unsafe extern "C" fn push_str_truncate(ls: &mut CacheString, s: &str) {
    ls.push_str_truncate(s)
}

#[no_mangle]
pub unsafe extern "C" fn push_str_unchecked(ls: &mut CacheString, s: &str) {
    ls.push_str_unchecked(s)
}

#[no_mangle]
pub unsafe extern "C" fn push(ls: &mut CacheString, c: char) -> Result<(), OutOfBoundsError> {
    ls.push(c)
}

#[no_mangle]
pub unsafe extern "C" fn push_unchecked(ls: &mut CacheString, c: char) {
    ls.push_unchecked(c)
}

#[no_mangle]
pub unsafe extern "C" fn truncate(ls: &mut CacheString, s: Size) -> Result<(), FromUtf8Error> {
    ls.truncate(s)
}

#[no_mangle]
pub unsafe extern "C" fn pop(ls: &mut CacheString) -> Option<char> {
    ls.pop()
}

#[no_mangle]
pub unsafe extern "C" fn remove(ls: &mut CacheString, idx: Size) -> Result<char, StringError> {
    ls.remove(idx)
}

#[no_mangle]
pub unsafe extern "C" fn retain(ls: &mut CacheString, i: Size) {
    ls.retain(|_| i < 30);
}

#[no_mangle]
pub unsafe extern "C" fn insert(
    ls: &mut CacheString,
    idx: Size,
    c: char,
) -> Result<(), StringError> {
    ls.insert(idx, c)
}

#[no_mangle]
pub unsafe extern "C" fn insert_unchecked(ls: &mut CacheString, idx: Size, c: char) {
    ls.insert_unchecked(idx, c)
}

#[no_mangle]
pub unsafe extern "C" fn insert_str(
    ls: &mut CacheString,
    idx: Size,
    s: &str,
) -> Result<(), StringError> {
    ls.insert_str(idx, s)
}

#[no_mangle]
pub unsafe extern "C" fn insert_str_truncate(
    ls: &mut CacheString,
    idx: Size,
    s: &str,
) -> Result<(), StringError> {
    ls.insert_str_truncate(idx, s)
}

#[no_mangle]
pub unsafe extern "C" fn insert_str_unchecked(ls: &mut CacheString, idx: Size, s: &str) {
    ls.insert_str_unchecked(idx, s)
}

#[no_mangle]
pub unsafe extern "C" fn split_off(
    ls: &mut CacheString,
    at: Size,
) -> Result<CacheString, StringError> {
    ls.split_off(at)
}

#[no_mangle]
pub unsafe extern "C" fn replace_range(
    ls: &mut CacheString,
    start: Size,
    end: Size,
    with: &str,
) -> Result<(), StringError> {
    ls.replace_range(start..end, with)
}

#[no_mangle]
pub unsafe extern "C" fn drain(
    ls: &mut CacheString,
    start: Size,
    end: Size,
) -> Result<Drain, StringError> {
    ls.drain(start..end)
}

#[no_mangle]
pub unsafe extern "C" fn add(ls: CacheString, s: &str) -> CacheString {
    ls + s
}
