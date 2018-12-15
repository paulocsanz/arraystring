extern crate arraystring;

#[cfg(feature = "logs")]
extern crate env_logger;

use arraystring::prelude::*;
use std::{fmt::Debug, iter::FromIterator, panic::RefUnwindSafe};

macro_rules! unwind {
    ($x: expr) => {{
        ::std::panic::catch_unwind(|| $x).map_err(|_| ())
    }};
}

static STRINGS: [&'static str; 6] = [
    "🤔🤔🤔🤔🤔🤔🤔",
    "ABCDEFGHIJKLMNOPQRSASHUDAHSDIUASH",
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
    F: Fn(&'static str) -> T + RefUnwindSafe,
    G: Fn(&'static str) -> U + RefUnwindSafe,
{
    for string in STRINGS.into_iter() {
        assert_eq!(unwind!(f(string)).map(|s| s.as_ref().to_owned()), unwind!(g(string)).map(|s| s.as_ref().to_owned()));
    }
}

#[test]
fn try_from_str() {
    assert(String::from, |s| MaxString::try_from_str(s).unwrap());
}

#[test]
fn from_str_truncate() {
    assert(String::from, MaxString::from_str_truncate);
}

#[test]
fn from_str_unchecked() {
    assert(String::from, |s| unsafe {
        MaxString::from_str_unchecked(s)
    });
}

#[test]
fn try_from_chars() {
    assert(
        |s| String::from_iter(s.chars()),
        |s| MaxString::try_from_chars(s.chars()).unwrap(),
    );
}

#[test]
fn from_chars() {
    assert(
        |s| String::from_iter(s.chars()),
        |s| MaxString::from_chars(s.chars()),
    );
}

#[test]
fn from_chars_unchecked() {
    assert(
        |s| String::from_iter(s.chars()),
        |s| unsafe { MaxString::from_chars_unchecked(s.chars()) },
    );
}

#[test]
fn try_from_iter() {
    assert(
        |s| String::from_iter(vec![s]),
        |s| MaxString::try_from_iterator(vec![s]).unwrap(),
    );
}

#[test]
fn from_iter() {
    assert(
        |s| String::from_iter(vec![s]),
        |s| MaxString::from_iterator(vec![s]),
    );
}

#[test]
fn from_iter_unchecked() {
    assert(
        |s| String::from_iter(vec![s]),
        |s| unsafe { MaxString::from_iterator_unchecked(vec![s]) },
    );
}

#[test]
fn from_utf8() {
    assert(
        |s| String::from_utf8(s.as_bytes().to_vec()).unwrap(),
        |s| MaxString::from_utf8(s.as_bytes()).unwrap(),
    );
}

#[test]
fn from_utf16() {
    let utf16 = |s: &str| s.encode_utf16().collect::<Vec<_>>();
    assert(
        |s| String::from_utf16(&utf16(s)).unwrap(),
        |s| MaxString::from_utf16(&utf16(s)).unwrap(),
    );
}

#[test]
fn from_utf8_unchecked() {
    unsafe {
        assert(
            |s| String::from_utf8_unchecked(s.as_bytes().to_vec()),
            |s| MaxString::from_utf8_unchecked(s.as_bytes()),
        );
    }
}

#[test]
fn try_push_str() {
    assert(|s| {
        let mut st = String::from(s);
        st.push_str(s);
        st
    }, |s| {
        let mut ms = MaxString::try_from_str(s).unwrap();
        ms.try_push_str(s).unwrap();
        ms
    });
}

#[test]
fn push_str() {
    assert(|s| {
        let mut st = String::from(s);
        st.push_str(s);
        st
    }, |s| {
        let mut ms = MaxString::try_from_str(s).unwrap();
        ms.push_str(s);
        ms
    });
}

#[test]
fn add_str() {
    assert(
        |s| String::from(s) + s,
        |s| MaxString::try_from_str(s).unwrap() + s
    );
}

#[test]
fn push_str_unchecked() {
    assert(|s| {
        let mut st = String::from(s);
        st.push_str(s);
        st
    }, |s| {
        let mut ms = MaxString::try_from_str(s).unwrap();
        unsafe { ms.push_str_unchecked(s) };
        ms
    });
}

#[test]
fn push() {
    assert(|s| {
        let mut s = String::from(s);
        s.push('🤔');
        s
    }, |s| {
        let mut ms = MaxString::try_from_str(s).unwrap();
        ms.try_push('🤔').unwrap();
        ms
    });
}

#[test]
fn push_unchecked() {
    assert(|s| {
        let mut s = String::from(s);
        s.push('🤔');
        s
    }, |s| {
        let mut ms = MaxString::try_from_str(s).unwrap();
        unsafe { ms.push_unchecked('🤔') };
        ms
    });
}

#[test]
fn truncate() {
    assert(|s| {
        let mut s = String::from(s);
        s.truncate(10);
        s
    }, |s| {
        let mut ms = MaxString::try_from_str(s).unwrap();
        ms.truncate(10).unwrap();
        ms
    });
}

#[test]
fn pop() {
    assert(|s| {
        let mut s = String::from(s);
        s.pop();
        s
    }, |s| {
        let mut ms = MaxString::try_from_str(s).unwrap();
        ms.pop();
        ms
    });

    assert(|s| {
        let mut s = String::from(s);
        s.pop().unwrap_or('0').to_string()
    }, |s| {
        let mut ms = MaxString::try_from_str(s).unwrap();
        ms.pop().unwrap_or('0').to_string()
    });
}

#[test]
fn remove() {
    assert(|s| {
        let mut s = String::from(s);
        s.remove(2);
        s
    }, |s| {
        let mut ms = MaxString::try_from_str(s).unwrap();
        ms.remove(2).unwrap();
        ms
    });

    assert(|s| {
        let mut s = String::from(s);
        s.remove(2).to_string()
    }, |s| {
        let mut ms = MaxString::try_from_str(s).unwrap();
        ms.remove(2).unwrap().to_string()
    });
}

#[test]
fn retain() {
    assert(|s| {
        let mut s = String::from(s);
        s.retain(|c| c == 'a');
        s
    }, |s| {
        let mut ms = MaxString::try_from_str(s).unwrap();
        ms.retain(|c| c == 'a');
        ms
    });
}

#[test]
fn try_insert() {
    assert(|s| {
        let mut s = String::from(s);
        s.insert(0, 'a');
        s
    }, |s| {
        let mut ms = MaxString::try_from_str(s).unwrap();
        ms.try_insert(0, 'a').unwrap();
        ms
    });
}

#[test]
fn insert_unchecked() {
    assert(|s| {
        let mut s = String::from(s);
        s.insert(0, 'a');
        s
    }, |s| {
        let mut ms = MaxString::try_from_str(s).unwrap();
        unsafe { ms.insert_unchecked(0, 'a') };
        ms
    });
}

#[test]
fn try_insert_str() {
    assert(|s| {
        let mut st = String::from(s);
        st.insert_str(0, s);
        st
    }, |s| {
        let mut ms = MaxString::try_from_str(s).unwrap();
        ms.try_insert_str(0, s).unwrap();
        ms
    });
}

#[test]
fn insert_str() {
    assert(|s| {
        let mut st = String::from(s);
        st.insert_str(0, s);
        st
    }, |s| {
        let mut ms = MaxString::try_from_str(s).unwrap();
        ms.insert_str(0, s).unwrap();
        ms
    });
}

#[test]
fn insert_str_unchecked() {
    assert(|s| {
        let mut st = String::from(s);
        st.insert_str(0, s);
        st
    }, |s| {
        let mut ms = MaxString::try_from_str(s).unwrap();
        unsafe { ms.insert_str_unchecked(0, s) };
        ms
    });
}

#[test]
fn clear() {
    assert(|s| {
        let mut st = String::from(s);
        st.clear();
        st
    }, |s| {
        let mut ms = MaxString::try_from_str(s).unwrap();
        ms.clear();
        ms
    });
}

#[test]
fn split_off() {
    assert(|s| {
        let mut st = String::from(s);
        let _ = st.split_off(3);
        st
    }, |s| {
        let mut ms = MaxString::try_from_str(s).unwrap();
        let _ = ms.split_off(3).unwrap();
        ms
    });

    assert(|s| {
        let mut st = String::from(s);
        st.split_off(3)
    }, |s| {
        let mut ms = MaxString::try_from_str(s).unwrap();
        ms.split_off(3).unwrap()
    });
}

#[test]
fn drain() {
    assert(|s| {
        let mut st = String::from(s);
        let _ = st.drain(..5);
        st
    }, |s| {
        let mut ms = MaxString::try_from_str(s).unwrap();
        let _ = ms.drain(..5).unwrap();
        ms
    });

    assert(|s| {
        let mut st = String::from(s);
        let s: String = st.drain(..5).collect();
        s
    }, |s| {
        let mut ms = MaxString::try_from_str(s).unwrap();
        ms.drain(..5).unwrap().collect::<String>()
    });
}

#[test]
fn replace_range() {
    assert(|s| {
        let mut st = String::from(s);
        st.replace_range(..5, s);
        st
    }, |s| {
        let mut ms = MaxString::try_from_str(s).unwrap();
        ms.replace_range(..5, s).unwrap();
        ms
    });
}

#[test]
fn string_parity() {
    setup_logger();
    assert_eq!(String::new().as_str(), MaxString::new().as_str());
    for string in STRINGS.into_iter() {
        let string = *string;
        let mut s = String::from(string);
        let mut ms = MaxString::try_from_str(string).unwrap();

        assert_eq!(s.len(), ms.len() as usize);
        assert_eq!(s.is_empty(), ms.is_empty());
        assert_eq!(s.as_mut_str(), ms.as_str_mut());
        assert_eq!(s.as_bytes(), ms.as_bytes());
    }
}
