extern crate limited_string;

#[cfg(feature = "logs")]
extern crate env_logger;

use limited_string::prelude::*;
use std::{fmt::Debug, iter::FromIterator, str::FromStr};

static STRINGS: [&'static str; 5] = [
    "ABCDEFGHIJKLMNOPQRS",
    "iejueueheuheuheu",
    "",
    "1",
    "899saH(8hadhaiuhsidnkandu",
];

#[cfg(feature = "logs")]
fn setup_logger() {
    use self::std::sync::Once;
    static INITIALIZE: Once = Once::new();
    INITIALIZE.call_once(env_logger::init);
}

#[cfg(not(feature = "logs"))]
fn setup_logger() {}

fn assert<F, G, T, U>(f: F, g: G)
where
    T: Debug + PartialEq + AsRef<str>,
    U: Debug + PartialEq + AsRef<str>,
    F: Fn(&str) -> T,
    G: Fn(&str) -> U,
{
    for string in STRINGS.into_iter() {
        assert_eq!(f(string).as_ref(), g(string).as_ref());
    }
}

#[test]
fn from_str() {
    assert(
        |s| String::from(s),
        |s| <LimitedString as FromStr>::from_str(s).unwrap(),
    );
}

#[test]
fn from_str_truncate() {
    assert(|s| String::from(s), |s| LimitedString::from_str_truncate(s));
}

#[test]
fn from_str_unchecked() {
    assert(
        |s| String::from(s),
        |s| unsafe { LimitedString::from_str_unchecked(s) },
    );
}

#[test]
fn from_chars() {
    assert(
        |s| String::from_iter(s.chars()),
        |s| LimitedString::from_chars(s.chars()).unwrap(),
    );
}

#[test]
fn from_chars_truncate() {
    assert(
        |s| String::from_iter(s.chars()),
        |s| LimitedString::from_chars_truncate(s.chars()),
    );
}

#[test]
fn from_chars_unchecked() {
    assert(
        |s| String::from_iter(s.chars()),
        |s| unsafe { LimitedString::from_chars_unchecked(s.chars()) },
    );
}

#[test]
fn from_iter() {
    assert(
        |s| String::from_iter(vec![s]),
        |s| LimitedString::from_iterator(vec![s]).unwrap(),
    );
}

#[test]
fn from_iter_truncate() {
    assert(
        |s| String::from_iter(vec![s]),
        |s| LimitedString::from_iterator_truncate(vec![s]),
    );
}

#[test]
fn from_iter_unchecked() {
    assert(
        |s| String::from_iter(vec![s]),
        |s| unsafe { LimitedString::from_iterator_unchecked(vec![s]) },
    );
}

#[test]
fn from_utf8() {
    assert(
        |s| String::from_utf8(s.as_bytes().to_vec()).unwrap(),
        |s| LimitedString::from_utf8(s.as_bytes()).unwrap(),
    );
}

#[test]
fn from_utf16() {
    let utf16 = |s: &str| s.encode_utf16().collect::<Vec<_>>();
    assert(
        |s| String::from_utf16(&utf16(s)).unwrap(),
        |s| LimitedString::from_utf16(&utf16(s)).unwrap(),
    );
}

#[test]
fn from_utf8_unchecked() {
    unsafe {
        assert(
            |s| String::from_utf8_unchecked(s.as_bytes().to_vec()),
            |s| LimitedString::from_utf8_unchecked(s.as_bytes()),
        );
    }
}

#[test]
fn string_parity() {
    setup_logger();
    assert_eq!(String::new().as_str(), LimitedString::new().as_str());
    for string in STRINGS.into_iter() {
        let string = *string;
        let mut s = String::from(string);
        let mut ls: LimitedString = FromStr::from_str(string).unwrap();

        assert_eq!(s.len(), ls.len() as usize);
        assert_eq!(s.is_empty(), ls.is_empty());

        let mut s = String::from(string);
        assert_eq!(s.as_mut_str(), ls.as_str_mut());
        assert_eq!(s.as_bytes(), ls.as_bytes());

        s.push_str(string);
        ls.push_str(string).unwrap();
        assert_eq!(s.as_str(), ls.as_str());

        let mut ls: LimitedString = FromStr::from_str(string).unwrap();
        ls.push_str_truncate(string);
        assert_eq!(s.as_str(), ls.as_str());

        let ls: LimitedString = FromStr::from_str(string).unwrap();
        assert_eq!(s.as_str(), (ls + string).as_str());

        let mut ls: LimitedString = FromStr::from_str(string).unwrap();
        unsafe { ls.push_str_unchecked(string) };
        assert_eq!(s.as_str(), ls.as_str());

        let mut s = String::from(string);
        s.push('🤔');

        let mut ls: LimitedString = FromStr::from_str(string).unwrap();
        ls.push('🤔').unwrap();
        assert_eq!(s.as_str(), ls.as_str());

        let mut ls: LimitedString = FromStr::from_str(string).unwrap();
        unsafe { ls.push_unchecked('🤔') };
        assert_eq!(s.as_str(), ls.as_str());

        let mut s = String::from(string);
        s.truncate(10);

        let mut ls: LimitedString = FromStr::from_str(string).unwrap();
        ls.truncate(10).unwrap();
        assert_eq!(s.as_str(), ls.as_str());

        assert_eq!(s.pop(), ls.pop());

        let mut s = String::from(string);
        if s.len() > 2 {
            let mut ls: LimitedString = FromStr::from_str(string).unwrap();
            assert_eq!(s.remove(2), ls.remove(2).unwrap());
        }

        let mut s = String::from(string);
        let mut ls: LimitedString = FromStr::from_str(string).unwrap();
        s.retain(|c| c == 'a');
        ls.retain(|c| c == 'a');
        assert_eq!(s.as_str(), ls.as_str());

        let mut s = String::from(string);
        s.insert(0, 'a');

        let mut ls: LimitedString = FromStr::from_str(string).unwrap();
        ls.insert(0, 'a').unwrap();
        assert_eq!(s.as_str(), ls.as_str());

        let mut ls: LimitedString = FromStr::from_str(string).unwrap();
        unsafe { ls.insert_unchecked(0, 'a') };
        assert_eq!(s.as_str(), ls.as_str());

        let mut s = String::from(string);
        s.insert_str(0, "eifha");

        let mut ls: LimitedString = FromStr::from_str(string).unwrap();
        ls.insert_str(0, "eifha").unwrap();
        assert_eq!(s.as_str(), ls.as_str());

        let mut ls: LimitedString = FromStr::from_str(string).unwrap();
        ls.insert_str_truncate(0, "eifha").unwrap();
        assert_eq!(s.as_str(), ls.as_str());

        let mut ls: LimitedString = FromStr::from_str(string).unwrap();
        unsafe { ls.insert_str_unchecked(0, "eifha") };
        assert_eq!(s.as_str(), ls.as_str());

        let mut s = String::from(string);
        if s.len() > 2 {
            let os = s.split_off(3);
            let mut ls: LimitedString = FromStr::from_str(string).unwrap();
            let ols = ls.split_off(3).unwrap();
            assert_eq!(s.as_str(), ls.as_str());
            assert_eq!(os.as_str(), ols.as_str());
        }

        s.clear();
        ls.clear();
        assert_eq!(s.as_str(), ls.as_str());

        let mut s = String::from(string);
        if s.len() > 4 {
            let sd: String = s.drain(..5).collect();
            let mut ls: LimitedString = FromStr::from_str(string).unwrap();
            let lsd = ls.drain(..5).unwrap();
            let lsd: String = lsd.collect();
            assert_eq!(sd.as_str(), lsd.as_str());
            assert_eq!(s.as_str(), ls.as_str());
        }

        let mut s = String::from(string);
        if s.len() > 4 {
            s.replace_range(..5, string);
            let mut ls: LimitedString = FromStr::from_str(string).unwrap();
            ls.replace_range(..5, string).unwrap();
            assert_eq!(s.as_str(), ls.as_str());
        }
    }
}
