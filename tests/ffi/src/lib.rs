use arraystring::{Error, prelude::*, typenum::U63};

pub type Str = CacheString;
pub type Len = U63;

#[no_mangle]
pub unsafe extern "C" fn new() -> Str {
    Str::new()
}

#[no_mangle]
pub unsafe extern "C" fn try_from_str(s: &str) -> Result<Str, OutOfBounds> {
    Str::try_from_str(s)
}

#[no_mangle]
pub unsafe extern "C" fn from_str_truncate(s: &str) -> Str {
    Str::from_str_truncate(s)
}

#[no_mangle]
pub unsafe extern "C" fn from_str_unchecked(s: &str) -> Str {
    Str::from_str_unchecked(s)
}

#[no_mangle]
pub unsafe extern "C" fn try_from_iterator(s: &[&str]) -> Result<Str, OutOfBounds> {
    Str::try_from_iterator(s)
}

#[no_mangle]
pub unsafe extern "C" fn from_iterator(s: &[&str]) -> Str {
    Str::from_iterator(s)
}

#[no_mangle]
pub unsafe extern "C" fn from_iterator_unchecked(s: &[&str]) -> Str {
    Str::from_iterator_unchecked(s)
}

#[no_mangle]
pub unsafe extern "C" fn try_from_chars(s: &[char]) -> Result<Str, OutOfBounds> {
    Str::try_from_chars(s.iter().map(|c| *c))
}

#[no_mangle]
pub unsafe extern "C" fn from_chars(s: &[char]) -> Str {
    Str::from_chars(s.iter().map(|c| *c))
}

#[no_mangle]
pub unsafe extern "C" fn from_chars_unchecked(s: &[char]) -> Str {
    Str::from_chars_unchecked(s.iter().map(|c| *c))
}

#[no_mangle]
pub unsafe extern "C" fn try_from_utf8(s: &[u8]) -> Result<Str, Error> {
    Str::try_from_utf8(s)
}

#[no_mangle]
pub unsafe extern "C" fn from_utf8(s: &[u8]) -> Result<Str, Utf8> {
    Str::from_utf8(s)
}

#[no_mangle]
pub unsafe extern "C" fn try_from_utf16(s: &[u16]) -> Result<Str, Error> {
    Str::try_from_utf16(s)
}

#[no_mangle]
pub unsafe extern "C" fn from_utf16(s: &[u16]) -> Result<Str, Utf16> {
    Str::from_utf16(s)
}

#[no_mangle]
pub unsafe extern "C" fn from_utf16_lossy(s: &[u16]) -> Str {
    Str::from_utf16_lossy(s)
}

#[no_mangle]
pub unsafe extern "C" fn as_str(s: &Str) -> &str {
    s.as_str()
}

#[no_mangle]
pub unsafe extern "C" fn as_mut_str(s: &mut Str) -> &mut str {
    s.as_mut()
}

#[no_mangle]
pub unsafe extern "C" fn as_bytes(s: &Str) -> &[u8] {
    s.as_bytes()
}

#[no_mangle]
pub unsafe extern "C" fn as_mut_bytes(s: &mut Str) -> &mut [u8] {
    s.as_mut_bytes()
}

#[no_mangle]
pub unsafe extern "C" fn capacity() -> u8 {
    Str::capacity()
}

#[no_mangle]
pub unsafe extern "C" fn try_push_str(st: &mut Str, s: &str) -> Result<(), OutOfBounds> {
    st.try_push_str(s)
}

#[no_mangle]
pub unsafe extern "C" fn push_str(st: &mut Str, s: &str) {
    st.push_str(s);
}

#[no_mangle]
pub unsafe extern "C" fn push_str_unchecked(st: &mut Str, s: &str) {
    st.push_str_unchecked(s)
}

#[no_mangle]
pub unsafe extern "C" fn try_push(st: &mut Str, s: char) -> Result<(), OutOfBounds> {
    st.try_push(s)
}

#[no_mangle]
pub unsafe extern "C" fn push_unchecked(st: &mut Str, s: char) {
    st.push_unchecked(s)
}

#[no_mangle]
pub unsafe extern "C" fn truncate(st: &mut Str, s: u8) -> Result<(), Utf8> {
    st.truncate(s)
}

#[no_mangle]
pub unsafe extern "C" fn pop(st: &mut Str) -> Option<char> {
    st.pop()
}

#[no_mangle]
pub unsafe extern "C" fn trim(st: &mut Str) {
    st.trim()
}

#[no_mangle]
pub unsafe extern "C" fn remove(st: &mut Str, s: u8) -> Result<char, Error> {
    st.remove(s)
}

#[no_mangle]
pub unsafe extern "C" fn retain(st: &mut Str, f: &mut FnMut(char) -> bool) {
    st.retain(|el| f(el))
}

#[no_mangle]
pub unsafe extern "C" fn try_insert(st: &mut Str, idx: u8, ch: char) -> Result<(), Error> {
    st.try_insert(idx, ch)
}

#[no_mangle]
pub unsafe extern "C" fn try_insert_str(st: &mut Str, idx: u8, ch: &str) -> Result<(), Error> {
    st.try_insert_str(idx, ch)
}

#[no_mangle]
pub unsafe extern "C" fn insert_str(st: &mut Str, idx: u8, ch: &str) -> Result<(), Error> {
    st.insert_str(idx, ch)
}

#[no_mangle]
pub unsafe extern "C" fn insert_str_unchecked(st: &mut Str, idx: u8, ch: &str) {
    st.insert_str_unchecked(idx, ch)
}

#[no_mangle]
pub unsafe extern "C" fn len(st: &Str) -> u8 {
    st.len()
}

#[no_mangle]
pub unsafe extern "C" fn is_empty(st: &Str) -> bool {
    st.is_empty()
}

#[no_mangle]
pub unsafe extern "C" fn split_off(st: &mut Str, at: u8) -> Result<Str, Error> {
    st.split_off(at)
}

#[no_mangle]
pub unsafe extern "C" fn clear(st: &mut Str) {
    st.clear()
}

#[no_mangle]
pub unsafe extern "C" fn drain(st: &mut Str, start: u8, end: u8) -> Result<Drain<Len>, Error> {
    st.drain(start..end)
}

#[no_mangle]
pub unsafe extern "C" fn replace_range(st: &mut Str, start: u8, end: u8, with: &str) -> Result<(), Error> {
    st.replace_range(start..end, with)
}
