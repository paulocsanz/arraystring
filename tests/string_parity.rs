use arraystring::prelude::*;
use std::panic::{catch_unwind, AssertUnwindSafe, RefUnwindSafe};
use std::{fmt::Debug, iter::FromIterator};

type TestString = ArrayString<255>;

fn unwind<R, F>(func: F) -> Result<R, ()>
where
    F: FnOnce() -> R,
{
    catch_unwind(AssertUnwindSafe(func)).map_err(|_| ())
}

static STRINGS: [&str; 8] = [
    "🤔🤔🤔🤔🤔🤔🤔",
    "ABCDEFGHIJKLMNOPQRSASHUDAHSDIUASH         ",
    "iejueueheuheuheu        0",
    "",
    "1",
    "ab",
    "   ",
    "        899saH(8hadhaiuhsidnkandu",
];

#[test]
fn try_from_str() {
    assert(String::from, TestString::try_from_str);
}

#[test]
fn from_str_truncate() {
    assert(String::from, TestString::from_str_truncate);
}

#[test]
fn try_from_chars() {
    assert(
        |s| String::from_iter(s.chars()),
        |s| TestString::try_from_chars(s.chars()),
    );
}

#[test]
fn from_chars() {
    assert(
        |s| String::from_iter(s.chars()),
        |s| TestString::from_chars_truncate(s.chars()),
    );
}

#[test]
fn try_from_iter() {
    assert(
        |s| String::from_iter(vec![s]),
        |s| TestString::try_from_iterator(vec![s]),
    );
}

#[test]
fn from_iter() {
    assert(
        |s| String::from_iter(vec![s]),
        |s| TestString::from_iterator_truncate(vec![s]),
    );
}

#[test]
fn try_from_utf16() {
    let utf16 = |s: &str| s.encode_utf16().collect::<Vec<_>>();
    assert(
        |s| String::from_utf16(&utf16(s)),
        |s| TestString::try_from_utf16(&utf16(s)),
    );
}

#[test]
fn from_utf16() {
    let utf16 = |s: &str| s.encode_utf16().collect::<Vec<_>>();
    assert(
        |s| String::from_utf16(&utf16(s)),
        |s| TestString::from_utf16_truncate(&utf16(s)),
    );
}

#[test]
fn from_utf16_lossy() {
    let utf16 = |s: &str| s.encode_utf16().collect::<Vec<_>>();
    assert(
        |s| String::from_utf16_lossy(&utf16(s)),
        |s| TestString::from_utf16_lossy_truncate(&utf16(s)),
    );
}

fn invalidate_utf16(buf: &mut [u16]) -> &mut [u16] {
    if buf.len() >= 7 {
        buf[0] = 0xD834;
        buf[1] = 0xDD1E;
        buf[2] = 0x006D;
        buf[3] = 0x0075;
        buf[4] = 0xD800;
        buf[5] = 0x0069;
        buf[6] = 0x0063;
    }
    buf
}

#[test]
fn try_from_utf16_invalid() {
    let utf16 = |s: &str| s.encode_utf16().collect::<Vec<_>>();
    assert(
        |s| String::from_utf16(invalidate_utf16(&mut utf16(s))),
        |s| TestString::try_from_utf16(invalidate_utf16(&mut utf16(s))),
    );
}

#[test]
fn from_utf16_invalid() {
    let utf16 = |s: &str| s.encode_utf16().collect::<Vec<_>>();
    assert(
        |s| String::from_utf16(invalidate_utf16(&mut utf16(s))),
        |s| TestString::from_utf16_truncate(invalidate_utf16(&mut utf16(s))),
    );
}

#[test]
fn from_utf16_lossy_invalid() {
    let utf16 = |s: &str| s.encode_utf16().collect::<Vec<_>>();
    assert(
        |s| String::from_utf16_lossy(invalidate_utf16(&mut utf16(s))),
        |s| TestString::from_utf16_lossy_truncate(invalidate_utf16(&mut utf16(s))),
    );
}

#[test]
fn try_push_str() {
    assert(
        |s| {
            let mut st = String::from(s);
            st.push_str(s);
            st
        },
        |s| {
            let mut ms = TestString::try_from_str(s).unwrap();
            ms.try_push_str(s).map(|()| ms)
        },
    );
}

#[test]
fn push_str() {
    assert(
        |s| {
            let mut st = String::from(s);
            st.push_str(s);
            st
        },
        |s| {
            let mut ms = TestString::try_from_str(s).unwrap();
            ms.push_str_truncate(s);
            ms
        },
    );
}

#[test]
fn add_str() {
    assert(
        |s| String::from(s) + s,
        |s| TestString::try_from_str(s).unwrap() + s,
    );
}

#[test]
fn push() {
    assert(
        |s| {
            let mut s = String::from(s);
            s.push('🤔');
            s
        },
        |s| {
            let mut ms = TestString::try_from_str(s).unwrap();
            ms.try_push('🤔').map(|()| ms)
        },
    );
}

#[test]
fn truncate() {
    assert(
        |s| {
            unwind(move || {
                let mut s = String::from(s);
                s.truncate(2);
                s
            })
        },
        |s| {
            let mut ms = TestString::try_from_str(s).unwrap();
            ms.truncate(2).map(|()| ms)
        },
    );
}

#[test]
fn pop() {
    assert(
        |s| {
            let mut s = String::from(s);
            let old = s.pop();
            (s, old)
        },
        |s| {
            let mut ms = TestString::try_from_str(s).unwrap();
            let old = ms.pop();
            (ms, old)
        },
    );
}

#[test]
fn trim() {
    assert(
        |s| String::from(s).trim().to_owned(),
        |s| {
            let mut ms = TestString::try_from_str(s).unwrap();
            ms.trim();
            ms
        },
    );
}

#[test]
fn remove() {
    assert(
        |s| {
            unwind(move || {
                let mut s = String::from(s);
                let removed = s.remove(2);
                (removed, s)
            })
        },
        |s| {
            let mut ms = TestString::try_from_str(s).unwrap();
            ms.remove(2).map(|r| (r, ms))
        },
    );
}

#[test]
fn retain() {
    assert(
        |s| {
            let mut s = String::from(s);
            s.retain(|c| c == 'a');
            s
        },
        |s| {
            let mut ms = TestString::try_from_str(s).unwrap();
            ms.retain(|c| c == 'a');
            ms
        },
    );
}

#[test]
fn try_insert() {
    assert(
        |s| {
            unwind(move || {
                let mut s = String::from(s);
                s.insert(2, 'a');
                s
            })
        },
        |s| {
            let mut ms = TestString::try_from_str(s).unwrap();
            ms.try_insert(2, 'a').map(|()| ms)
        },
    );
}

#[test]
fn try_insert_str() {
    assert(
        |s| {
            unwind(move || {
                let mut st = String::from(s);
                st.insert_str(2, s);
                st
            })
        },
        |s| {
            let mut ms = TestString::try_from_str(s).unwrap();
            ms.try_insert_str(2, s).map(|()| ms)
        },
    );
}

#[test]
fn insert_str() {
    assert(
        |s| {
            unwind(move || {
                let mut st = String::from(s);
                st.insert_str(2, s);
                (st, ())
            })
        },
        |s| {
            let mut ms = TestString::try_from_str(s).unwrap();
            let res = ms.insert_str_truncate(2, s);
            res.map(|()| (ms, ()))
        },
    );
}

#[test]
fn clear() {
    assert(
        |s| {
            let mut st = String::from(s);
            st.clear();
            st
        },
        |s| {
            let mut ms = TestString::try_from_str(s).unwrap();
            ms.clear();
            ms
        },
    );
}

#[test]
fn split_off() {
    assert(
        |s| {
            unwind(move || {
                let mut st = String::from(s);
                let split = st.split_off(2);
                (st, split)
            })
        },
        |s| {
            let mut ms = TestString::try_from_str(s).unwrap();
            ms.split_off(2).map(|s| (ms, s))
        },
    );
}

#[test]
fn drain() {
    assert(
        |s| {
            unwind(move || {
                let mut st = String::from(s);
                let drained: String = st.drain(..2).collect();
                (st, drained)
            })
        },
        |s| {
            let mut ms = TestString::try_from_str(s).unwrap();
            let drained = ms.drain(..2).map(|d| d.collect::<String>());
            drained.map(|d| (ms, d))
        },
    );
}

#[test]
fn replace_range() {
    assert(
        |s| {
            unwind(move || {
                let mut st = String::from(s);
                st.replace_range(..2, s);
                (st, ())
            })
        },
        |s| {
            let mut ms = TestString::try_from_str(s).unwrap();
            ms.replace_range(..2, s).map(|()| (ms, ()))
        },
    );
}

#[test]
fn len() {
    assert(
        |s| {
            let st = String::from(s);
            st.len().to_string()
        },
        |s| {
            let ms = TestString::try_from_str(s).unwrap();
            ms.len().to_string()
        },
    );
}

#[test]
fn is_empty() {
    assert(
        |s| {
            let st = String::from(s);
            st.is_empty().to_string()
        },
        |s| {
            let ms = TestString::try_from_str(s).unwrap();
            ms.is_empty().to_string()
        },
    );
}

#[test]
fn new() {
    assert_eq!(String::new().as_str(), TestString::new().as_str());
}

// Internal hackery to make the function `assert` possible

trait Normalize<EQ: PartialEq> {
    fn normalize(&self) -> EQ;
}

impl Normalize<Result<String, ()>> for () {
    fn normalize(&self) -> Result<String, ()> {
        Ok("".to_owned())
    }
}

impl Normalize<Result<String, ()>> for TestString {
    fn normalize(&self) -> Result<String, ()> {
        Ok(self.as_str().to_owned())
    }
}

impl Normalize<Result<String, ()>> for String {
    fn normalize(&self) -> Result<String, ()> {
        Ok(self.as_str().to_owned())
    }
}

impl<'a> Normalize<Result<String, ()>> for &'a str {
    fn normalize(&self) -> Result<String, ()> {
        Ok(self.to_string())
    }
}

impl Normalize<Result<String, ()>> for char {
    fn normalize(&self) -> Result<String, ()> {
        Ok(self.to_string())
    }
}

impl<N: Normalize<Result<String, ()>>> Normalize<Result<String, ()>> for Option<N> {
    fn normalize(&self) -> Result<String, ()> {
        self.as_ref().ok_or(()).and_then(|n| n.normalize())
    }
}

impl<E, K: PartialEq, N: Normalize<Result<K, ()>>> Normalize<Result<K, ()>> for Result<N, E> {
    fn normalize(&self) -> Result<K, ()> {
        self.as_ref().map_err(|_| ()).and_then(|n| n.normalize())
    }
}

impl<K: PartialEq, L: PartialEq, M: Normalize<K>, N: Normalize<L>> Normalize<Result<(K, L), ()>>
    for (M, N)
{
    fn normalize(&self) -> Result<(K, L), ()> {
        Ok((self.0.normalize(), self.1.normalize()))
    }
}

impl<J, K, L, M, N, O> Normalize<Result<(J, K, L), ()>> for (M, N, O)
where
    J: PartialEq,
    K: PartialEq,
    L: PartialEq,
    M: Normalize<J>,
    N: Normalize<K>,
    O: Normalize<L>,
{
    fn normalize(&self) -> Result<(J, K, L), ()> {
        Ok((self.0.normalize(), self.1.normalize(), self.2.normalize()))
    }
}

fn assert<Q, F, G, T, U>(f: F, g: G)
where
    Q: PartialEq + Debug,
    T: Normalize<Q>,
    U: Normalize<Q>,
    F: Fn(&'static str) -> T + RefUnwindSafe,
    G: Fn(&'static str) -> U + RefUnwindSafe,
{
    #[cfg(not(miri))]
    let _ = env_logger::try_init();
    for string in STRINGS.iter() {
        let f = f(string).normalize();
        let g = g(string).normalize();
        assert_eq!(f, g);
    }
}
