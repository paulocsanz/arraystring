//! Helps analyzing binary generated for each function

pub use error::Error;
pub use prelude::*;
pub use traits::Drain;

impl_string!(pub struct MaxString(255));

/// Alias to easily allow testing multiple sizes
type String = MaxString;

/// Creates new empty string
#[no_mangle]
pub unsafe extern "C" fn new() -> String {
    String::new()
}

/// Try to create string from str
#[no_mangle]
pub unsafe extern "C" fn try_from_str(s: &str) -> Result<String, OutOfBounds> {
    String::try_from_str(s)
}

/// Create string from str truncating it
#[no_mangle]
pub unsafe extern "C" fn from_str_truncate(s: &str) -> String {
    String::from_str_truncate(s)
}

/// Unsafelly create string from str (UB if bigger than capacity)
#[no_mangle]
pub unsafe extern "C" fn from_str_unchecked(s: &str) -> String {
    String::from_str_unchecked(s)
}

///
#[no_mangle]
pub unsafe extern "C" fn try_from_iterator(v: &[&str]) -> Result<String, OutOfBounds> {
    String::try_from_iterator(v)
}

///
#[no_mangle]
pub unsafe extern "C" fn from_iterator(v: &[&str]) -> String {
    String::from_iterator(v)
}

///
#[no_mangle]
pub unsafe extern "C" fn from_iterator_unchecked(v: &[&str]) -> String {
    String::from_iterator_unchecked(v)
}

///
#[no_mangle]
pub unsafe extern "C" fn try_from_chars(v: &str) -> Result<String, OutOfBounds> {
    String::try_from_chars(v.chars())
}

///
#[no_mangle]
pub unsafe extern "C" fn from_chars(v: &str) -> String {
    String::from_chars(v.chars())
}

///
#[no_mangle]
pub unsafe extern "C" fn from_chars_unchecked(v: &str) -> String {
    String::from_chars_unchecked(v.chars())
}

///
#[no_mangle]
pub unsafe extern "C" fn from_utf8(slice: &[u8]) -> Result<String, Error> {
    String::try_from_utf8(slice)
}

///
#[no_mangle]
pub unsafe extern "C" fn from_utf16(slice: &[u16]) -> Result<String, Error> {
    String::try_from_utf16(slice)
}

///
#[no_mangle]
pub unsafe extern "C" fn from_utf8_unchecked(slice: &[u8]) -> String {
    String::from_utf8_unchecked(slice)
}

///
#[no_mangle]
pub unsafe extern "C" fn try_push_str(ls: &mut String, s: &str) -> Result<(), OutOfBounds> {
    ls.try_push_str(s)
}

///
#[no_mangle]
pub unsafe extern "C" fn push_str(ls: &mut String, s: &str) {
    ls.push_str(s)
}

///
#[no_mangle]
pub unsafe extern "C" fn push_str_unchecked(ls: &mut String, s: &str) {
    ls.push_str_unchecked(s)
}

///
#[no_mangle]
pub unsafe extern "C" fn try_push(ls: &mut String, c: char) -> Result<(), OutOfBounds> {
    ls.try_push(c)
}

///
#[no_mangle]
pub unsafe extern "C" fn push_unchecked(ls: &mut String, c: char) {
    ls.push_unchecked(c)
}

///
#[no_mangle]
pub unsafe extern "C" fn truncate(ls: &mut String, s: u8) -> Result<(), Utf8> {
    ls.truncate(s)
}

///
#[no_mangle]
pub unsafe extern "C" fn pop(ls: &mut String) -> Option<char> {
    ls.pop()
}

///
#[no_mangle]
pub unsafe extern "C" fn remove(ls: &mut String, idx: u8) -> Result<char, Error> {
    ls.remove(idx)
}

///
#[no_mangle]
pub unsafe extern "C" fn retain(ls: &mut String) {
    ls.retain(|c| c != 'a');
}

///
#[no_mangle]
pub unsafe extern "C" fn try_insert(ls: &mut String, idx: u8, c: char) -> Result<(), Error> {
    ls.try_insert(idx, c)
}

///
#[no_mangle]
pub unsafe extern "C" fn insert_unchecked(ls: &mut String, idx: u8, c: char) {
    ls.insert_unchecked(idx, c)
}

///
#[no_mangle]
pub unsafe extern "C" fn try_insert_str(ls: &mut String, idx: u8, s: &str) -> Result<(), Error> {
    ls.try_insert_str(idx, s)
}

///
#[no_mangle]
pub unsafe extern "C" fn insert_str(ls: &mut String, idx: u8, s: &str) -> Result<(), Error> {
    ls.insert_str(idx, s)
}

///
#[no_mangle]
pub unsafe extern "C" fn insert_str_unchecked(ls: &mut String, idx: u8, s: &str) {
    ls.insert_str_unchecked(idx, s)
}

///
#[no_mangle]
pub unsafe extern "C" fn split_off(ls: &mut String, at: u8) -> Result<String, Error> {
    ls.split_off(at)
}

///
#[no_mangle]
pub unsafe extern "C" fn replace_range(
    ls: &mut String,
    start: u8,
    end: u8,
    with: &str,
) -> Result<(), Error> {
    ls.replace_range(start..end, with)
}

///
#[no_mangle]
pub unsafe extern "C" fn drain(
    ls: &mut String,
    start: u8,
    end: u8,
) -> Result<Drain<String>, Error> {
    ls.drain(start..end)
}

///
#[no_mangle]
pub unsafe extern "C" fn add(ls: String, s: &str) -> String {
    ls + s
}
