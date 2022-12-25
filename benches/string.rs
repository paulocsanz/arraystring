use arraystring::prelude::*;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use inlinable_string::{InlinableString, InlineString, StringExt};
use std::time::Duration;

const TIME: u64 = 500;

fn bench_clones(c: &mut Criterion) {
    let params = [
        (""),
        ("a"),
        ("acb"),
        ("abcdefg"),
        (core::str::from_utf8(&['a' as u8; 15]).unwrap()),
        (core::str::from_utf8(&['b' as u8; 31]).unwrap()),
        (core::str::from_utf8(&['c' as u8; 63]).unwrap()),
        (core::str::from_utf8(&['d' as u8; 127]).unwrap()),
        (core::str::from_utf8(&['e' as u8; 255]).unwrap()),
    ];
    let mut group = c.benchmark_group("clone");
    // for ns time ops one second is plenty
    group.measurement_time(Duration::from_millis(TIME));
    group.warm_up_time(Duration::from_millis(TIME));
    for param in params {
        macro_rules! build {
            ($($str:path),*$(,)*) => {
                $(
                    if let Ok(Ok(string)) = std::panic::catch_unwind(|| { <$str>::try_from(param) }) {
                        if (&string as &str) == param {
                            group.bench_with_input(
                                BenchmarkId::new(stringify!($str), param.len()),
                                &string,
                                |b, p| b.iter(|| p.clone()),
                            );
                        }
                    }
                )*
            };
        }
        build!(
            std::string::String,
            InlineString,
            InlinableString,
            smallstring::SmallString,
            arrayvec::ArrayString<7>,
            arrayvec::ArrayString<63>,
            arrayvec::ArrayString<255>,
            ArrayString<7>,
            ArrayString<63>,
            ArrayString<255>,
            CacheString,
        );
    }
    group.finish();
}

fn bench_try_from(c: &mut Criterion) {
    let params = [
        (""),
        ("a"),
        ("acb"),
        ("abcdefg"),
        (core::str::from_utf8(&['a' as u8; 15]).unwrap()),
        (core::str::from_utf8(&['b' as u8; 31]).unwrap()),
        (core::str::from_utf8(&['c' as u8; 63]).unwrap()),
        (core::str::from_utf8(&['d' as u8; 127]).unwrap()),
        (core::str::from_utf8(&['e' as u8; 255]).unwrap()),
    ];
    let mut group = c.benchmark_group("try_from");
    // for ns time ops 0.1 second is plenty
    group.measurement_time(Duration::from_millis(TIME));
    group.warm_up_time(Duration::from_millis(TIME));
    for param in params {
        macro_rules! build {
            ($($str:path),*$(,)*) => {
                $(
                    if let Ok(Ok(string)) = std::panic::catch_unwind(|| { <$str>::try_from(param) }) {
                        if (&string as &str) == param {
                            group.bench_with_input(
                                BenchmarkId::new(stringify!($str), param.len()),
                                param,
                                |b, p| b.iter(|| <$str>::try_from(p)),
                            );
                        }
                    }
                )*
            };
        }
        build!(
            std::string::String,
            InlineString,
            InlinableString,
            smallstring::SmallString,
            arrayvec::ArrayString<7>,
            arrayvec::ArrayString<63>,
            arrayvec::ArrayString<255>,
            ArrayString<7>,
            ArrayString<63>,
            ArrayString<255>,
            CacheString,
        );
    }
    group.finish();
}

fn bench_push(c: &mut Criterion) {
    let params = [
        (""),
        ("a"),
        ("acb"),
        ("abcdefg"),
        (core::str::from_utf8(&['b' as u8; 31]).unwrap()),
        (core::str::from_utf8(&['c' as u8; 63]).unwrap()),
        (core::str::from_utf8(&['d' as u8; 127]).unwrap()),
        (core::str::from_utf8(&['e' as u8; 255]).unwrap()),
    ];
    let mut group = c.benchmark_group("push_str");
    // for ns time ops 0.1 second is plenty
    group.measurement_time(Duration::from_millis(TIME));
    group.warm_up_time(Duration::from_millis(TIME));
    for param in params {
        macro_rules! build {
            ($($str:path: $f:ident),*$(,)*) => {
                $(
                    if let Ok(mut string) = std::panic::catch_unwind(|| { <$str>::new() }) {
                        if let Ok(true) = std::panic::catch_unwind(|| {
                            let mut string = string.clone();
                            let _ = string.$f(param);
                            (&string as &str) == param // only do ones that are correct
                        }) {
                            group.bench_with_input(
                                BenchmarkId::new(stringify!($str::$f), param.len()),
                                param,
                                |b, p| b.iter(|| {
                                    let _ = string.$f(p);
                                    criterion::black_box(&mut string).clear(); // clear not inline
                                }),
                            );
                        }
                    }
                )*
            };
        }
        build!(
            std::string::String : push_str,
            InlineString : push_str,
            InlinableString : push_str,
            arrayvec::ArrayString<7> : push_str,
            arrayvec::ArrayString<63> : push_str,
            arrayvec::ArrayString<255> : push_str,
            arrayvec::ArrayString<7> : try_push_str,
            arrayvec::ArrayString<63> : try_push_str,
            arrayvec::ArrayString<255> : try_push_str,
            ArrayString<7> : push_str_truncate,
            ArrayString<63> : push_str_truncate,
            ArrayString<255> : push_str_truncate,
            ArrayString<7> : try_push_str,
            ArrayString<63> : try_push_str,
            ArrayString<255> : try_push_str,
            CacheString : push_str_truncate,
            CacheString : try_push_str,
        );
    }
    group.finish();
}

criterion_group!(string, bench_clones, bench_try_from, bench_push);

criterion_main!(string);
